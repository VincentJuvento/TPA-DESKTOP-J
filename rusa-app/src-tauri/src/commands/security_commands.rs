use crate::auth::{is_admin, permissions, require_role_name, validate_session_command};
use crate::queries::auth::write_audit_log;
use crate::queries::security as security_queries;
use tauri::Emitter;
use uuid::Uuid;

#[tauri::command]
pub async fn create_incident_report(
    token: String,
    title: String,
    description: String,
    location: Option<String>,
    incident_date: Option<String>,
    severity: String,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    let idate = incident_date.as_deref()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.with_timezone(&chrono::Utc));

    let id = security_queries::insert_incident_report(
        &title,
        &description,
        location.as_deref(),
        idate,
        &severity,
        session.user_id,
    )
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "CREATE_INCIDENT_REPORT", Some("incident_reports"), Some(id), None, Some(serde_json::json!({ "title": title, "severity": severity }))).await;
    Ok(id.to_string())
}

#[tauri::command]
pub async fn get_incident_reports(token: String) -> Result<Vec<serde_json::Value>, String> {
    let _session = validate_session_command(&token).await?;

    let rows = security_queries::get_all_incident_reports()
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn update_incident_status(
    token: String,
    report_id: String,
    status: String,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    let rid = Uuid::parse_str(&report_id).map_err(|_| "Invalid report ID".to_string())?;

    security_queries::update_incident_status(rid, &status)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "UPDATE_INCIDENT_STATUS", Some("incident_reports"), Some(rid), None, Some(serde_json::json!({ "status": status }))).await;
    Ok(())
}

#[tauri::command]
pub async fn add_lost_found_item(
    token: String,
    item_name: String,
    description: Option<String>,
    found_location: Option<String>,
    found_date: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "earth_security_head")?;

    let fdate = found_date.as_deref()
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());

    let id = security_queries::insert_lost_found_item(
        &item_name,
        description.as_deref(),
        found_location.as_deref(),
        fdate,
        session.user_id,
    )
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "ADD_LOST_FOUND", Some("lost_and_found"), Some(id), None, Some(serde_json::json!({ "item_name": item_name }))).await;
    Ok(id.to_string())
}

#[tauri::command]
pub async fn get_lost_found(token: String) -> Result<Vec<serde_json::Value>, String> {
    let _session = validate_session_command(&token).await?;

    let rows = security_queries::get_all_lost_found()
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn claim_lost_found(token: String, item_id: String) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    let iid = Uuid::parse_str(&item_id).map_err(|_| "Invalid item ID".to_string())?;

    security_queries::claim_lost_found_item(iid, session.user_id)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "CLAIM_LOST_FOUND", Some("lost_and_found"), Some(iid), None, None).await;
    Ok(())
}

#[tauri::command]
pub async fn submit_broadcast_request(
    token: String,
    title: String,
    content: String,
    target_audience: Option<String>,
    target_filters: Option<serde_json::Value>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;

    let routed_to = if session.role_name == "head_of_earth_security" || session.role_name == "head_of_galactic_security" {
        "the_guardian"
    } else {
        "the_anchorman"
    };

    let id = security_queries::insert_broadcast_request(
        &title,
        &content,
        target_audience.as_deref(),
        target_filters,
        routed_to,
        session.user_id,
    )
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "SUBMIT_BROADCAST_REQUEST", Some("broadcast_requests"), Some(id), None, Some(serde_json::json!({ "title": title }))).await;
    Ok(id.to_string())
}

#[tauri::command]
pub async fn get_broadcast_requests(token: String) -> Result<Vec<serde_json::Value>, String> {
    let session = validate_session_command(&token).await?;

    let routed_to_role = if session.role_name == "the_guardian" {
        Some("the_guardian")
    } else if session.role_name == "the_anchorman" {
        Some("the_anchorman")
    } else {
        None
    };

    let rows = security_queries::get_broadcast_requests(routed_to_role)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn send_broadcast_direct(
    app_handle: tauri::AppHandle,
    token: String,
    title: String,
    content: String,
    target_filters: Option<serde_json::Value>,
    scheduled_at: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    if !is_admin(&session) && session.role_name != "the_guardian" && session.role_name != "the_anchorman" {
        return Err("Only guardian or anchorman can send broadcasts directly".to_string());
    }

    let sched = scheduled_at
        .as_deref()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc));

    let target_user_ids = security_queries::resolve_broadcast_targets(target_filters.as_ref())
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    if target_user_ids.is_empty() {
        return Err("No recipients matched the target filters".to_string());
    }

    let sender_label = if session.role_name == "the_guardian" {
        "The Guardian"
    } else if session.role_name == "the_anchorman" {
        "The Anchorman"
    } else {
        "The Administrator"
    };

    let subject = format!("SYSTEM BROADCAST - [{}] - {}", sender_label, title);

    let msg_id = crate::queries::messages::send_message(
        session.user_id,
        &subject,
        &content,
        sched,
        &target_user_ids,
        &[],
        &[],
    )
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = sqlx::query("UPDATE messages SET is_broadcast = true, broadcast_sender = $1 WHERE id = $2")
        .bind(sender_label)
        .bind(msg_id)
        .execute(crate::db::get_db())
        .await;

    let should_emit_now = sched.map(|dt| dt <= chrono::Utc::now()).unwrap_or(true);
    if should_emit_now {
        let payload = serde_json::json!({
            "message_id": msg_id,
            "from": sender_label,
            "subject": subject,
            "is_broadcast": true,
            "target_users": target_user_ids
        });
        let _ = app_handle.emit("new_broadcast", payload);
    }

    let _ = write_audit_log(
        Some(session.user_id),
        "SEND_BROADCAST_DIRECT",
        Some("messages"),
        Some(msg_id),
        None,
        Some(serde_json::json!({ "title": title, "scheduled_at": scheduled_at })),
    )
    .await;

    Ok(msg_id.to_string())
}

#[tauri::command]
pub async fn review_broadcast_request(
    app_handle: tauri::AppHandle,
    token: String,
    request_id: String,
    status: String,
    notes: Option<String>,
    scheduled_at: Option<String>,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    if !is_admin(&session) && session.role_name != "the_guardian" && session.role_name != "the_anchorman" {
        return Err("Only guardian or anchorman can review broadcast requests".to_string());
    }

    let rid = Uuid::parse_str(&request_id).map_err(|_| "Invalid request ID".to_string())?;

    security_queries::update_broadcast_request_review(rid, &status, session.user_id, notes.as_deref())
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    if status == "approved" {
        let req = security_queries::get_broadcast_request_by_id(rid)
            .await
            .map_err(|e| format!("DB error: {}", e))?
            .ok_or_else(|| "Broadcast request not found".to_string())?;

        let sched = scheduled_at
            .as_deref()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc));

        let target_user_ids = security_queries::resolve_broadcast_targets(req.target_filters.as_ref())
            .await
            .map_err(|e| format!("DB error: {}", e))?;

        if !target_user_ids.is_empty() {
            let sender_label = if session.role_name == "the_guardian" {
                "The Guardian"
            } else if session.role_name == "the_anchorman" {
                "The Anchorman"
            } else {
                "The Administrator"
            };

            let subject = format!("SYSTEM BROADCAST - [{}] - {}", sender_label, req.title);
            let content = req.content.unwrap_or_default();

            let msg_id = crate::queries::messages::send_message(
                session.user_id,
                &subject,
                &content,
                sched,
                &target_user_ids,
                &[],
                &[],
            )
            .await
            .map_err(|e| format!("DB error: {}", e))?;

            let _ = sqlx::query("UPDATE messages SET is_broadcast = true, broadcast_sender = $1 WHERE id = $2")
                .bind(sender_label)
                .bind(msg_id)
                .execute(crate::db::get_db())
                .await;

            let _ = sqlx::query("UPDATE broadcast_requests SET status = 'broadcast' WHERE id = $1")
                .bind(rid)
                .execute(crate::db::get_db())
                .await;

            let should_emit_now = sched.map(|dt| dt <= chrono::Utc::now()).unwrap_or(true);
            if should_emit_now {
                let payload = serde_json::json!({
                    "message_id": msg_id,
                    "from": sender_label,
                    "subject": subject,
                    "is_broadcast": true,
                    "target_users": target_user_ids
                });
                let _ = app_handle.emit("new_broadcast", payload);
            }
        }
    }

    let _ = write_audit_log(Some(session.user_id), "REVIEW_BROADCAST_REQUEST", Some("broadcast_requests"), Some(rid), None, Some(serde_json::json!({ "status": status }))).await;
    Ok(())
}

#[tauri::command]
pub async fn submit_security_findings(
    token: String,
    title: String,
    description: Option<String>,
    findings_date: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    let fdate = findings_date.as_deref()
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());

    let id = security_queries::insert_security_finding(
        &title,
        description.as_deref(),
        fdate,
        session.user_id,
    )
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "SUBMIT_SECURITY_FINDINGS", Some("security_findings"), Some(id), None, Some(serde_json::json!({ "title": title }))).await;
    Ok(id.to_string())
}

#[tauri::command]
pub async fn get_security_findings(token: String) -> Result<Vec<serde_json::Value>, String> {
    let _session = validate_session_command(&token).await?;

    let rows = security_queries::get_all_security_findings()
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn assign_security_task(
    token: String,
    assigned_to: String,
    title: String,
    description: Option<String>,
    due_date: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    if !is_admin(&session) && session.role_name != "the_guardian" && session.role_name != "the_overseer" {
        return Err("Only guardian, overseer, or administrator can assign security tasks".to_string());
    }

    security_queries::ensure_security_tasks_table()
        .await
        .map_err(|e| format!("DB error creating table: {}", e))?;

    let atid = Uuid::parse_str(&assigned_to).map_err(|_| "Invalid user ID".to_string())?;
    let dd = due_date
        .as_deref()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.with_timezone(&chrono::Utc));

    let id = security_queries::insert_security_task(
        &title,
        description.as_deref(),
        atid,
        session.user_id,
        dd,
    )
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(
        Some(session.user_id),
        "ASSIGN_SECURITY_TASK",
        Some("security_tasks"),
        Some(id),
        None,
        Some(serde_json::json!({ "title": title, "assigned_to": assigned_to })),
    )
    .await;

    Ok(id.to_string())
}

#[tauri::command]
pub async fn get_security_tasks(token: String) -> Result<Vec<serde_json::Value>, String> {
    let session = validate_session_command(&token).await?;

    security_queries::ensure_security_tasks_table()
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let rows = if permissions::has_permission(&session.role_name, "the_guardian")
        || permissions::has_permission(&session.role_name, "the_overseer")
    {
        security_queries::get_security_tasks_for_assigner(session.user_id).await
    } else {
        security_queries::get_security_tasks_for_user(session.user_id).await
    }
    .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows
        .into_iter()
        .map(|r| serde_json::to_value(r).unwrap_or_default())
        .collect())
}

#[tauri::command]
pub async fn update_security_task_status(
    token: String,
    task_id: String,
    status: String,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;

    security_queries::ensure_security_tasks_table()
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let tid = Uuid::parse_str(&task_id).map_err(|_| "Invalid task ID".to_string())?;

    security_queries::update_security_task_status(tid, &status)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(
        Some(session.user_id),
        "UPDATE_SECURITY_TASK_STATUS",
        Some("security_tasks"),
        Some(tid),
        None,
        Some(serde_json::json!({ "status": status })),
    )
    .await;

    Ok(())
}

#[tauri::command]
pub async fn submit_external_report(
    token: String,
    title: String,
    description: String,
    security_type: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;

    let security_roles = ["the_guardian", "the_overseer", "earth_security_head", "earth_security_staff", "galactic_security_head", "galactic_security_staff"];
    if security_roles.contains(&session.role_name.as_str()) {
        return Err("Security personnel cannot submit external reports. Use incident reports instead.".to_string());
    }

    let id = security_queries::insert_external_report(
        &title,
        &description,
        session.user_id,
        security_type.as_deref(),
    )
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "SUBMIT_EXTERNAL_REPORT", Some("external_reports"), Some(id), None, Some(serde_json::json!({ "title": title }))).await;
    Ok(id.to_string())
}

#[tauri::command]
pub async fn get_external_reports(token: String) -> Result<Vec<serde_json::Value>, String> {
    let session = validate_session_command(&token).await?;

    let security_roles = ["the_guardian", "the_overseer", "earth_security_head", "earth_security_staff", "galactic_security_head", "galactic_security_staff"];
    let rows = if security_roles.contains(&session.role_name.as_str()) {
        security_queries::get_all_external_reports().await
    } else {
        security_queries::get_external_reports_by_submitter(session.user_id).await
    }
    .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}
