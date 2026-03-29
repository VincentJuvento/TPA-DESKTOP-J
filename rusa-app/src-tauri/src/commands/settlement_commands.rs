use crate::auth::{is_admin, permissions, require_role, require_role_name, validate_session_command};
use crate::db;
use crate::queries::auth::write_audit_log;
use uuid::Uuid;

#[derive(sqlx::FromRow, serde::Serialize)]
struct SettlementRow {
    id: Uuid,
    name: String,
    location: Option<String>,
    status: Option<String>,
    commander_id: Option<Uuid>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct SettlerTaskRow {
    id: Uuid,
    title: String,
    description: Option<String>,
    status: Option<String>,
    assigned_to: Option<Uuid>,
    assigned_by: Option<Uuid>,
    settlement_id: Option<Uuid>,
    due_date: Option<chrono::NaiveDate>,
    progress_notes: Option<String>,
    activity_logs: Option<serde_json::Value>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
    conclusion_requested_at: Option<chrono::DateTime<chrono::Utc>>,
    conclusion_requested_by: Option<Uuid>,
    conclusion_approved_at: Option<chrono::DateTime<chrono::Utc>>,
    conclusion_approved_by: Option<Uuid>,
    final_notes: Option<String>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct SupplyRequestRow {
    id: Uuid,
    title: String,
    description: Option<String>,
    items: Option<serde_json::Value>,
    status: Option<String>,
    settlement_id: Option<Uuid>,
    requested_by: Option<Uuid>,
    reviewed_by: Option<Uuid>,
    notes: Option<String>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct AnomalyReportRow {
    id: Uuid,
    title: String,
    description: Option<String>,
    severity: Option<String>,
    status: Option<String>,
    settlement_id: Option<Uuid>,
    reported_by: Option<Uuid>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct InventoryRow {
    id: Uuid,
    settlement_id: Option<Uuid>,
    item_name: String,
    category: Option<String>,
    quantity: Option<i32>,
    unit: Option<String>,
    updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

async fn ensure_settler_task_columns() -> Result<(), String> {
    let alter_stmts = [
        "ALTER TABLE settler_tasks ADD COLUMN IF NOT EXISTS activity_logs JSONB DEFAULT '[]'",
        "ALTER TABLE settler_tasks ADD COLUMN IF NOT EXISTS conclusion_requested_at TIMESTAMPTZ",
        "ALTER TABLE settler_tasks ADD COLUMN IF NOT EXISTS conclusion_requested_by UUID REFERENCES users(id)",
        "ALTER TABLE settler_tasks ADD COLUMN IF NOT EXISTS conclusion_approved_at TIMESTAMPTZ",
        "ALTER TABLE settler_tasks ADD COLUMN IF NOT EXISTS conclusion_approved_by UUID REFERENCES users(id)",
        "ALTER TABLE settler_tasks ADD COLUMN IF NOT EXISTS final_notes TEXT",
    ];
    for stmt in &alter_stmts {
        sqlx::query(stmt)
            .execute(db::get_db())
            .await
            .map_err(|e| format!("DB error: {}", e))?;
    }
    Ok(())
}

const SETTLER_TASK_SELECT: &str =
    "SELECT id, title, description, status, assigned_to, assigned_by, settlement_id, due_date, \
     progress_notes, activity_logs, created_at, conclusion_requested_at, conclusion_requested_by, \
     conclusion_approved_at, conclusion_approved_by, final_notes FROM settler_tasks";

#[tauri::command]
pub async fn get_settlements(token: String) -> Result<Vec<serde_json::Value>, String> {
    let _session = validate_session_command(&token).await?;

    let rows = sqlx::query_as::<_, SettlementRow>(
        "SELECT id, name, location, status, commander_id, created_at FROM settlements WHERE deleted_at IS NULL ORDER BY name"
    )
    .fetch_all(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn get_settler_tasks(token: String) -> Result<Vec<serde_json::Value>, String> {
    let session = validate_session_command(&token).await?;
    ensure_settler_task_columns().await?;

    let rows = if permissions::has_permission(&session.role_name, "settler_commander") || session.tier >= 3 {
        sqlx::query_as::<_, SettlerTaskRow>(
            &format!("{} WHERE deleted_at IS NULL ORDER BY created_at DESC", SETTLER_TASK_SELECT),
        ).fetch_all(db::get_db()).await
    } else {
        sqlx::query_as::<_, SettlerTaskRow>(
            &format!("{} WHERE assigned_to = $1 AND deleted_at IS NULL ORDER BY created_at DESC", SETTLER_TASK_SELECT),
        ).bind(session.user_id).fetch_all(db::get_db()).await
    }.map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn assign_settler_task(
    token: String,
    assigned_to: String,
    title: String,
    description: Option<String>,
    due_date: Option<String>,
    settlement_id: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "settler_commander")?;
    ensure_settler_task_columns().await?;

    let atid = Uuid::parse_str(&assigned_to).map_err(|_| "Invalid user ID".to_string())?;
    let sid = settlement_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());
    let ddate = due_date.as_deref()
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO settler_tasks (title, description, status, assigned_to, assigned_by, settlement_id, due_date) VALUES ($1,$2,'pending',$3,$4,$5,$6) RETURNING id"
    )
    .bind(&title).bind(&description).bind(atid).bind(session.user_id).bind(sid).bind(ddate)
    .fetch_one(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "ASSIGN_SETTLER_TASK", Some("settler_tasks"), Some(row.0), None, Some(serde_json::json!({ "title": title, "assigned_to": assigned_to }))).await;
    Ok(row.0.to_string())
}

#[tauri::command]
pub async fn update_task_progress(
    token: String,
    task_id: String,
    progress_notes: String,
    status: String,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    ensure_settler_task_columns().await?;
    let tid = Uuid::parse_str(&task_id).map_err(|_| "Invalid task ID".to_string())?;

    let task: Option<(Option<Uuid>, Option<Uuid>, Option<String>)> = sqlx::query_as(
        "SELECT assigned_to, assigned_by, status FROM settler_tasks WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(tid)
    .fetch_optional(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let (assigned_to, assigned_by, current_status) = task.ok_or_else(|| "Task not found".to_string())?;
    let is_assigner = assigned_by.map(|id| id == session.user_id).unwrap_or(false);
    let is_assignee = assigned_to.map(|id| id == session.user_id).unwrap_or(false);
    let is_head = permissions::has_permission(&session.role_name, "settler_commander") || session.tier >= 3;

    if !is_assigner && !is_assignee && !is_head {
        return Err("You do not have permission to update this task".to_string());
    }
    if !is_assigner && !is_head && status == "completed" {
        return Err("Only the task assigner can mark a task as completed".to_string());
    }
    if !is_assigner && !is_head && current_status.as_deref() == Some("conclusion_requested") {
        return Err("Status cannot be changed while awaiting assigner review".to_string());
    }

    let log_entry = serde_json::json!([{
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "author_id": session.user_id.to_string(),
        "author_name": session.full_name,
        "content": format!("Status changed to '{}'{}",
            status,
            if progress_notes.is_empty() { "".to_string() } else { format!(": {}", progress_notes) }),
        "log_type": "status_change"
    }]);

    sqlx::query("UPDATE settler_tasks SET progress_notes = $1, status = $2, activity_logs = COALESCE(activity_logs, '[]'::jsonb) || $3 WHERE id = $4 AND deleted_at IS NULL")
        .bind(&progress_notes).bind(&status).bind(&log_entry).bind(tid)
        .execute(db::get_db())
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "UPDATE_TASK_PROGRESS", Some("settler_tasks"), Some(tid), None, Some(serde_json::json!({ "status": status }))).await;
    Ok(())
}

#[tauri::command]
pub async fn get_settler_task(token: String, task_id: String) -> Result<serde_json::Value, String> {
    let session = validate_session_command(&token).await?;
    ensure_settler_task_columns().await?;
    let tid = Uuid::parse_str(&task_id).map_err(|_| "Invalid task ID".to_string())?;

    let row = sqlx::query_as::<_, SettlerTaskRow>(
        &format!("{} WHERE id = $1 AND (assigned_to = $2 OR assigned_by = $2 OR $3 >= 3) AND deleted_at IS NULL", SETTLER_TASK_SELECT),
    )
    .bind(tid)
    .bind(session.user_id)
    .bind(session.tier)
    .fetch_optional(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    row.map(|r| serde_json::to_value(r).unwrap_or_default())
        .ok_or_else(|| "Task not found".to_string())
}

#[tauri::command]
pub async fn append_settler_task_activity_log(
    token: String,
    task_id: String,
    content: String,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    ensure_settler_task_columns().await?;
    let tid = Uuid::parse_str(&task_id).map_err(|_| "Invalid task ID".to_string())?;

    let task: Option<(Option<String>,)> = sqlx::query_as(
        "SELECT status FROM settler_tasks WHERE id = $1 AND (assigned_to = $2 OR assigned_by = $2 OR $3 >= 3) AND deleted_at IS NULL",
    )
    .bind(tid)
    .bind(session.user_id)
    .bind(session.tier)
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
        "UPDATE settler_tasks SET activity_logs = COALESCE(activity_logs, '[]'::jsonb) || $1 WHERE id = $2 AND deleted_at IS NULL",
    )
    .bind(&entry)
    .bind(tid)
    .execute(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "APPEND_SETTLER_TASK_LOG", Some("settler_tasks"), Some(tid), None, None).await;
    Ok(())
}

#[tauri::command]
pub async fn request_settler_task_conclusion(
    token: String,
    task_id: String,
    notes: String,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    ensure_settler_task_columns().await?;
    let tid = Uuid::parse_str(&task_id).map_err(|_| "Invalid task ID".to_string())?;

    if notes.trim().is_empty() {
        return Err("Conclusion notes are required".to_string());
    }

    let task: Option<(Option<String>,)> = sqlx::query_as(
        "SELECT status FROM settler_tasks WHERE id = $1 AND assigned_to = $2 AND deleted_at IS NULL",
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
        "UPDATE settler_tasks SET status = 'conclusion_requested', \
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

    let _ = write_audit_log(Some(session.user_id), "REQUEST_SETTLER_TASK_CONCLUSION", Some("settler_tasks"), Some(tid), None, None).await;
    Ok(())
}

#[tauri::command]
pub async fn review_settler_task_conclusion(
    token: String,
    task_id: String,
    decision: String,
    review_notes: Option<String>,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    ensure_settler_task_columns().await?;
    let tid = Uuid::parse_str(&task_id).map_err(|_| "Invalid task ID".to_string())?;

    let task: Option<(Option<String>,)> = sqlx::query_as(
        "SELECT status FROM settler_tasks WHERE id = $1 AND assigned_by = $2 AND deleted_at IS NULL",
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
            "UPDATE settler_tasks SET status = 'completed', \
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
            "UPDATE settler_tasks SET status = 'in_progress', \
             activity_logs = COALESCE(activity_logs, '[]'::jsonb) || $1 \
             WHERE id = $2 AND deleted_at IS NULL",
        )
        .bind(&log_entry)
        .bind(tid)
        .execute(db::get_db())
        .await
        .map_err(|e| format!("DB error: {}", e))?;
    }

    let _ = write_audit_log(Some(session.user_id), "REVIEW_SETTLER_TASK_CONCLUSION", Some("settler_tasks"), Some(tid), None, Some(serde_json::json!({ "decision": decision }))).await;
    Ok(())
}

#[tauri::command]
pub async fn submit_supply_request(
    token: String,
    settlement_id: Option<String>,
    title: String,
    description: Option<String>,
    items: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "settler_commander")?;

    let sid = settlement_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());
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

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO supply_requests (title, description, items, status, settlement_id, requested_by) VALUES ($1,$2,$3,'pending',$4,$5) RETURNING id"
    )
    .bind(&title).bind(&description).bind(&items_json).bind(sid).bind(session.user_id)
    .fetch_one(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "SUBMIT_SUPPLY_REQUEST", Some("supply_requests"), Some(row.0), None, Some(serde_json::json!({ "title": title }))).await;
    Ok(row.0.to_string())
}

#[tauri::command]
pub async fn get_supply_requests(token: String) -> Result<Vec<serde_json::Value>, String> {
    let _session = validate_session_command(&token).await?;

    let rows = sqlx::query_as::<_, SupplyRequestRow>(
        "SELECT id, title, description, items, status, settlement_id, requested_by, reviewed_by, notes, created_at FROM supply_requests WHERE deleted_at IS NULL ORDER BY created_at DESC"
    )
    .fetch_all(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn review_supply_request(
    token: String,
    request_id: String,
    decision: String,
    notes: Option<String>,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    require_role(&session, 3)?;

    let rid = Uuid::parse_str(&request_id).map_err(|_| "Invalid request ID".to_string())?;

    sqlx::query("UPDATE supply_requests SET status = $1, reviewed_by = $2, notes = $3 WHERE id = $4 AND deleted_at IS NULL")
        .bind(&decision).bind(session.user_id).bind(&notes).bind(rid)
        .execute(db::get_db())
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "REVIEW_SUPPLY_REQUEST", Some("supply_requests"), Some(rid), None, Some(serde_json::json!({ "decision": decision }))).await;
    Ok(())
}

#[tauri::command]
pub async fn submit_anomaly_report(
    token: String,
    settlement_id: Option<String>,
    title: String,
    description: String,
    severity: String,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "settler_commander")?;

    let sid = settlement_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO anomaly_reports (title, description, severity, status, settlement_id, reported_by) VALUES ($1,$2,$3,'open',$4,$5) RETURNING id"
    )
    .bind(&title).bind(&description).bind(&severity).bind(sid).bind(session.user_id)
    .fetch_one(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "SUBMIT_ANOMALY_REPORT", Some("anomaly_reports"), Some(row.0), None, Some(serde_json::json!({ "title": title, "severity": severity }))).await;
    Ok(row.0.to_string())
}

#[tauri::command]
pub async fn get_anomaly_reports(token: String) -> Result<Vec<serde_json::Value>, String> {
    let _session = validate_session_command(&token).await?;

    let rows = sqlx::query_as::<_, AnomalyReportRow>(
        "SELECT id, title, description, severity, status, settlement_id, reported_by, created_at FROM anomaly_reports WHERE deleted_at IS NULL ORDER BY created_at DESC"
    )
    .fetch_all(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn review_anomaly_report(
    token: String,
    report_id: String,
    outcome: String,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    require_role(&session, 3)?;

    let rid = Uuid::parse_str(&report_id).map_err(|_| "Invalid report ID".to_string())?;

    sqlx::query("UPDATE anomaly_reports SET status = $1, reviewed_by = $2 WHERE id = $3 AND deleted_at IS NULL")
        .bind(&outcome).bind(session.user_id).bind(rid)
        .execute(db::get_db())
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "REVIEW_ANOMALY_REPORT", Some("anomaly_reports"), Some(rid), None, Some(serde_json::json!({ "outcome": outcome }))).await;
    Ok(())
}

#[tauri::command]
pub async fn issue_house_arrest(
    token: String,
    settler_id: String,
    settlement_id: Option<String>,
    reason: String,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "settler_commander")?;

    let settler = Uuid::parse_str(&settler_id).map_err(|_| "Invalid settler ID".to_string())?;
    let sid = settlement_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());
    let sdate = start_date.as_deref()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.with_timezone(&chrono::Utc));
    let edate = end_date.as_deref()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.with_timezone(&chrono::Utc));

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO house_arrests (settler_id, settlement_id, reason, start_date, end_date, issued_by) VALUES ($1,$2,$3,$4,$5,$6) RETURNING id"
    )
    .bind(settler).bind(sid).bind(&reason).bind(sdate).bind(edate).bind(session.user_id)
    .fetch_one(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "ISSUE_HOUSE_ARREST", Some("house_arrests"), Some(row.0), None, Some(serde_json::json!({ "settler_id": settler_id, "reason": reason }))).await;
    Ok(row.0.to_string())
}

#[tauri::command]
pub async fn submit_send_to_earth(
    token: String,
    settler_id: String,
    reason: String,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "settler_commander")?;

    let settler = Uuid::parse_str(&settler_id).map_err(|_| "Invalid settler ID".to_string())?;

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO send_to_earth_requests (settler_id, reason, status, requested_by) VALUES ($1,$2,'pending',$3) RETURNING id"
    )
    .bind(settler).bind(&reason).bind(session.user_id)
    .fetch_one(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "SUBMIT_SEND_TO_EARTH", Some("send_to_earth_requests"), Some(row.0), None, Some(serde_json::json!({ "settler_id": settler_id, "reason": reason }))).await;
    Ok(row.0.to_string())
}

#[tauri::command]
pub async fn review_send_to_earth(
    token: String,
    request_id: String,
    decision: String,
    notes: Option<String>,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    require_role(&session, 3)?;

    let rid = Uuid::parse_str(&request_id).map_err(|_| "Invalid request ID".to_string())?;

    sqlx::query("UPDATE send_to_earth_requests SET status = $1, reviewed_by = $2, notes = $3 WHERE id = $4 AND deleted_at IS NULL")
        .bind(&decision).bind(session.user_id).bind(&notes).bind(rid)
        .execute(db::get_db())
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "REVIEW_SEND_TO_EARTH", Some("send_to_earth_requests"), Some(rid), None, Some(serde_json::json!({ "decision": decision }))).await;
    Ok(())
}

#[tauri::command]
pub async fn log_settlement_inventory(
    token: String,
    settlement_id: String,
    item_name: String,
    category: Option<String>,
    quantity: i32,
    unit: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    let sid = Uuid::parse_str(&settlement_id).map_err(|_| "Invalid settlement ID".to_string())?;

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO settlement_inventory (settlement_id, item_name, category, quantity, unit, updated_by) VALUES ($1,$2,$3,$4,$5,$6) RETURNING id"
    )
    .bind(sid).bind(&item_name).bind(&category).bind(quantity).bind(&unit).bind(session.user_id)
    .fetch_one(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "LOG_SETTLEMENT_INVENTORY", Some("settlement_inventory"), Some(row.0), None, Some(serde_json::json!({ "item_name": item_name, "quantity": quantity }))).await;
    Ok(row.0.to_string())
}

#[tauri::command]
pub async fn get_settlement_inventory(
    token: String,
    settlement_id: String,
) -> Result<Vec<serde_json::Value>, String> {
    let _session = validate_session_command(&token).await?;
    let sid = Uuid::parse_str(&settlement_id).map_err(|_| "Invalid settlement ID".to_string())?;

    let rows = sqlx::query_as::<_, InventoryRow>(
        "SELECT id, settlement_id, item_name, category, quantity, unit, updated_at FROM settlement_inventory WHERE settlement_id = $1 AND deleted_at IS NULL ORDER BY item_name"
    )
    .bind(sid)
    .fetch_all(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn submit_farm_report(
    token: String,
    settlement_id: Option<String>,
    title: String,
    content: String,
    crop_status: Option<String>,
    health_check_notes: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    if session.role_name != "farmer" && session.role_name != "settler_commander" && !is_admin(&session) {
        return Err("Only farmers or commanders can submit farm reports".to_string());
    }

    let sid = settlement_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO farm_progress_reports (settlement_id, title, content, crop_status, health_check_notes, farmer_id) VALUES ($1,$2,$3,$4,$5,$6) RETURNING id"
    )
    .bind(sid).bind(&title).bind(&content).bind(&crop_status).bind(&health_check_notes).bind(session.user_id)
    .fetch_one(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "SUBMIT_FARM_REPORT", Some("farm_progress_reports"), Some(row.0), None, Some(serde_json::json!({ "title": title }))).await;
    Ok(row.0.to_string())
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct FarmReportRow {
    id: Uuid,
    settlement_id: Option<Uuid>,
    farmer_id: Option<Uuid>,
    title: String,
    content: Option<String>,
    crop_status: Option<String>,
    health_check_notes: Option<String>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[tauri::command]
pub async fn get_farm_reports(token: String) -> Result<Vec<serde_json::Value>, String> {
    let session = validate_session_command(&token).await?;

    let rows = if permissions::has_permission(&session.role_name, "farmer")
        || permissions::has_permission(&session.role_name, "settler_commander")
        || session.tier >= 3
    {
        sqlx::query_as::<_, FarmReportRow>(
            "SELECT id, settlement_id, farmer_id, title, content, crop_status, health_check_notes, created_at FROM farm_progress_reports WHERE deleted_at IS NULL ORDER BY created_at DESC"
        ).fetch_all(db::get_db()).await
    } else {
        sqlx::query_as::<_, FarmReportRow>(
            "SELECT id, settlement_id, farmer_id, title, content, crop_status, health_check_notes, created_at FROM farm_progress_reports WHERE farmer_id = $1 AND deleted_at IS NULL ORDER BY created_at DESC"
        ).bind(session.user_id).fetch_all(db::get_db()).await
    }.map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct TroublesomeSettlerReportRow {
    id: Uuid,
    reported_settler_id: Option<Uuid>,
    reported_by: Option<Uuid>,
    settlement_id: Option<Uuid>,
    description: Option<String>,
    status: Option<String>,
    house_arrest_id: Option<Uuid>,
    send_to_earth_id: Option<Uuid>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[tauri::command]
pub async fn submit_troublesome_settler_report(
    token: String,
    reported_settler_id: String,
    settlement_id: Option<String>,
    description: String,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;

    let settler = Uuid::parse_str(&reported_settler_id).map_err(|_| "Invalid settler ID".to_string())?;
    let sid = settlement_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO troublesome_settler_reports (reported_settler_id, reported_by, settlement_id, description, status) VALUES ($1,$2,$3,$4,'pending') RETURNING id"
    )
    .bind(settler).bind(session.user_id).bind(sid).bind(&description)
    .fetch_one(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "SUBMIT_TROUBLESOME_SETTLER_REPORT", Some("troublesome_settler_reports"), Some(row.0), None, Some(serde_json::json!({ "reported_settler_id": reported_settler_id }))).await;
    Ok(row.0.to_string())
}

#[tauri::command]
pub async fn get_troublesome_settler_reports(token: String) -> Result<Vec<serde_json::Value>, String> {
    let session = validate_session_command(&token).await?;

    let rows = if permissions::has_permission(&session.role_name, "settler_commander") || session.tier >= 3 {
        sqlx::query_as::<_, TroublesomeSettlerReportRow>(
            "SELECT id, reported_settler_id, reported_by, settlement_id, description, status, house_arrest_id, send_to_earth_id, created_at FROM troublesome_settler_reports WHERE deleted_at IS NULL ORDER BY created_at DESC"
        ).fetch_all(db::get_db()).await
    } else {
        sqlx::query_as::<_, TroublesomeSettlerReportRow>(
            "SELECT id, reported_settler_id, reported_by, settlement_id, description, status, house_arrest_id, send_to_earth_id, created_at FROM troublesome_settler_reports WHERE reported_by = $1 AND deleted_at IS NULL ORDER BY created_at DESC"
        ).bind(session.user_id).fetch_all(db::get_db()).await
    }.map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct CivilEngineerProgressReportRow {
    id: Uuid,
    task_id: Option<Uuid>,
    settlement_id: Option<Uuid>,
    title: String,
    content: Option<String>,
    materials_used: Option<serde_json::Value>,
    progress_percentage: Option<i32>,
    problems_encountered: Option<String>,
    plans_next_steps: Option<String>,
    submitted_by: Option<Uuid>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[tauri::command]
pub async fn submit_civil_engineer_report(
    token: String,
    settlement_id: Option<String>,
    task_id: Option<String>,
    title: String,
    content: String,
    materials_used: Option<String>,
    progress_percentage: Option<i32>,
    problems_encountered: Option<String>,
    plans_next_steps: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    if session.role_name != "civil_engineer" && session.role_name != "settler_commander" && !is_admin(&session) {
        return Err("Only civil engineers or the settler commander can submit civil engineer reports".to_string());
    }

    let sid = settlement_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());
    let tid = task_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());
    let materials: Option<serde_json::Value> = materials_used
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok());

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO civil_engineer_progress_reports (task_id, settlement_id, title, content, materials_used, progress_percentage, problems_encountered, plans_next_steps, submitted_by) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9) RETURNING id"
    )
    .bind(tid).bind(sid).bind(&title).bind(&content).bind(&materials).bind(progress_percentage).bind(&problems_encountered).bind(&plans_next_steps).bind(session.user_id)
    .fetch_one(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "SUBMIT_CIVIL_ENGINEER_REPORT", Some("civil_engineer_progress_reports"), Some(row.0), None, Some(serde_json::json!({ "title": title }))).await;
    Ok(row.0.to_string())
}

#[tauri::command]
pub async fn get_civil_engineer_reports(token: String) -> Result<Vec<serde_json::Value>, String> {
    let session = validate_session_command(&token).await?;

    let rows = if permissions::has_permission(&session.role_name, "civil_engineer")
        || permissions::has_permission(&session.role_name, "settler_commander")
        || session.tier >= 3
    {
        sqlx::query_as::<_, CivilEngineerProgressReportRow>(
            "SELECT id, task_id, settlement_id, title, content, materials_used, progress_percentage, problems_encountered, plans_next_steps, submitted_by, created_at FROM civil_engineer_progress_reports WHERE deleted_at IS NULL ORDER BY created_at DESC"
        ).fetch_all(db::get_db()).await
    } else {
        sqlx::query_as::<_, CivilEngineerProgressReportRow>(
            "SELECT id, task_id, settlement_id, title, content, materials_used, progress_percentage, problems_encountered, plans_next_steps, submitted_by, created_at FROM civil_engineer_progress_reports WHERE submitted_by = $1 AND deleted_at IS NULL ORDER BY created_at DESC"
        ).bind(session.user_id).fetch_all(db::get_db()).await
    }.map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}
