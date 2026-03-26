use crate::auth::{require_role, require_role_name, validate_session_command};
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

    let rows = if session.tier >= 2 {
        sqlx::query_as::<_, AerospaceTaskRow>(
            "SELECT id, title, description, assigned_to, assigned_by, status, progress_notes, due_date, created_at, conclusion_requested_at, conclusion_requested_by, conclusion_approved_at, conclusion_approved_by, final_notes, final_findings, methodology_summary, key_results, recommendations, limitations FROM aerospace_assigned_tasks WHERE assigned_by = $1 AND deleted_at IS NULL ORDER BY created_at DESC",
        )
        .bind(session.user_id)
        .fetch_all(db::get_db())
        .await
    } else {
        sqlx::query_as::<_, AerospaceTaskRow>(
            "SELECT id, title, description, assigned_to, assigned_by, status, progress_notes, due_date, created_at, conclusion_requested_at, conclusion_requested_by, conclusion_approved_at, conclusion_approved_by, final_notes, final_findings, methodology_summary, key_results, recommendations, limitations FROM aerospace_assigned_tasks WHERE assigned_to = $1 AND deleted_at IS NULL ORDER BY created_at DESC",
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
pub async fn update_aerospace_task_status(
    token: String,
    task_id: String,
    status: String,
    progress_notes: Option<String>,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;

    ensure_aerospace_tasks_table().await?;

    let tid = Uuid::parse_str(&task_id).map_err(|_| "Invalid task ID".to_string())?;

    sqlx::query(
        "UPDATE aerospace_assigned_tasks SET status = $1, progress_notes = COALESCE($2, progress_notes) WHERE id = $3 AND (assigned_to = $4 OR $5 >= 2) AND deleted_at IS NULL",
    )
    .bind(&status)
    .bind(&progress_notes)
    .bind(tid)
    .bind(session.user_id)
    .bind(session.tier)
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
        "Directors' vote required for blueprint proposal '{}' submitted by user {}. \
         Description: {}",
        ship_name, session.user_id, blueprint_description
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

    let tid = Uuid::parse_str(&task_id).map_err(|_| "Invalid task ID".to_string())?;

    if final_notes.trim().is_empty() {
        return Err("Final notes are required".to_string());
    }

    // Verify this task is assigned to the requesting engineer
    let row: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM aerospace_assigned_tasks WHERE id = $1 AND assigned_to = $2 AND deleted_at IS NULL",
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
        "UPDATE aerospace_assigned_tasks SET status = 'conclusion_requested', \
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
            sqlx::query(
                "UPDATE aerospace_assigned_tasks SET status = 'completed', \
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
                "UPDATE aerospace_assigned_tasks SET status = 'in_progress', \
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

/// Submit a help request. Routing is determined by the requester's role:
///   - aerospace_engineer → the_artificer
///   - biological_engineer / agricultural_engineer → the_observer
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

    let assigned_proxy_director = match session.role_name.as_str() {
        "aerospace_engineer" => "the_artificer",
        "biological_engineer" | "agricultural_engineer" => "the_observer",
        _ => "the_artificer",
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
            "SELECT id, requested_by, title, description, category, assigned_proxy_director, status, response, resolved_by, created_at \
             FROM help_requests WHERE assigned_proxy_director = $1 AND deleted_at IS NULL ORDER BY created_at DESC",
        )
        .bind(proxy_role)
        .fetch_all(db::get_db())
        .await
    } else {
        // Engineers see their own requests
        sqlx::query_as::<_, HelpRequestRow>(
            "SELECT id, requested_by, title, description, category, assigned_proxy_director, status, response, resolved_by, created_at \
             FROM help_requests WHERE requested_by = $1 AND deleted_at IS NULL ORDER BY created_at DESC",
        )
        .bind(session.user_id)
        .fetch_all(db::get_db())
        .await
    }
    .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

/// Director resolves or updates a help request (e.g., marks it converted to a task).
#[tauri::command]
pub async fn resolve_help_request(
    token: String,
    request_id: String,
    status: String,
    response: Option<String>,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    require_role(&session, 3)?;

    let valid_statuses = ["in_review", "converted", "resolved", "closed"];
    if !valid_statuses.contains(&status.as_str()) {
        return Err(format!("Invalid status '{}'. Must be one of: in_review, converted, resolved, closed", status));
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
