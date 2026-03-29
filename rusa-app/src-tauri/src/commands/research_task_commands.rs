use crate::auth::{is_admin, validate_session_command};
use crate::db;
use crate::queries::auth::write_audit_log;
use uuid::Uuid;

#[derive(sqlx::FromRow, serde::Serialize)]
struct ResearchTaskRow {
    id: Uuid,
    title: String,
    description: Option<String>,
    assigned_to: Option<Uuid>,
    assigned_by: Option<Uuid>,
    source_message_id: Option<Uuid>,
    status: String,
    result_notes: Option<String>,
    progress_notes: Option<String>,
    due_date: Option<chrono::DateTime<chrono::Utc>>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
    final_notes: Option<String>,
    final_findings: Option<String>,
    methodology_summary: Option<String>,
    key_results: Option<String>,
    recommendations: Option<String>,
    limitations: Option<String>,
    conclusion_requested_at: Option<chrono::DateTime<chrono::Utc>>,
    conclusion_requested_by: Option<Uuid>,
    conclusion_approved_at: Option<chrono::DateTime<chrono::Utc>>,
    conclusion_approved_by: Option<Uuid>,
}

async fn ensure_research_tasks_table() -> Result<(), String> {
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS research_tasks (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            title TEXT NOT NULL,
            description TEXT,
            assigned_to UUID REFERENCES users(id),
            assigned_by UUID REFERENCES users(id),
            source_message_id UUID REFERENCES messages(id),
            status TEXT NOT NULL DEFAULT 'pending',
            result_notes TEXT,
            due_date TIMESTAMPTZ,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            deleted_at TIMESTAMPTZ
        )"#,
    )
    .execute(db::get_db())
    .await
    .map_err(|e| format!("DB error creating table: {}", e))?;

    // Add new columns if they don't exist (idempotent migrations)
    let alter_stmts = [
        "ALTER TABLE research_tasks ADD COLUMN IF NOT EXISTS progress_notes TEXT",
        "ALTER TABLE research_tasks ADD COLUMN IF NOT EXISTS final_notes TEXT",
        "ALTER TABLE research_tasks ADD COLUMN IF NOT EXISTS final_findings TEXT",
        "ALTER TABLE research_tasks ADD COLUMN IF NOT EXISTS methodology_summary TEXT",
        "ALTER TABLE research_tasks ADD COLUMN IF NOT EXISTS key_results TEXT",
        "ALTER TABLE research_tasks ADD COLUMN IF NOT EXISTS recommendations TEXT",
        "ALTER TABLE research_tasks ADD COLUMN IF NOT EXISTS limitations TEXT",
        "ALTER TABLE research_tasks ADD COLUMN IF NOT EXISTS conclusion_requested_at TIMESTAMPTZ",
        "ALTER TABLE research_tasks ADD COLUMN IF NOT EXISTS conclusion_requested_by UUID REFERENCES users(id)",
        "ALTER TABLE research_tasks ADD COLUMN IF NOT EXISTS conclusion_approved_at TIMESTAMPTZ",
        "ALTER TABLE research_tasks ADD COLUMN IF NOT EXISTS conclusion_approved_by UUID REFERENCES users(id)",
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
pub async fn assign_research_task(
    token: String,
    title: String,
    description: Option<String>,
    assigned_to: Option<String>,
    source_message_id: Option<String>,
    due_date: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    if session.role_name != "the_observer" && session.role_name != "the_artificer" && session.role_name != "the_taskmaster" && !is_admin(&session) {
        return Err("Only the_observer, the_artificer, or the_taskmaster can assign research tasks".to_string());
    }

    ensure_research_tasks_table().await?;

    let aid = assigned_to
        .as_deref()
        .and_then(|s| Uuid::parse_str(s).ok());
    let smid = source_message_id
        .as_deref()
        .and_then(|s| Uuid::parse_str(s).ok());
    let dd = due_date
        .as_deref()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.with_timezone(&chrono::Utc));

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO research_tasks (title, description, assigned_to, assigned_by, source_message_id, status, due_date) VALUES ($1,$2,$3,$4,$5,'pending',$6) RETURNING id"
    )
    .bind(&title)
    .bind(&description)
    .bind(aid)
    .bind(session.user_id)
    .bind(smid)
    .bind(dd)
    .fetch_one(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(
        Some(session.user_id),
        "ASSIGN_RESEARCH_TASK",
        Some("research_tasks"),
        Some(row.0),
        None,
        Some(serde_json::json!({ "title": title, "assigned_to": assigned_to })),
    )
    .await;

    Ok(row.0.to_string())
}

#[tauri::command]
pub async fn get_research_tasks(token: String) -> Result<Vec<serde_json::Value>, String> {
    let session = validate_session_command(&token).await?;

    ensure_research_tasks_table().await?;

    let rows = sqlx::query_as::<_, ResearchTaskRow>(
        "SELECT id, title, description, assigned_to, assigned_by, source_message_id, status, \
         result_notes, progress_notes, due_date, created_at, \
         final_notes, final_findings, methodology_summary, key_results, recommendations, limitations, \
         conclusion_requested_at, conclusion_requested_by, conclusion_approved_at, conclusion_approved_by \
         FROM research_tasks WHERE (assigned_by = $1 OR assigned_to = $1) AND deleted_at IS NULL ORDER BY created_at DESC"
    )
    .bind(session.user_id)
    .fetch_all(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows
        .into_iter()
        .map(|r| serde_json::to_value(r).unwrap_or_default())
        .collect())
}

#[tauri::command]
pub async fn submit_research_task_result(
    token: String,
    task_id: String,
    result_notes: String,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;

    ensure_research_tasks_table().await?;

    let tid = Uuid::parse_str(&task_id).map_err(|_| "Invalid task ID".to_string())?;

    let task: Option<(Uuid,)> = sqlx::query_as(
        "SELECT assigned_to FROM research_tasks WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(tid)
    .fetch_optional(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    match task {
        None => return Err("Task not found".to_string()),
        Some((assigned_to,)) => {
            if assigned_to != session.user_id {
                return Err("You are not the assigned user for this task".to_string());
            }
        }
    }

    sqlx::query(
        "UPDATE research_tasks SET status = 'result_submitted', result_notes = $1 WHERE id = $2 AND deleted_at IS NULL"
    )
    .bind(&result_notes)
    .bind(tid)
    .execute(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(
        Some(session.user_id),
        "SUBMIT_RESEARCH_TASK_RESULT",
        Some("research_tasks"),
        Some(tid),
        None,
        None,
    )
    .await;

    Ok(())
}

#[tauri::command]
pub async fn complete_research_task(token: String, task_id: String) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    if session.role_name != "the_observer" && session.role_name != "the_artificer" && session.role_name != "the_taskmaster" && !is_admin(&session) {
        return Err(
            "Only the_observer, the_artificer, or the_taskmaster can mark research tasks as completed".to_string(),
        );
    }

    ensure_research_tasks_table().await?;

    let tid = Uuid::parse_str(&task_id).map_err(|_| "Invalid task ID".to_string())?;

    let task: Option<(Uuid,)> = sqlx::query_as(
        "SELECT assigned_by FROM research_tasks WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(tid)
    .fetch_optional(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    match task {
        None => return Err("Task not found".to_string()),
        Some((assigned_by,)) => {
            if assigned_by != session.user_id {
                return Err("You can only complete tasks that you assigned".to_string());
            }
        }
    }

    sqlx::query(
        "UPDATE research_tasks SET status = 'completed' WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(tid)
    .execute(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(
        Some(session.user_id),
        "COMPLETE_RESEARCH_TASK",
        Some("research_tasks"),
        Some(tid),
        None,
        None,
    )
    .await;

    Ok(())
}

/// Update the status and progress notes of a research task.
/// The assignee may update status and progress notes.
/// Directors (the_artificer, the_observer, the_taskmaster) may also update any task.
#[tauri::command]
pub async fn update_research_task_status(
    token: String,
    task_id: String,
    status: String,
    progress_notes: Option<String>,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;

    let allowed_statuses = ["pending", "in_progress", "completed", "cancelled"];
    if !allowed_statuses.contains(&status.as_str()) {
        return Err(format!("Invalid status '{}'. Allowed: pending, in_progress, completed, cancelled", status));
    }

    ensure_research_tasks_table().await?;

    let tid = Uuid::parse_str(&task_id).map_err(|_| "Invalid task ID".to_string())?;

    let is_director = session.role_name == "the_observer"
        || session.role_name == "the_artificer"
        || session.role_name == "the_taskmaster"
        || is_admin(&session);

    let task: Option<(Option<Uuid>, Option<Uuid>)> = sqlx::query_as(
        "SELECT assigned_to, assigned_by FROM research_tasks WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(tid)
    .fetch_optional(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    match task {
        None => return Err("Task not found".to_string()),
        Some((assigned_to, assigned_by)) => {
            if !is_director {
                let is_assignee = assigned_to.map(|id| id == session.user_id).unwrap_or(false);
                let is_assigner = assigned_by.map(|id| id == session.user_id).unwrap_or(false);
                if !is_assignee && !is_assigner {
                    return Err("You can only update tasks assigned to or by you".to_string());
                }
            }
        }
    }

    sqlx::query(
        "UPDATE research_tasks SET status = $1, progress_notes = COALESCE($2, progress_notes) \
         WHERE id = $3 AND deleted_at IS NULL",
    )
    .bind(&status)
    .bind(&progress_notes)
    .bind(tid)
    .execute(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(
        Some(session.user_id),
        "UPDATE_RESEARCH_TASK_STATUS",
        Some("research_tasks"),
        Some(tid),
        None,
        Some(serde_json::json!({ "status": status })),
    )
    .await;

    Ok(())
}

/// Mathematician requests a task conclusion with detailed findings.
/// Routes directly to the director who assigned the task (the_artificer).
#[tauri::command]
pub async fn request_research_task_conclusion(
    token: String,
    task_id: String,
    final_notes: String,
    final_findings: Option<String>,
    methodology_summary: Option<String>,
    key_results: Option<String>,
    recommendations: Option<String>,
    limitations: Option<String>,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;

    if final_notes.trim().is_empty() {
        return Err("Final notes are required".to_string());
    }

    ensure_research_tasks_table().await?;

    let tid = Uuid::parse_str(&task_id).map_err(|_| "Invalid task ID".to_string())?;

    let row: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM research_tasks WHERE id = $1 AND assigned_to = $2 AND deleted_at IS NULL",
    )
    .bind(tid)
    .bind(session.user_id)
    .fetch_optional(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    if row.is_none() {
        return Err("Task not found or not assigned to you".to_string());
    }

    sqlx::query(
        "UPDATE research_tasks SET status = 'conclusion_requested', \
         conclusion_requested_at = NOW(), conclusion_requested_by = $1, \
         final_notes = $2, final_findings = $3, methodology_summary = $4, \
         key_results = $5, recommendations = $6, limitations = $7 \
         WHERE id = $8 AND deleted_at IS NULL",
    )
    .bind(session.user_id)
    .bind(&final_notes)
    .bind(&final_findings)
    .bind(&methodology_summary)
    .bind(&key_results)
    .bind(&recommendations)
    .bind(&limitations)
    .bind(tid)
    .execute(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(
        Some(session.user_id),
        "REQUEST_RESEARCH_TASK_CONCLUSION",
        Some("research_tasks"),
        Some(tid),
        None,
        Some(serde_json::json!({ "task_id": task_id })),
    )
    .await;

    Ok(())
}

/// Director reviews (approves or rejects) a research task conclusion request.
/// Only the_artificer, the_observer, the_taskmaster, or admins may review.
#[tauri::command]
pub async fn review_research_task_conclusion(
    token: String,
    task_id: String,
    decision: String,
    review_notes: Option<String>,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;

    let is_director = session.role_name == "the_observer"
        || session.role_name == "the_artificer"
        || session.role_name == "the_taskmaster"
        || is_admin(&session);
    if !is_director {
        return Err("Only directors can review task conclusions".to_string());
    }

    if decision != "approve" && decision != "reject" {
        return Err("Decision must be 'approve' or 'reject'".to_string());
    }

    ensure_research_tasks_table().await?;

    let tid = Uuid::parse_str(&task_id).map_err(|_| "Invalid task ID".to_string())?;

    let row: Option<(String,)> = sqlx::query_as(
        "SELECT status FROM research_tasks WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(tid)
    .fetch_optional(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    match &row {
        None => return Err("Task not found".to_string()),
        Some((status,)) if status != "conclusion_requested" => {
            return Err(format!("Task is not in 'conclusion_requested' state (current: {})", status));
        }
        _ => {}
    }

    match decision.as_str() {
        "approve" => {
            sqlx::query(
                "UPDATE research_tasks SET status = 'completed', \
                 conclusion_approved_at = NOW(), conclusion_approved_by = $1, \
                 progress_notes = COALESCE($2, progress_notes) \
                 WHERE id = $3 AND deleted_at IS NULL",
            )
            .bind(session.user_id)
            .bind(&review_notes)
            .bind(tid)
            .execute(db::get_db())
            .await
            .map_err(|e| format!("DB error: {}", e))?;
        }
        "reject" => {
            sqlx::query(
                "UPDATE research_tasks SET status = 'in_progress', \
                 conclusion_requested_at = NULL, conclusion_requested_by = NULL, \
                 progress_notes = COALESCE($1, progress_notes) \
                 WHERE id = $2 AND deleted_at IS NULL",
            )
            .bind(&review_notes)
            .bind(tid)
            .execute(db::get_db())
            .await
            .map_err(|e| format!("DB error: {}", e))?;
        }
        _ => unreachable!(),
    }

    let _ = write_audit_log(
        Some(session.user_id),
        "REVIEW_RESEARCH_TASK_CONCLUSION",
        Some("research_tasks"),
        Some(tid),
        None,
        Some(serde_json::json!({ "decision": decision })),
    )
    .await;

    Ok(())
}
