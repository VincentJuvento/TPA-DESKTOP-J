use crate::auth::{permissions, require_role_name, validate_session_command};
use crate::db;
use crate::queries::auth::write_audit_log;
use crate::queries::budget as budget_queries;
use crate::queries::medical as medical_queries;
use uuid::Uuid;

#[derive(sqlx::FromRow, serde::Serialize)]
struct MedicalTaskRow {
    id: Uuid,
    title: String,
    description: Option<String>,
    assigned_to: Option<Uuid>,
    assigned_by: Option<Uuid>,
    status: Option<String>,
    progress_notes: Option<String>,
    activity_logs: Option<serde_json::Value>,
    due_date: Option<chrono::DateTime<chrono::Utc>>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
    conclusion_requested_at: Option<chrono::DateTime<chrono::Utc>>,
    conclusion_requested_by: Option<Uuid>,
    conclusion_approved_at: Option<chrono::DateTime<chrono::Utc>>,
    conclusion_approved_by: Option<Uuid>,
    final_notes: Option<String>,
}

async fn ensure_medical_tasks_table() -> Result<(), String> {
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS medical_assigned_tasks (
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

    let alter_stmts = [
        "ALTER TABLE medical_assigned_tasks ADD COLUMN IF NOT EXISTS activity_logs JSONB DEFAULT '[]'",
        "ALTER TABLE medical_assigned_tasks ADD COLUMN IF NOT EXISTS conclusion_requested_at TIMESTAMPTZ",
        "ALTER TABLE medical_assigned_tasks ADD COLUMN IF NOT EXISTS conclusion_requested_by UUID REFERENCES users(id)",
        "ALTER TABLE medical_assigned_tasks ADD COLUMN IF NOT EXISTS conclusion_approved_at TIMESTAMPTZ",
        "ALTER TABLE medical_assigned_tasks ADD COLUMN IF NOT EXISTS conclusion_approved_by UUID REFERENCES users(id)",
        "ALTER TABLE medical_assigned_tasks ADD COLUMN IF NOT EXISTS final_notes TEXT",
    ];
    for stmt in &alter_stmts {
        sqlx::query(stmt)
            .execute(db::get_db())
            .await
            .map_err(|e| format!("DB migration error: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
pub async fn assign_medical_task(
    token: String,
    assigned_to: String,
    title: String,
    description: Option<String>,
    due_date: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "head_of_medicine")?;

    ensure_medical_tasks_table().await?;

    let atid = Uuid::parse_str(&assigned_to).map_err(|_| "Invalid user ID".to_string())?;
    let dd = due_date
        .as_deref()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.with_timezone(&chrono::Utc));

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO medical_assigned_tasks (title, description, assigned_to, assigned_by, status, due_date) VALUES ($1,$2,$3,$4,'pending',$5) RETURNING id",
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
        "ASSIGN_MEDICAL_TASK",
        Some("medical_assigned_tasks"),
        Some(row.0),
        None,
        Some(serde_json::json!({ "title": title, "assigned_to": assigned_to })),
    )
    .await;

    Ok(row.0.to_string())
}

const MEDICAL_TASK_SELECT: &str =
    "SELECT id, title, description, assigned_to, assigned_by, status, progress_notes, activity_logs, \
     due_date, created_at, conclusion_requested_at, conclusion_requested_by, \
     conclusion_approved_at, conclusion_approved_by, final_notes \
     FROM medical_assigned_tasks";

#[tauri::command]
pub async fn get_medical_tasks(token: String) -> Result<Vec<serde_json::Value>, String> {
    let session = validate_session_command(&token).await?;

    ensure_medical_tasks_table().await?;

    let rows = if permissions::has_permission(&session.role_name, "head_of_medicine") {
        sqlx::query_as::<_, MedicalTaskRow>(
            &format!("{} WHERE assigned_by = $1 AND deleted_at IS NULL ORDER BY created_at DESC", MEDICAL_TASK_SELECT),
        )
        .bind(session.user_id)
        .fetch_all(db::get_db())
        .await
    } else {
        sqlx::query_as::<_, MedicalTaskRow>(
            &format!("{} WHERE assigned_to = $1 AND deleted_at IS NULL ORDER BY created_at DESC", MEDICAL_TASK_SELECT),
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
pub async fn get_medical_task(token: String, task_id: String) -> Result<serde_json::Value, String> {
    let session = validate_session_command(&token).await?;
    ensure_medical_tasks_table().await?;
    let tid = Uuid::parse_str(&task_id).map_err(|_| "Invalid task ID".to_string())?;

    let row = sqlx::query_as::<_, MedicalTaskRow>(
        &format!("{} WHERE id = $1 AND (assigned_to = $2 OR assigned_by = $2) AND deleted_at IS NULL", MEDICAL_TASK_SELECT),
    )
    .bind(tid)
    .bind(session.user_id)
    .fetch_optional(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    row.map(|r| serde_json::to_value(r).unwrap_or_default())
        .ok_or_else(|| "Task not found".to_string())
}

#[tauri::command]
pub async fn update_medical_task_status(
    token: String,
    task_id: String,
    status: String,
    progress_notes: Option<String>,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;

    ensure_medical_tasks_table().await?;

    let tid = Uuid::parse_str(&task_id).map_err(|_| "Invalid task ID".to_string())?;

    // Fetch task to check authorization and current status
    let task: Option<(Option<Uuid>, Option<Uuid>, Option<String>)> = sqlx::query_as(
        "SELECT assigned_to, assigned_by, status FROM medical_assigned_tasks WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(tid)
    .fetch_optional(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let (assigned_to, assigned_by, current_status) = task.ok_or_else(|| "Task not found".to_string())?;

    let is_assigner = assigned_by.map(|id| id == session.user_id).unwrap_or(false);
    let is_assignee = assigned_to.map(|id| id == session.user_id).unwrap_or(false);
    let is_head = permissions::has_permission(&session.role_name, "head_of_medicine");

    if !is_assigner && !is_assignee && !is_head {
        return Err("You do not have permission to update this task".to_string());
    }

    // Subordinates cannot set status to 'completed'
    if !is_assigner && !is_head && status == "completed" {
        return Err("Only the task assigner can mark a task as completed".to_string());
    }

    // Subordinates cannot change status when conclusion is requested (unless assigner)
    if !is_assigner && !is_head && current_status.as_deref() == Some("conclusion_requested") {
        return Err("Status cannot be changed while awaiting assigner review".to_string());
    }

    let log_entry = serde_json::json!([{
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "author_id": session.user_id.to_string(),
        "author_name": session.full_name,
        "content": format!("Status changed to '{}'{}",
            status,
            progress_notes.as_deref().map(|n| format!(": {}", n)).unwrap_or_default()),
        "log_type": "status_change"
    }]);

    sqlx::query(
        "UPDATE medical_assigned_tasks SET status = $1, \
         progress_notes = COALESCE($2, progress_notes), \
         activity_logs = COALESCE(activity_logs, '[]'::jsonb) || $3 \
         WHERE id = $4 AND deleted_at IS NULL",
    )
    .bind(&status)
    .bind(&progress_notes)
    .bind(&log_entry)
    .bind(tid)
    .execute(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(
        Some(session.user_id),
        "UPDATE_MEDICAL_TASK_STATUS",
        Some("medical_assigned_tasks"),
        Some(tid),
        None,
        Some(serde_json::json!({ "status": status })),
    )
    .await;

    Ok(())
}

#[tauri::command]
pub async fn append_medical_task_activity_log(
    token: String,
    task_id: String,
    content: String,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    ensure_medical_tasks_table().await?;
    let tid = Uuid::parse_str(&task_id).map_err(|_| "Invalid task ID".to_string())?;

    // Verify access
    let task: Option<(Option<String>,)> = sqlx::query_as(
        "SELECT status FROM medical_assigned_tasks WHERE id = $1 AND (assigned_to = $2 OR assigned_by = $2) AND deleted_at IS NULL",
    )
    .bind(tid)
    .bind(session.user_id)
    .fetch_optional(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    if task.is_none() {
        return Err("Task not found or access denied".to_string());
    }

    let entry = serde_json::json!([{
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "author_id": session.user_id.to_string(),
        "author_name": session.full_name,
        "content": content,
        "log_type": "progress_update"
    }]);

    sqlx::query(
        "UPDATE medical_assigned_tasks SET activity_logs = COALESCE(activity_logs, '[]'::jsonb) || $1 WHERE id = $2 AND deleted_at IS NULL",
    )
    .bind(&entry)
    .bind(tid)
    .execute(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "APPEND_MEDICAL_TASK_LOG", Some("medical_assigned_tasks"), Some(tid), None, None).await;
    Ok(())
}

#[tauri::command]
pub async fn request_medical_task_conclusion(
    token: String,
    task_id: String,
    notes: String,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    ensure_medical_tasks_table().await?;
    let tid = Uuid::parse_str(&task_id).map_err(|_| "Invalid task ID".to_string())?;

    if notes.trim().is_empty() {
        return Err("Conclusion notes are required".to_string());
    }

    let task: Option<(Option<String>,)> = sqlx::query_as(
        "SELECT status FROM medical_assigned_tasks WHERE id = $1 AND assigned_to = $2 AND deleted_at IS NULL",
    )
    .bind(tid)
    .bind(session.user_id)
    .fetch_optional(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    match task {
        None => return Err("Task not found or not assigned to you".to_string()),
        Some((status,)) => {
            let s = status.as_deref().unwrap_or("pending");
            if s == "conclusion_requested" || s == "completed" {
                return Err("Task is already in conclusion or completed state".to_string());
            }
        }
    }

    let log_entry = serde_json::json!([{
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "author_id": session.user_id.to_string(),
        "author_name": session.full_name,
        "content": notes,
        "log_type": "conclusion_requested"
    }]);

    sqlx::query(
        "UPDATE medical_assigned_tasks SET status = 'conclusion_requested', \
         conclusion_requested_at = NOW(), conclusion_requested_by = $1, \
         final_notes = $2, \
         activity_logs = COALESCE(activity_logs, '[]'::jsonb) || $3 \
         WHERE id = $4 AND deleted_at IS NULL",
    )
    .bind(session.user_id)
    .bind(&notes)
    .bind(&log_entry)
    .bind(tid)
    .execute(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "REQUEST_MEDICAL_TASK_CONCLUSION", Some("medical_assigned_tasks"), Some(tid), None, None).await;
    Ok(())
}

#[tauri::command]
pub async fn review_medical_task_conclusion(
    token: String,
    task_id: String,
    decision: String,
    review_notes: Option<String>,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    ensure_medical_tasks_table().await?;
    let tid = Uuid::parse_str(&task_id).map_err(|_| "Invalid task ID".to_string())?;

    let task: Option<(Option<String>,)> = sqlx::query_as(
        "SELECT status FROM medical_assigned_tasks WHERE id = $1 AND assigned_by = $2 AND deleted_at IS NULL",
    )
    .bind(tid)
    .bind(session.user_id)
    .fetch_optional(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    match &task {
        None => return Err("Task not found or you are not the assigner".to_string()),
        Some((status,)) => {
            if status.as_deref() != Some("conclusion_requested") {
                return Err("Task is not awaiting conclusion review".to_string());
            }
        }
    }

    let new_status = match decision.as_str() {
        "approve" => "completed",
        "reject" => "in_progress",
        _ => return Err("Invalid decision. Use 'approve' or 'reject'".to_string()),
    };

    let log_type = if new_status == "completed" { "conclusion_approved" } else { "conclusion_rejected" };
    let log_entry = serde_json::json!([{
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "author_id": session.user_id.to_string(),
        "author_name": session.full_name,
        "content": review_notes.as_deref().unwrap_or(""),
        "log_type": log_type
    }]);

    if new_status == "completed" {
        sqlx::query(
            "UPDATE medical_assigned_tasks SET status = 'completed', \
             conclusion_approved_at = NOW(), conclusion_approved_by = $1, \
             activity_logs = COALESCE(activity_logs, '[]'::jsonb) || $2 \
             WHERE id = $3 AND deleted_at IS NULL",
        )
        .bind(session.user_id)
        .bind(&log_entry)
        .bind(tid)
        .execute(db::get_db())
        .await
        .map_err(|e| format!("DB error: {}", e))?;
    } else {
        sqlx::query(
            "UPDATE medical_assigned_tasks SET status = 'in_progress', \
             activity_logs = COALESCE(activity_logs, '[]'::jsonb) || $1 \
             WHERE id = $2 AND deleted_at IS NULL",
        )
        .bind(&log_entry)
        .bind(tid)
        .execute(db::get_db())
        .await
        .map_err(|e| format!("DB error: {}", e))?;
    }

    let _ = write_audit_log(Some(session.user_id), "REVIEW_MEDICAL_TASK_CONCLUSION", Some("medical_assigned_tasks"), Some(tid), None, Some(serde_json::json!({ "decision": decision }))).await;
    Ok(())
}

#[tauri::command]
pub async fn allocate_shift(
    token: String,
    staff_id: String,
    shift_start: String,
    shift_end: String,
    notes: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "head_of_medicine")?;

    let sid = Uuid::parse_str(&staff_id).map_err(|_| "Invalid staff ID".to_string())?;
    let sstart = chrono::DateTime::parse_from_rfc3339(&shift_start)
        .map(|d| d.with_timezone(&chrono::Utc))
        .map_err(|_| "Invalid shift_start format".to_string())?;
    let send = chrono::DateTime::parse_from_rfc3339(&shift_end)
        .map(|d| d.with_timezone(&chrono::Utc))
        .map_err(|_| "Invalid shift_end format".to_string())?;

    let id = medical_queries::insert_medical_shift(sid, sstart, send, notes.as_deref(), session.user_id)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "ALLOCATE_SHIFT", Some("medical_shifts"), Some(id), None, Some(serde_json::json!({ "staff_id": staff_id }))).await;
    Ok(id.to_string())
}

#[tauri::command]
pub async fn get_shifts(token: String) -> Result<Vec<serde_json::Value>, String> {
    let session = validate_session_command(&token).await?;

    let rows = if permissions::has_permission(&session.role_name, "head_of_medicine") {
        medical_queries::get_all_medical_shifts().await
    } else {
        medical_queries::get_user_medical_shifts(session.user_id).await
    }
    .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn get_medical_inventory(token: String) -> Result<Vec<serde_json::Value>, String> {
    let _session = validate_session_command(&token).await?;

    let rows = medical_queries::get_all_medical_inventory()
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn update_medical_inventory(
    token: String,
    item_name: String,
    category: Option<String>,
    quantity: i32,
    unit: Option<String>,
    expiry_date: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    let edate = expiry_date.as_deref()
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());

    let id = medical_queries::insert_medical_inventory(
        &item_name,
        category.as_deref(),
        quantity,
        unit.as_deref(),
        edate,
        session.user_id,
    )
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "UPDATE_MEDICAL_INVENTORY", Some("medical_inventory"), Some(id), None, Some(serde_json::json!({ "item_name": item_name, "quantity": quantity }))).await;
    Ok(id.to_string())
}

#[tauri::command]
pub async fn create_patient_record(
    token: String,
    patient_id: String,
    diagnosis: Option<String>,
    treatment: Option<String>,
    medications: Option<String>,
    notes: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    let pid = Uuid::parse_str(&patient_id).map_err(|_| "Invalid patient ID".to_string())?;

    let id = medical_queries::insert_patient_record(
        pid,
        diagnosis.as_deref(),
        treatment.as_deref(),
        medications.as_deref(),
        notes.as_deref(),
        session.user_id,
    )
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "CREATE_PATIENT_RECORD", Some("patient_records"), Some(id), None, None).await;
    Ok(id.to_string())
}

#[tauri::command]
pub async fn get_patient_records(
    token: String,
    patient_id: String,
) -> Result<Vec<serde_json::Value>, String> {
    let session = validate_session_command(&token).await?;
    // Must be medical staff (subsystem check) or admin
    if session.subsystem.as_deref() != Some("medical") && session.tier < 4 {
        return Err("Only medical staff can view patient records".to_string());
    }

    let pid = Uuid::parse_str(&patient_id).map_err(|_| "Invalid patient ID".to_string())?;

    let rows = medical_queries::get_patient_records(pid)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn add_specialization(
    token: String,
    staff_id: String,
    specialization: String,
    certified_at: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    let sid = Uuid::parse_str(&staff_id).map_err(|_| "Invalid staff ID".to_string())?;
    let cdate = certified_at.as_deref()
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());

    let id = medical_queries::insert_staff_specialization(sid, &specialization, cdate)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "ADD_SPECIALIZATION", Some("staff_specializations"), Some(id), None, Some(serde_json::json!({ "staff_id": staff_id, "specialization": specialization }))).await;
    Ok(id.to_string())
}

#[tauri::command]
pub async fn get_staff_specializations(token: String) -> Result<Vec<serde_json::Value>, String> {
    let _session = validate_session_command(&token).await?;

    let rows = medical_queries::get_all_staff_specializations()
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn submit_budget_request(
    token: String,
    title: String,
    description: String,
    amount: f64,
    items: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    if session.tier < 2 {
        return Err("Only department heads and above can submit budget requests".to_string());
    }

    let amount_dec = rust_decimal::Decimal::try_from(amount)
        .map_err(|e| format!("Invalid amount value '{}': {}", amount, e))?;
    let items_json = items.as_deref().and_then(|s| {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return None;
        }
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(trimmed) {
            return Some(v);
        }
        let lines = trimmed
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(|l| serde_json::Value::String(l.to_string()))
            .collect::<Vec<_>>();
        Some(serde_json::Value::Array(lines))
    });

    let id = budget_queries::insert_budget_request(
        &title,
        &description,
        amount_dec,
        items_json,
        session.user_id,
    )
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "SUBMIT_BUDGET_REQUEST", Some("budget_requests"), Some(id), None, Some(serde_json::json!({ "title": title, "amount": amount }))).await;
    Ok(id.to_string())
}

#[tauri::command]
pub async fn submit_expenditure_report(
    token: String,
    title: String,
    description: String,
    total_amount: f64,
    items: Option<String>,
    invoice_data: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;

    let total_amount_dec = rust_decimal::Decimal::try_from(total_amount)
        .map_err(|e| format!("Invalid total_amount value '{}': {}", total_amount, e))?;
    let items_json = items.as_deref().and_then(|s| {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return None;
        }
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(trimmed) {
            return Some(v);
        }
        let lines = trimmed
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(|l| serde_json::Value::String(l.to_string()))
            .collect::<Vec<_>>();
        Some(serde_json::Value::Array(lines))
    });

    let id = budget_queries::insert_expenditure_report(
        &title,
        &description,
        total_amount_dec,
        items_json,
        invoice_data.as_deref(),
        session.user_id,
    )
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "SUBMIT_EXPENDITURE_REPORT", Some("expenditure_reports"), Some(id), None, Some(serde_json::json!({ "title": title, "total_amount": total_amount }))).await;
    Ok(id.to_string())
}
