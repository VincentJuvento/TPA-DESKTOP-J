use crate::auth::{is_admin, require_role, require_role_name, validate_session_command};
use crate::db;
use crate::queries::auth::write_audit_log;
use crate::queries::aerospace as aerospace_queries;
use crate::queries::governance as governance_queries;
use uuid::Uuid;

#[derive(sqlx::FromRow, serde::Serialize)]
struct AerospaceTaskRow {
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
    final_findings: Option<String>,
    methodology_summary: Option<String>,
    key_results: Option<String>,
    recommendations: Option<String>,
    limitations: Option<String>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct ShipRow {
    id: Uuid,
    name: String,
    ship_name: Option<String>,
    model: Option<String>,
    capacity: Option<i32>,
    ship_type: Option<String>,
    status: Option<String>,
    build_status: Option<String>,
    blueprint_approver: Option<Uuid>,
    launch_date: Option<chrono::DateTime<chrono::Utc>>,
    materials_used: Option<String>,
    last_updated: Option<chrono::DateTime<chrono::Utc>>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct HelpRequestRow {
    id: Uuid,
    requested_by: Option<Uuid>,
    title: String,
    description: Option<String>,
    category: Option<String>,
    assigned_proxy_director: String,
    status: String,
    response: Option<String>,
    resolved_by: Option<Uuid>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
    rejection_reason: Option<String>,
    rejected_at: Option<chrono::DateTime<chrono::Utc>>,
    created_task_id: Option<Uuid>,
}

#[derive(sqlx::FromRow)]
struct LinkedTaskInfo {
    id: Uuid,
    title: String,
    assigned_to: Option<Uuid>,
    status: Option<String>,
}

async fn ensure_aerospace_tasks_table() -> Result<(), String> {
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS aerospace_assigned_tasks (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            title TEXT NOT NULL,
            description TEXT,
            assigned_to UUID REFERENCES users(id),
            assigned_by UUID REFERENCES users(id),
            status TEXT NOT NULL DEFAULT 'pending',
            progress_notes TEXT,
            due_date TIMESTAMPTZ,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            deleted_at TIMESTAMPTZ,
            conclusion_requested_at TIMESTAMPTZ,
            conclusion_requested_by UUID REFERENCES users(id),
            conclusion_approved_at TIMESTAMPTZ,
            conclusion_approved_by UUID REFERENCES users(id),
            final_notes TEXT,
            final_findings TEXT,
            methodology_summary TEXT,
            key_results TEXT,
            recommendations TEXT,
            limitations TEXT
        )"#,
    )
    .execute(db::get_db())
    .await
    .map_err(|e| format!("DB error creating table: {}", e))?;
    sqlx::query("ALTER TABLE aerospace_assigned_tasks ADD COLUMN IF NOT EXISTS activity_logs JSONB DEFAULT '[]'")
        .execute(db::get_db())
        .await
        .map_err(|e| format!("DB error adding activity_logs: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn assign_aerospace_task(
    token: String,
    assigned_to: String,
    title: String,
    description: Option<String>,
    due_date: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    require_role(&session, 2)?;

    ensure_aerospace_tasks_table().await?;

    let atid = Uuid::parse_str(&assigned_to).map_err(|_| "Invalid user ID".to_string())?;
    let dd = due_date
        .as_deref()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.with_timezone(&chrono::Utc));

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO aerospace_assigned_tasks (title, description, assigned_to, assigned_by, status, due_date) VALUES ($1,$2,$3,$4,'pending',$5) RETURNING id",
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
        "ASSIGN_AEROSPACE_TASK",
        Some("aerospace_assigned_tasks"),
        Some(row.0),
        None,
        Some(serde_json::json!({ "title": title, "assigned_to": assigned_to })),
    )
    .await;

    Ok(row.0.to_string())
}

#[tauri::command]
pub async fn get_aerospace_assigned_tasks(token: String) -> Result<Vec<serde_json::Value>, String> {
    let session = validate_session_command(&token).await?;

    ensure_aerospace_tasks_table().await?;

    let rows = sqlx::query_as::<_, AerospaceTaskRow>(
        "SELECT id, title, description, assigned_to, assigned_by, status, progress_notes, activity_logs, due_date, created_at, conclusion_requested_at, conclusion_requested_by, conclusion_approved_at, conclusion_approved_by, final_notes, final_findings, methodology_summary, key_results, recommendations, limitations FROM aerospace_assigned_tasks WHERE (assigned_by = $1 OR assigned_to = $1) AND deleted_at IS NULL ORDER BY created_at DESC",
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
pub async fn update_aerospace_task_status(
    token: String,
    task_id: String,
    status: String,
    progress_notes: Option<String>,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;

    ensure_aerospace_tasks_table().await?;

    let tid = Uuid::parse_str(&task_id).map_err(|_| "Invalid task ID".to_string())?;

    let task: Option<(Option<Uuid>, Option<Uuid>, Option<String>)> = sqlx::query_as(
        "SELECT assigned_to, assigned_by, status FROM aerospace_assigned_tasks WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(tid)
    .fetch_optional(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let (assigned_to, assigned_by, current_status) = task.ok_or_else(|| "Task not found".to_string())?;
    let is_assigner = assigned_by.map(|id| id == session.user_id).unwrap_or(false);
    let is_assignee = assigned_to.map(|id| id == session.user_id).unwrap_or(false);
    let is_head = session.tier >= 3 || session.role_name == "the_taskmaster";

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
            progress_notes.as_deref().map(|n| format!(": {}", n)).unwrap_or_default()),
        "log_type": "status_change"
    }]);

    sqlx::query(
        "UPDATE aerospace_assigned_tasks SET status = $1, progress_notes = COALESCE($2, progress_notes), \
         activity_logs = COALESCE(activity_logs, '[]'::jsonb) || $3 WHERE id = $4 AND deleted_at IS NULL",
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
        "UPDATE_AEROSPACE_TASK_STATUS",
        Some("aerospace_assigned_tasks"),
        Some(tid),
        None,
        Some(serde_json::json!({ "status": status })),
    )
    .await;

    Ok(())
}

#[tauri::command]
pub async fn get_aerospace_task(token: String, task_id: String) -> Result<serde_json::Value, String> {
    let session = validate_session_command(&token).await?;
    ensure_aerospace_tasks_table().await?;
    let tid = Uuid::parse_str(&task_id).map_err(|_| "Invalid task ID".to_string())?;

    let row = sqlx::query_as::<_, AerospaceTaskRow>(
        "SELECT id, title, description, assigned_to, assigned_by, status, progress_notes, activity_logs, due_date, created_at, conclusion_requested_at, conclusion_requested_by, conclusion_approved_at, conclusion_approved_by, final_notes, final_findings, methodology_summary, key_results, recommendations, limitations \
         FROM aerospace_assigned_tasks WHERE id = $1 AND (assigned_to = $2 OR assigned_by = $2 OR $3 >= 3 OR $4 = 'the_taskmaster') AND deleted_at IS NULL",
    )
    .bind(tid)
    .bind(session.user_id)
    .bind(session.tier)
    .bind(&session.role_name)
    .fetch_optional(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    row.map(|r| serde_json::to_value(r).unwrap_or_default())
        .ok_or_else(|| "Task not found".to_string())
}

#[tauri::command]
pub async fn append_aerospace_task_activity_log(
    token: String,
    task_id: String,
    content: String,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    ensure_aerospace_tasks_table().await?;
    let tid = Uuid::parse_str(&task_id).map_err(|_| "Invalid task ID".to_string())?;

    let task: Option<(Option<String>,)> = sqlx::query_as(
        "SELECT status FROM aerospace_assigned_tasks WHERE id = $1 AND (assigned_to = $2 OR assigned_by = $2 OR $3 >= 3 OR $4 = 'the_taskmaster') AND deleted_at IS NULL",
    )
    .bind(tid)
    .bind(session.user_id)
    .bind(session.tier)
    .bind(&session.role_name)
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
        "UPDATE aerospace_assigned_tasks SET activity_logs = COALESCE(activity_logs, '[]'::jsonb) || $1 WHERE id = $2 AND deleted_at IS NULL",
    )
    .bind(&entry)
    .bind(tid)
    .execute(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "APPEND_AEROSPACE_TASK_LOG", Some("aerospace_assigned_tasks"), Some(tid), None, None).await;
    Ok(())
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct WorkOrderRow {
    id: Uuid,
    title: String,
    description: Option<String>,
    priority: Option<String>,
    system_affected: Option<String>,
    status: Option<String>,
    assigned_to: Option<Uuid>,
    notes: Option<String>,
    created_by: Option<Uuid>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct TechnicalReportRow {
    id: Uuid,
    title: String,
    content: Option<String>,
    findings: Option<String>,
    recommendations: Option<String>,
    submitted_by: Option<Uuid>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[tauri::command]
pub async fn get_work_orders(token: String) -> Result<Vec<serde_json::Value>, String> {
    let _session = validate_session_command(&token).await?;
    let rows = sqlx::query_as::<_, WorkOrderRow>(
        "SELECT id, title, description, priority, system_affected, status, assigned_to, notes, created_by, created_at FROM aerospace_work_orders WHERE deleted_at IS NULL ORDER BY created_at DESC"
    ).fetch_all(db::get_db()).await.map_err(|e| format!("DB error: {}", e))?;
    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn create_work_order(
    token: String,
    title: String,
    description: Option<String>,
    priority: Option<String>,
    system_affected: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "aerospace_engineer")?;

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO aerospace_work_orders (title, description, priority, system_affected, status, created_by) VALUES ($1,$2,$3,$4,'open',$5) RETURNING id"
    )
    .bind(&title).bind(&description).bind(&priority).bind(&system_affected).bind(session.user_id)
    .fetch_one(db::get_db()).await.map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "CREATE_WORK_ORDER", Some("aerospace_work_orders"), Some(row.0), None, Some(serde_json::json!({ "title": title }))).await;
    Ok(row.0.to_string())
}

#[tauri::command]
pub async fn update_work_order_status(
    token: String,
    work_order_id: String,
    status: String,
    notes: Option<String>,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "aerospace_engineer")?;

    let wid = Uuid::parse_str(&work_order_id).map_err(|_| "Invalid ID".to_string())?;
    sqlx::query("UPDATE aerospace_work_orders SET status = $1, notes = $2 WHERE id = $3 AND deleted_at IS NULL")
        .bind(&status).bind(&notes).bind(wid)
        .execute(db::get_db()).await.map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "UPDATE_WORK_ORDER_STATUS", Some("aerospace_work_orders"), Some(wid), None, Some(serde_json::json!({ "status": status }))).await;
    Ok(())
}

#[tauri::command]
pub async fn submit_technical_report(
    token: String,
    title: String,
    content: String,
    findings: Option<String>,
    recommendations: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "aerospace_engineer")?;

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO aerospace_technical_reports (title, content, findings, recommendations, submitted_by) VALUES ($1,$2,$3,$4,$5) RETURNING id"
    )
    .bind(&title).bind(&content).bind(&findings).bind(&recommendations).bind(session.user_id)
    .fetch_one(db::get_db()).await.map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "SUBMIT_TECHNICAL_REPORT", Some("aerospace_technical_reports"), Some(row.0), None, Some(serde_json::json!({ "title": title }))).await;
    Ok(row.0.to_string())
}

#[tauri::command]
pub async fn get_technical_reports(token: String) -> Result<Vec<serde_json::Value>, String> {
    let _session = validate_session_command(&token).await?;
    let rows = sqlx::query_as::<_, TechnicalReportRow>(
        "SELECT id, title, content, findings, recommendations, submitted_by, created_at FROM aerospace_technical_reports WHERE deleted_at IS NULL ORDER BY created_at DESC"
    ).fetch_all(db::get_db()).await.map_err(|e| format!("DB error: {}", e))?;
    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

/// Aerospace engineer submits a blueprint proposal for a ship design.
/// A directors' vote is automatically initiated on submission.
#[tauri::command]
pub async fn submit_blueprint_proposal(
    token: String,
    ship_name: String,
    blueprint_description: String,
    design_specs: Option<String>,
    ship_id: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "aerospace_engineer")?;

    if ship_name.trim().is_empty() {
        return Err("Ship name is required".to_string());
    }
    if blueprint_description.trim().is_empty() {
        return Err("Blueprint description is required".to_string());
    }

    aerospace_queries::ensure_blueprint_proposals_table()
        .await
        .map_err(|e| format!("DB error creating table: {}", e))?;

    let sid = ship_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());

    let proposal_id = aerospace_queries::insert_blueprint_proposal(
        &ship_name,
        &blueprint_description,
        design_specs.as_deref(),
        sid,
        session.user_id,
    )
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    // Auto-initiate a directors' vote for this blueprint proposal
    let vote_title = format!("Blueprint Proposal: {}", ship_name);
    let vote_desc = format!(
        "Directors' vote required for blueprint proposal '{}' submitted by {}. \
         Description: {}",
        ship_name, session.full_name, blueprint_description
    );

    let vote_id = governance_queries::insert_vote_typed(
        &vote_title,
        Some(&vote_desc),
        session.user_id,
        "blueprint",
    )
    .await
    .map_err(|e| format!("DB error creating vote: {}", e))?;

    aerospace_queries::link_blueprint_proposal_to_vote(proposal_id, vote_id)
        .await
        .map_err(|e| format!("DB error linking vote: {}", e))?;

    let _ = write_audit_log(
        Some(session.user_id),
        "SUBMIT_BLUEPRINT_PROPOSAL",
        Some("blueprint_proposals"),
        Some(proposal_id),
        None,
        Some(serde_json::json!({ "ship_name": ship_name, "vote_id": vote_id.to_string() })),
    )
    .await;

    Ok(proposal_id.to_string())
}

/// Fetch blueprint proposals. Directors see all; engineers see their own.
#[tauri::command]
pub async fn get_blueprint_proposals(token: String) -> Result<Vec<serde_json::Value>, String> {
    let session = validate_session_command(&token).await?;

    aerospace_queries::ensure_blueprint_proposals_table()
        .await
        .map_err(|e| format!("DB error creating table: {}", e))?;

    let rows = if session.tier >= 3 {
        aerospace_queries::get_all_blueprint_proposals().await
    } else {
        aerospace_queries::get_user_blueprint_proposals(session.user_id).await
    }
    .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows
        .into_iter()
        .map(|r| serde_json::to_value(r).unwrap_or_default())
        .collect())
}

/// Directors approve or reject a blueprint proposal after the linked vote has passed.
#[tauri::command]
pub async fn review_blueprint_proposal(
    token: String,
    proposal_id: String,
    status: String,
    notes: Option<String>,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    require_role(&session, 3)?;

    if status != "approved" && status != "rejected" {
        return Err("Status must be 'approved' or 'rejected'".to_string());
    }

    aerospace_queries::ensure_blueprint_proposals_table()
        .await
        .map_err(|e| format!("DB error creating table: {}", e))?;

    let pid = Uuid::parse_str(&proposal_id).map_err(|_| "Invalid proposal ID".to_string())?;

    // Only allow approval after the linked vote has passed
    if status == "approved" {
        let vote_status = aerospace_queries::get_blueprint_vote_status(pid)
            .await
            .map_err(|e| format!("DB error: {}", e))?;

        match vote_status.as_deref() {
            Some("passed") => {}
            Some(vs) => {
                return Err(format!(
                    "Cannot approve: linked vote has not passed (current vote status: {}). \
                     Wait for the directors' vote to conclude.",
                    vs
                ))
            }
            None => {
                return Err(
                    "Cannot approve: no vote is linked to this proposal or the vote does not exist."
                        .to_string(),
                )
            }
        }
    }

    aerospace_queries::update_blueprint_proposal_review(
        pid,
        &status,
        session.user_id,
        notes.as_deref(),
    )
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(
        Some(session.user_id),
        "REVIEW_BLUEPRINT_PROPOSAL",
        Some("blueprint_proposals"),
        Some(pid),
        None,
        Some(serde_json::json!({ "status": status })),
    )
    .await;

    Ok(())
}

/// Update a ship's operational status.
/// Moving a ship to 'building' requires an approved blueprint proposal linked to that ship.
#[tauri::command]
pub async fn update_ship_status(
    token: String,
    ship_id: String,
    status: String,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    // Directors (tier >= 3) or aerospace engineers can update ship status
    if session.tier < 3 && session.role_name != "aerospace_engineer" {
        return Err("Only directors or aerospace engineers can update ship status".to_string());
    }

    let sid = Uuid::parse_str(&ship_id).map_err(|_| "Invalid ship ID".to_string())?;

    // Enforce blueprint approval gate before 'building' phase
    if status == "building" {
        aerospace_queries::ensure_blueprint_proposals_table()
            .await
            .map_err(|e| format!("DB error: {}", e))?;

        let has_blueprint = aerospace_queries::ship_has_approved_blueprint(sid)
            .await
            .map_err(|e| format!("DB error: {}", e))?;

        if !has_blueprint {
            return Err(
                "Cannot move ship to 'building' phase: no approved blueprint proposal found \
                 for this ship. Submit a blueprint proposal and obtain director approval first."
                    .to_string(),
            );
        }
    }

    sqlx::query(
        "UPDATE ships SET status = $1 WHERE id = $2 AND deleted_at IS NULL",
    )
    .bind(&status)
    .bind(sid)
    .execute(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(
        Some(session.user_id),
        "UPDATE_SHIP_STATUS",
        Some("ships"),
        Some(sid),
        None,
        Some(serde_json::json!({ "status": status })),
    )
    .await;

    Ok(())
}

// ─── Aerospace Task Conclusion Flow ─────────────────────────────────────────

/// Aerospace engineer requests conclusion of an assigned task.
/// Bypasses voting; goes directly to the director who assigned the task.
#[tauri::command]
pub async fn request_aerospace_task_conclusion(
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
    require_role_name(&session, "aerospace_engineer")?;
    ensure_aerospace_tasks_table().await?;

    let tid = Uuid::parse_str(&task_id).map_err(|_| "Invalid task ID".to_string())?;

    if final_notes.trim().is_empty() {
        return Err("Final notes are required".to_string());
    }

    // Verify this task is assigned to the requesting engineer and not already concluded
    let row: Option<(Uuid, Option<String>)> = sqlx::query_as(
        "SELECT id, status FROM aerospace_assigned_tasks WHERE id = $1 AND assigned_to = $2 AND deleted_at IS NULL",
    )
    .bind(tid)
    .bind(session.user_id)
    .fetch_optional(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    match row {
        None => return Err("Task not found or not assigned to you".to_string()),
        Some((_id, status)) => {
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
        "content": final_notes,
        "log_type": "conclusion_requested"
    }]);

    sqlx::query(
        "UPDATE aerospace_assigned_tasks SET status = 'conclusion_requested', \
         conclusion_requested_at = NOW(), conclusion_requested_by = $1, \
         final_notes = $2, final_findings = $3, methodology_summary = $4, \
         key_results = $5, recommendations = $6, limitations = $7, \
         activity_logs = COALESCE(activity_logs, '[]'::jsonb) || $8 \
         WHERE id = $9 AND deleted_at IS NULL",
    )
    .bind(session.user_id)
    .bind(&final_notes)
    .bind(&final_findings)
    .bind(&methodology_summary)
    .bind(&key_results)
    .bind(&recommendations)
    .bind(&limitations)
    .bind(&log_entry)
    .bind(tid)
    .execute(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(
        Some(session.user_id),
        "REQUEST_AEROSPACE_TASK_CONCLUSION",
        Some("aerospace_assigned_tasks"),
        Some(tid),
        None,
        Some(serde_json::json!({ "task_id": task_id })),
    )
    .await;

    Ok(())
}

/// Director/Taskmaster approves or rejects an aerospace task conclusion request.
/// Does NOT go through the voting system — direct approval by the assigning director.
#[tauri::command]
pub async fn approve_aerospace_task_conclusion(
    token: String,
    task_id: String,
    decision: String,
    review_notes: Option<String>,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    ensure_aerospace_tasks_table().await?;
    // Requires director tier or the_taskmaster role
    if session.tier < 3 && session.role_name != "the_taskmaster" {
        return Err("Only directors (tier 3+) or The Taskmaster can approve task conclusions".to_string());
    }

    if decision != "approve" && decision != "reject" {
        return Err("Decision must be 'approve' or 'reject'".to_string());
    }

    let tid = Uuid::parse_str(&task_id).map_err(|_| "Invalid task ID".to_string())?;

    // Verify the task has a pending conclusion request
    let row: Option<(String, Option<Uuid>)> = sqlx::query_as(
        "SELECT status, assigned_by FROM aerospace_assigned_tasks WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(tid)
    .fetch_optional(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    match &row {
        None => return Err("Task not found".to_string()),
        Some((status, _)) if status != "conclusion_requested" => {
            return Err(format!("Task is not in 'conclusion_requested' state (current: {})", status))
        }
        _ => {}
    }

    match decision.as_str() {
        "approve" => {
            let log_entry = serde_json::json!([{
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "author_id": session.user_id.to_string(),
                "author_name": session.full_name,
                "content": review_notes.as_deref().unwrap_or(""),
                "log_type": "conclusion_approved"
            }]);
            sqlx::query(
                "UPDATE aerospace_assigned_tasks SET status = 'completed', \
                 conclusion_approved_at = NOW(), conclusion_approved_by = $1, \
                 progress_notes = COALESCE($2, progress_notes), \
                 activity_logs = COALESCE(activity_logs, '[]'::jsonb) || $3 \
                 WHERE id = $4 AND deleted_at IS NULL",
            )
            .bind(session.user_id)
            .bind(&review_notes)
            .bind(&log_entry)
            .bind(tid)
            .execute(db::get_db())
            .await
            .map_err(|e| format!("DB error: {}", e))?;
        }
        "reject" => {
            let log_entry = serde_json::json!([{
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "author_id": session.user_id.to_string(),
                "author_name": session.full_name,
                "content": review_notes.as_deref().unwrap_or(""),
                "log_type": "conclusion_rejected"
            }]);
            sqlx::query(
                "UPDATE aerospace_assigned_tasks SET status = 'in_progress', \
                 conclusion_requested_at = NULL, conclusion_requested_by = NULL, \
                 progress_notes = COALESCE($1, progress_notes), \
                 activity_logs = COALESCE(activity_logs, '[]'::jsonb) || $2 \
                 WHERE id = $3 AND deleted_at IS NULL",
            )
            .bind(&review_notes)
            .bind(&log_entry)
            .bind(tid)
            .execute(db::get_db())
            .await
            .map_err(|e| format!("DB error: {}", e))?;
        }
        _ => unreachable!(),
    }

    let _ = write_audit_log(
        Some(session.user_id),
        "APPROVE_AEROSPACE_TASK_CONCLUSION",
        Some("aerospace_assigned_tasks"),
        Some(tid),
        None,
        Some(serde_json::json!({ "decision": decision })),
    )
    .await;

    Ok(())
}

// ─── Ships Archive ───────────────────────────────────────────────────────────

/// List all ships with full details. Available to all authenticated users.
#[tauri::command]
pub async fn get_all_ships(token: String) -> Result<Vec<serde_json::Value>, String> {
    let _session = validate_session_command(&token).await?;

    let rows = sqlx::query_as::<_, ShipRow>(
        "SELECT id, name, ship_name, model, capacity, ship_type, status, build_status, \
         blueprint_approver, launch_date, materials_used, last_updated, created_at \
         FROM ships WHERE deleted_at IS NULL ORDER BY created_at DESC",
    )
    .fetch_all(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

/// Retrieve a single ship with its related blueprint proposals and approvals.
#[tauri::command]
pub async fn get_ship_details(token: String, ship_id: String) -> Result<serde_json::Value, String> {
    let _session = validate_session_command(&token).await?;

    let sid = Uuid::parse_str(&ship_id).map_err(|_| "Invalid ship ID".to_string())?;

    let ship = sqlx::query_as::<_, ShipRow>(
        "SELECT id, name, ship_name, model, capacity, ship_type, status, build_status, \
         blueprint_approver, launch_date, materials_used, last_updated, created_at \
         FROM ships WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(sid)
    .fetch_optional(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?
    .ok_or_else(|| "Ship not found".to_string())?;

    // Fetch related blueprint proposals
    aerospace_queries::ensure_blueprint_proposals_table()
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let blueprints = aerospace_queries::get_all_blueprint_proposals()
        .await
        .map_err(|e| format!("DB error: {}", e))?
        .into_iter()
        .filter(|bp| bp.ship_id == Some(sid))
        .map(|bp| serde_json::to_value(bp).unwrap_or_default())
        .collect::<Vec<_>>();

    let ship_json = serde_json::to_value(&ship).unwrap_or_default();
    Ok(serde_json::json!({
        "ship": ship_json,
        "blueprints": blueprints,
    }))
}

// ─── Help Requests ───────────────────────────────────────────────────────────

/// Submit a help request. Routing is determined first by category, then by the requester's role:
///   - category = "DATA"                                    → the_statistician
///   - aerospace_engineer / mathematician / physicist       → the_artificer
///   - biological_engineer / agricultural_engineer / chemist → the_observer
#[tauri::command]
pub async fn submit_help_request(
    token: String,
    title: String,
    description: Option<String>,
    category: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;

    if title.trim().is_empty() {
        return Err("Title is required".to_string());
    }

    // Data requests are always routed to the statistician regardless of role.
    let assigned_proxy_director =
        if category.as_deref().map(|c| c.eq_ignore_ascii_case("DATA")) == Some(true) {
            "the_statistician"
        } else {
            match session.role_name.as_str() {
                "aerospace_engineer" | "mathematician" | "physicist" => "the_artificer",
                "biological_engineer" | "agricultural_engineer" | "chemist" => "the_observer",
                _ => "the_artificer",
            }
        };

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO help_requests (requested_by, title, description, category, assigned_proxy_director, status) \
         VALUES ($1, $2, $3, $4, $5, 'open') RETURNING id",
    )
    .bind(session.user_id)
    .bind(&title)
    .bind(&description)
    .bind(&category)
    .bind(assigned_proxy_director)
    .fetch_one(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(
        Some(session.user_id),
        "SUBMIT_HELP_REQUEST",
        Some("help_requests"),
        Some(row.0),
        None,
        Some(serde_json::json!({ "title": title, "proxy": assigned_proxy_director })),
    )
    .await;

    Ok(row.0.to_string())
}

/// Retrieve help requests.
/// Directors see all requests assigned to their proxy role.
/// Engineers see their own requests.
#[tauri::command]
pub async fn get_help_requests(token: String) -> Result<Vec<serde_json::Value>, String> {
    let session = validate_session_command(&token).await?;

    let rows = if session.tier >= 3 {
        // Directors see requests routed to them
        let proxy_role = session.role_name.as_str();
        sqlx::query_as::<_, HelpRequestRow>(
            "SELECT id, requested_by, title, description, category, assigned_proxy_director, status, response, resolved_by, created_at, rejection_reason, rejected_at, created_task_id \
             FROM help_requests WHERE assigned_proxy_director = $1 AND deleted_at IS NULL ORDER BY created_at DESC",
        )
        .bind(proxy_role)
        .fetch_all(db::get_db())
        .await
    } else {
        // Engineers see their own requests
        sqlx::query_as::<_, HelpRequestRow>(
            "SELECT id, requested_by, title, description, category, assigned_proxy_director, status, response, resolved_by, created_at, rejection_reason, rejected_at, created_task_id \
             FROM help_requests WHERE requested_by = $1 AND deleted_at IS NULL ORDER BY created_at DESC",
        )
        .bind(session.user_id)
        .fetch_all(db::get_db())
        .await
    }
    .map_err(|e| format!("DB error: {}", e))?;

    // Batch fetch linked task details: collect task IDs grouped by task table.
    let mut aerospace_ids: Vec<Uuid> = Vec::new();
    let mut research_ids: Vec<Uuid> = Vec::new();
    let mut data_analyst_ids: Vec<Uuid> = Vec::new();
    for row in &rows {
        if let Some(tid) = row.created_task_id {
            match row.assigned_proxy_director.as_str() {
                "the_artificer" => aerospace_ids.push(tid),
                "the_statistician" => data_analyst_ids.push(tid),
                _ => research_ids.push(tid),
            }
        }
    }

    // Build a map from task_id → LinkedTaskInfo for quick lookup.
    let mut task_map: std::collections::HashMap<Uuid, (LinkedTaskInfo, &'static str)> =
        std::collections::HashMap::new();

    if !aerospace_ids.is_empty() {
        let tasks: Vec<LinkedTaskInfo> = sqlx::query_as(
            "SELECT id, title, assigned_to, status FROM aerospace_assigned_tasks WHERE id = ANY($1)",
        )
        .bind(&aerospace_ids)
        .fetch_all(db::get_db())
        .await
        .unwrap_or_default();
        for t in tasks {
            task_map.insert(t.id, (t, "aerospace"));
        }
    }

    if !research_ids.is_empty() {
        let tasks: Vec<LinkedTaskInfo> = sqlx::query_as(
            "SELECT id, title, assigned_to, status FROM research_tasks WHERE id = ANY($1)",
        )
        .bind(&research_ids)
        .fetch_all(db::get_db())
        .await
        .unwrap_or_default();
        for t in tasks {
            task_map.insert(t.id, (t, "research"));
        }
    }

    if !data_analyst_ids.is_empty() {
        let tasks: Vec<LinkedTaskInfo> = sqlx::query_as(
            "SELECT id, title, assigned_to, status FROM data_analyst_tasks WHERE id = ANY($1)",
        )
        .bind(&data_analyst_ids)
        .fetch_all(db::get_db())
        .await
        .unwrap_or_default();
        for t in tasks {
            task_map.insert(t.id, (t, "data"));
        }
    }

    let mut results = Vec::with_capacity(rows.len());
    for row in rows {
        let mut val = serde_json::to_value(&row).unwrap_or_default();

        if let (Some(tid), serde_json::Value::Object(ref mut map)) =
            (row.created_task_id, &mut val)
        {
            if let Some((info, task_type)) = task_map.get(&tid) {
                map.insert(
                    "linked_task_title".to_string(),
                    serde_json::Value::String(info.title.clone()),
                );
                map.insert(
                    "linked_task_assigned_to".to_string(),
                    info.assigned_to.map_or(serde_json::Value::Null, |u| {
                        serde_json::Value::String(u.to_string())
                    }),
                );
                map.insert(
                    "linked_task_status".to_string(),
                    info.status
                        .clone()
                        .map_or(serde_json::Value::Null, serde_json::Value::String),
                );
                map.insert(
                    "linked_task_type".to_string(),
                    serde_json::Value::String(task_type.to_string()),
                );
            }
        }

        results.push(val);
    }

    Ok(results)
}

/// Director marks a help request as in_review or closed (generic status update).
/// Use approve_help_request to convert to a task, reject_help_request to reject,
/// and proxy_deliver_task_response to deliver the completed task result.
#[tauri::command]
pub async fn resolve_help_request(
    token: String,
    request_id: String,
    status: String,
    response: Option<String>,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    require_role(&session, 3)?;

    let valid_statuses = ["in_review", "closed"];
    if !valid_statuses.contains(&status.as_str()) {
        return Err(format!("Invalid status '{}'. Must be one of: in_review, closed", status));
    }

    let rid = Uuid::parse_str(&request_id).map_err(|_| "Invalid request ID".to_string())?;

    sqlx::query(
        "UPDATE help_requests SET status = $1, response = COALESCE($2, response), resolved_by = $3 \
         WHERE id = $4 AND deleted_at IS NULL",
    )
    .bind(&status)
    .bind(&response)
    .bind(session.user_id)
    .bind(rid)
    .execute(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(
        Some(session.user_id),
        "RESOLVE_HELP_REQUEST",
        Some("help_requests"),
        Some(rid),
        None,
        Some(serde_json::json!({ "status": status })),
    )
    .await;

    Ok(())
}

/// Director rejects a help request with a mandatory reason.
/// Sets status to "rejected", records rejection_reason and rejected_at,
/// and dispatches an inbox message to the original requester.
/// The DB update and message dispatch are wrapped in a transaction: if
/// the message fails to send, the status update is rolled back so the
/// requester is never left without their notification ("return to desk").
#[tauri::command]
pub async fn reject_help_request(
    token: String,
    request_id: String,
    rejection_reason: String,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    require_role(&session, 3)?;

    if rejection_reason.trim().is_empty() {
        return Err("Rejection reason is required".to_string());
    }

    let rid = Uuid::parse_str(&request_id).map_err(|_| "Invalid request ID".to_string())?;

    // Fetch requester, title, and assigned proxy director so we can enforce domain
    // restriction and send the notification.
    let meta: Option<(Uuid, String, String)> = sqlx::query_as(
        "SELECT requested_by, title, assigned_proxy_director \
         FROM help_requests WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(rid)
    .fetch_optional(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let (requested_by, title, assigned_proxy_director) =
        meta.ok_or_else(|| "Help request not found".to_string())?;

    // Enforce domain restriction: only the Director assigned to this request
    // (or the Administrator) may reject it.
    if !is_admin(&session) && session.role_name != assigned_proxy_director {
        return Err(format!(
            "Access denied: this help request belongs to '{}'. Only that director or the Administrator may act on it.",
            assigned_proxy_director
        ));
    }

    // Open a transaction so the DB update and message dispatch are atomic.
    // If the notification fails, we roll back so the requester is never left
    // with a "rejected" record but no explanation in their inbox.
    let mut tx = db::get_db()
        .begin()
        .await
        .map_err(|e| format!("DB error starting transaction: {}", e))?;

    sqlx::query(
        "UPDATE help_requests SET status = 'rejected', rejection_reason = $1, rejected_at = NOW(), resolved_by = $2 \
         WHERE id = $3 AND deleted_at IS NULL AND status IN ('open', 'in_review')",
    )
    .bind(&rejection_reason)
    .bind(session.user_id)
    .bind(rid)
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    // Dispatch inbox notification to the original requester
    let subject = format!("Help Request Rejected: {}", title);
    let body = format!(
        "Your help request \"{}\" has been reviewed and rejected.\n\nReason: {}",
        title, rejection_reason
    );
    crate::queries::messages::send_message(
        session.user_id,
        &subject,
        &body,
        None,
        &[requested_by],
        &[],
        &[],
    )
    .await
    .map_err(|e| format!("Failed to deliver rejection notification: {}", e))?;

    tx.commit()
        .await
        .map_err(|e| format!("DB error committing transaction: {}", e))?;

    let _ = write_audit_log(
        Some(session.user_id),
        "REJECT_HELP_REQUEST",
        Some("help_requests"),
        Some(rid),
        None,
        Some(serde_json::json!({ "rejection_reason": rejection_reason })),
    )
    .await;

    Ok(())
}

/// Director approves a help request by converting it to an assigned task.
/// Creates the appropriate task record (aerospace or research) and links it
/// to the help request via created_task_id. Sets status to "converted".
/// Only the Director whose domain matches the request's assigned_proxy_director
/// (or the Administrator) may approve it.
#[tauri::command]
pub async fn approve_help_request(
    token: String,
    request_id: String,
    assigned_to_id: String,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    require_role(&session, 3)?;

    let rid = Uuid::parse_str(&request_id).map_err(|_| "Invalid request ID".to_string())?;
    let atid = Uuid::parse_str(&assigned_to_id).map_err(|_| "Invalid assignee ID".to_string())?;

    // Fetch the help request to get title, description, and proxy director
    let row: Option<(String, Option<String>, String)> = sqlx::query_as(
        "SELECT title, description, assigned_proxy_director FROM help_requests WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(rid)
    .fetch_optional(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let (title, description, proxy_director) =
        row.ok_or_else(|| "Help request not found".to_string())?;

    // Enforce domain restriction: only the Director assigned to this request
    // (or the Administrator) may approve it.
    if !is_admin(&session) && session.role_name != proxy_director {
        return Err(format!(
            "Access denied: this help request belongs to '{}'. Only that director or the Administrator may act on it.",
            proxy_director
        ));
    }

    // Create the linked task in the appropriate table based on the proxy director
    let task_id: Uuid = if proxy_director == "the_artificer" {
        ensure_aerospace_tasks_table().await?;
        let r: (Uuid,) = sqlx::query_as(
            "INSERT INTO aerospace_assigned_tasks (title, description, assigned_to, assigned_by, status) \
             VALUES ($1, $2, $3, $4, 'pending') RETURNING id",
        )
        .bind(&title)
        .bind(&description)
        .bind(atid)
        .bind(session.user_id)
        .fetch_one(db::get_db())
        .await
        .map_err(|e| format!("DB error creating aerospace task: {}", e))?;
        r.0
    } else if proxy_director == "the_statistician" {
        // the_statistician → data_analyst_tasks
        let r: (Uuid,) = sqlx::query_as(
            "INSERT INTO data_analyst_tasks (title, description, assigned_to, assigned_by, status) \
             VALUES ($1, $2, $3, $4, 'pending') RETURNING id",
        )
        .bind(&title)
        .bind(&description)
        .bind(atid)
        .bind(session.user_id)
        .fetch_one(db::get_db())
        .await
        .map_err(|e| format!("DB error creating data analyst task: {}", e))?;
        r.0
    } else {
        // the_observer → research_tasks
        let r: (Uuid,) = sqlx::query_as(
            "INSERT INTO research_tasks (title, description, assigned_to, assigned_by, status) \
             VALUES ($1, $2, $3, $4, 'pending') RETURNING id",
        )
        .bind(&title)
        .bind(&description)
        .bind(atid)
        .bind(session.user_id)
        .fetch_one(db::get_db())
        .await
        .map_err(|e| format!("DB error creating research task: {}", e))?;
        r.0
    };

    // Update the help request: mark as converted and store the new task's ID
    sqlx::query(
        "UPDATE help_requests SET status = 'converted', created_task_id = $1, resolved_by = $2 \
         WHERE id = $3 AND deleted_at IS NULL AND status IN ('open', 'in_review')",
    )
    .bind(task_id)
    .bind(session.user_id)
    .bind(rid)
    .execute(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(
        Some(session.user_id),
        "APPROVE_HELP_REQUEST",
        Some("help_requests"),
        Some(rid),
        None,
        Some(serde_json::json!({ "created_task_id": task_id.to_string(), "assigned_to": assigned_to_id })),
    )
    .await;

    Ok(task_id.to_string())
}

/// Director delivers the completed task result back to the original requester.
/// Called after the linked task has been completed; sets help request status to "resolved"
/// and dispatches an inbox message to the original requester with the response.
/// The DB update and message dispatch are wrapped in a transaction: if the message
/// fails to send, the status update is rolled back so the requester is never left
/// without their response ("return to desk" guarantee).
#[tauri::command]
pub async fn proxy_deliver_task_response(
    token: String,
    request_id: String,
    response: String,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    require_role(&session, 3)?;

    if response.trim().is_empty() {
        return Err("Response is required".to_string());
    }

    let rid = Uuid::parse_str(&request_id).map_err(|_| "Invalid request ID".to_string())?;

    // Fetch requester, title, and assigned proxy director so we can enforce domain
    // restriction and send the notification.
    let meta: Option<(Uuid, String, String)> = sqlx::query_as(
        "SELECT requested_by, title, assigned_proxy_director \
         FROM help_requests WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(rid)
    .fetch_optional(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let (requested_by, title, assigned_proxy_director) =
        meta.ok_or_else(|| "Help request not found".to_string())?;

    // Enforce domain restriction: only the Director assigned to this request
    // (or the Administrator) may deliver its response.
    if !is_admin(&session) && session.role_name != assigned_proxy_director {
        return Err(format!(
            "Access denied: this help request belongs to '{}'. Only that director or the Administrator may act on it.",
            assigned_proxy_director
        ));
    }

    // Open a transaction so the DB update and message dispatch are atomic.
    // If the notification fails, we roll back so the requester is never left
    // with a "resolved" record but no response in their inbox.
    let mut tx = db::get_db()
        .begin()
        .await
        .map_err(|e| format!("DB error starting transaction: {}", e))?;

    sqlx::query(
        "UPDATE help_requests SET status = 'resolved', response = $1, resolved_by = $2 \
         WHERE id = $3 AND deleted_at IS NULL AND status = 'converted'",
    )
    .bind(&response)
    .bind(session.user_id)
    .bind(rid)
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    // Dispatch inbox notification to the original requester
    let subject = format!("Help Request Fulfilled: {}", title);
    let body = format!(
        "Your help request \"{}\" has been completed and the results have been delivered by your proxy director.\n\nResponse: {}",
        title, response
    );
    crate::queries::messages::send_message(
        session.user_id,
        &subject,
        &body,
        None,
        &[requested_by],
        &[],
        &[],
    )
    .await
    .map_err(|e| format!("Failed to deliver task response notification: {}", e))?;

    tx.commit()
        .await
        .map_err(|e| format!("DB error committing transaction: {}", e))?;

    let _ = write_audit_log(
        Some(session.user_id),
        "PROXY_DELIVER_TASK_RESPONSE",
        Some("help_requests"),
        Some(rid),
        None,
        Some(serde_json::json!({ "response": response })),
    )
    .await;

    Ok(())
}
