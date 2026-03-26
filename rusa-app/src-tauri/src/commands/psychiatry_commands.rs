use crate::auth::{is_admin, permissions, require_role_name, validate_session_command};
use crate::db;
use crate::queries::auth::write_audit_log;
use crate::queries::medical as medical_queries;
use uuid::Uuid;

#[derive(sqlx::FromRow, serde::Serialize)]
struct PsychiatryTaskRow {
    id: Uuid,
    title: String,
    description: Option<String>,
    assigned_to: Option<Uuid>,
    assigned_by: Option<Uuid>,
    status: Option<String>,
    progress_notes: Option<String>,
    due_date: Option<chrono::DateTime<chrono::Utc>>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

async fn ensure_psychiatry_tasks_table() -> Result<(), String> {
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS psychiatry_assigned_tasks (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            title TEXT NOT NULL,
            description TEXT,
            assigned_to UUID REFERENCES users(id),
            assigned_by UUID REFERENCES users(id),
            status TEXT NOT NULL DEFAULT 'pending',
            progress_notes TEXT,
            due_date TIMESTAMPTZ,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            deleted_at TIMESTAMPTZ
        )"#,
    )
    .execute(db::get_db())
    .await
    .map_err(|e| format!("DB error creating table: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn assign_psychiatry_task(
    token: String,
    assigned_to: String,
    title: String,
    description: Option<String>,
    due_date: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "psychiatrist")?;

    ensure_psychiatry_tasks_table().await?;

    let atid = Uuid::parse_str(&assigned_to).map_err(|_| "Invalid user ID".to_string())?;
    let dd = due_date
        .as_deref()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.with_timezone(&chrono::Utc));

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO psychiatry_assigned_tasks (title, description, assigned_to, assigned_by, status, due_date) VALUES ($1,$2,$3,$4,'pending',$5) RETURNING id",
    )
    .bind(&title)
    .bind(&description)
    .bind(atid)
    .bind(session.user_id)
    .bind(dd)
    .fetch_one(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(
        Some(session.user_id),
        "ASSIGN_PSYCHIATRY_TASK",
        Some("psychiatry_assigned_tasks"),
        Some(row.0),
        None,
        Some(serde_json::json!({ "title": title, "assigned_to": assigned_to })),
    )
    .await;

    Ok(row.0.to_string())
}

#[tauri::command]
pub async fn get_psychiatry_tasks(token: String) -> Result<Vec<serde_json::Value>, String> {
    let session = validate_session_command(&token).await?;

    ensure_psychiatry_tasks_table().await?;

    let rows = if permissions::has_permission(&session.role_name, "psychiatrist") {
        sqlx::query_as::<_, PsychiatryTaskRow>(
            "SELECT id, title, description, assigned_to, assigned_by, status, progress_notes, due_date, created_at FROM psychiatry_assigned_tasks WHERE assigned_by = $1 AND deleted_at IS NULL ORDER BY created_at DESC",
        )
        .bind(session.user_id)
        .fetch_all(db::get_db())
        .await
    } else {
        sqlx::query_as::<_, PsychiatryTaskRow>(
            "SELECT id, title, description, assigned_to, assigned_by, status, progress_notes, due_date, created_at FROM psychiatry_assigned_tasks WHERE assigned_to = $1 AND deleted_at IS NULL ORDER BY created_at DESC",
        )
        .bind(session.user_id)
        .fetch_all(db::get_db())
        .await
    }
    .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows
        .into_iter()
        .map(|r| serde_json::to_value(r).unwrap_or_default())
        .collect())
}

#[tauri::command]
pub async fn update_psychiatry_task_status(
    token: String,
    task_id: String,
    status: String,
    progress_notes: Option<String>,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;

    ensure_psychiatry_tasks_table().await?;

    let tid = Uuid::parse_str(&task_id).map_err(|_| "Invalid task ID".to_string())?;

    sqlx::query(
        "UPDATE psychiatry_assigned_tasks SET status = $1, progress_notes = COALESCE($2, progress_notes) WHERE id = $3 AND (assigned_to = $4 OR assigned_by = $4) AND deleted_at IS NULL",
    )
    .bind(&status)
    .bind(&progress_notes)
    .bind(tid)
    .bind(session.user_id)
    .execute(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(
        Some(session.user_id),
        "UPDATE_PSYCHIATRY_TASK_STATUS",
        Some("psychiatry_assigned_tasks"),
        Some(tid),
        None,
        Some(serde_json::json!({ "status": status })),
    )
    .await;

    Ok(())
}

#[tauri::command]
pub async fn register_patient(
    token: String,
    patient_id: String,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "psychiatrist")?;

    let pid = Uuid::parse_str(&patient_id).map_err(|_| "Invalid patient ID".to_string())?;

    let id = medical_queries::insert_psychiatric_patient(pid)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "REGISTER_PATIENT", Some("psychiatric_patients"), Some(id), None, Some(serde_json::json!({ "patient_id": patient_id }))).await;
    Ok(id.to_string())
}

#[tauri::command]
pub async fn get_patients(token: String) -> Result<Vec<serde_json::Value>, String> {
    let session = validate_session_command(&token).await?;
    if session.role_name != "psychiatrist" && session.role_name != "psychiatrist_assistant" && !is_admin(&session) {
        return Err("Only psychiatrist or assistant can view patients".to_string());
    }

    let rows = medical_queries::get_all_psychiatric_patients()
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn schedule_appointment(
    token: String,
    patient_id: String,
    scheduled_at: String,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "psychiatrist")?;

    let pid = Uuid::parse_str(&patient_id).map_err(|_| "Invalid patient ID".to_string())?;
    let sched = chrono::DateTime::parse_from_rfc3339(&scheduled_at)
        .map(|d| d.with_timezone(&chrono::Utc))
        .map_err(|_| "Invalid scheduled_at format".to_string())?;

    let id = medical_queries::insert_appointment(pid, sched, session.user_id)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "SCHEDULE_APPOINTMENT", Some("appointments"), Some(id), None, Some(serde_json::json!({ "patient_id": patient_id }))).await;
    Ok(id.to_string())
}

#[tauri::command]
pub async fn get_appointments(token: String) -> Result<Vec<serde_json::Value>, String> {
    let session = validate_session_command(&token).await?;

    let rows = if permissions::has_permission(&session.role_name, "psychiatrist")
        || permissions::has_permission(&session.role_name, "psychiatrist_assistant")
    {
        medical_queries::get_all_appointments().await
    } else {
        medical_queries::get_patient_appointments(session.user_id).await
    }
    .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn complete_appointment(
    token: String,
    appointment_id: String,
    findings: Option<String>,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "psychiatrist")?;

    let aid = Uuid::parse_str(&appointment_id).map_err(|_| "Invalid appointment ID".to_string())?;

    medical_queries::complete_appointment(aid, findings.as_deref(), session.user_id)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "COMPLETE_APPOINTMENT", Some("appointments"), Some(aid), None, None).await;
    Ok(())
}

#[tauri::command]
pub async fn add_recovery_log(
    token: String,
    patient_id: String,
    entry_date: String,
    status: String,
    notes: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "psychiatrist")?;

    let pid = Uuid::parse_str(&patient_id).map_err(|_| "Invalid patient ID".to_string())?;
    let edate = chrono::NaiveDate::parse_from_str(&entry_date, "%Y-%m-%d")
        .map_err(|_| "Invalid entry_date format (expected YYYY-MM-DD)".to_string())?;

    let id = medical_queries::insert_recovery_log(pid, session.user_id, edate, &status, notes.as_deref())
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "ADD_RECOVERY_LOG", Some("patient_recovery_log"), Some(id), None, None).await;
    Ok(id.to_string())
}

#[tauri::command]
pub async fn get_recovery_log(
    token: String,
    patient_id: String,
) -> Result<Vec<serde_json::Value>, String> {
    let session = validate_session_command(&token).await?;
    if session.role_name != "psychiatrist" && session.role_name != "psychiatrist_assistant" && !is_admin(&session) {
        return Err("Only psychiatrist or assistant can view recovery logs".to_string());
    }

    let pid = Uuid::parse_str(&patient_id).map_err(|_| "Invalid patient ID".to_string())?;

    let rows = medical_queries::get_recovery_log_for_patient(pid)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}
