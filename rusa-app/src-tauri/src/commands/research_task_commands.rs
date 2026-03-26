use crate::auth::{is_admin, permissions, validate_session_command};
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
    due_date: Option<chrono::DateTime<chrono::Utc>>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
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

    let rows = if permissions::has_permission(&session.role_name, "the_observer")
        || permissions::has_permission(&session.role_name, "the_artificer")
        || permissions::has_permission(&session.role_name, "the_taskmaster")
    {
        sqlx::query_as::<_, ResearchTaskRow>(
            "SELECT id, title, description, assigned_to, assigned_by, source_message_id, status, result_notes, due_date, created_at FROM research_tasks WHERE assigned_by = $1 AND deleted_at IS NULL ORDER BY created_at DESC"
        )
        .bind(session.user_id)
        .fetch_all(db::get_db())
        .await
        .map_err(|e| format!("DB error: {}", e))?
    } else {
        sqlx::query_as::<_, ResearchTaskRow>(
            "SELECT id, title, description, assigned_to, assigned_by, source_message_id, status, result_notes, due_date, created_at FROM research_tasks WHERE assigned_to = $1 AND deleted_at IS NULL ORDER BY created_at DESC"
        )
        .bind(session.user_id)
        .fetch_all(db::get_db())
        .await
        .map_err(|e| format!("DB error: {}", e))?
    };

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
