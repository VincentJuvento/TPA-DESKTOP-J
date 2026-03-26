use crate::auth::{is_admin, permissions, require_role_name, validate_session_command};
use crate::queries::astronaut as astronaut_queries;
use crate::queries::auth::write_audit_log;
use uuid::Uuid;

#[tauri::command]
pub async fn get_missions(token: String) -> Result<Vec<serde_json::Value>, String> {
    let _session = validate_session_command(&token).await?;

    let rows = astronaut_queries::get_all_missions()
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn create_mission(
    token: String,
    title: String,
    description: Option<String>,
    mission_type: String,
    ship_id: Option<String>,
    sector_id: Option<String>,
    planet_id: Option<String>,
    start_date: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "the_wanderer")?;

    let sid = ship_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());
    let secid = sector_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());
    let pid = planet_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());
    let sdate = start_date.as_deref()
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());

    let id = astronaut_queries::insert_mission(
        &title,
        description.as_deref(),
        &mission_type,
        sid,
        secid,
        pid,
        sdate,
        session.user_id,
    )
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "CREATE_MISSION", Some("missions"), Some(id), None, Some(serde_json::json!({ "title": title }))).await;
    Ok(id.to_string())
}

#[tauri::command]
pub async fn update_mission_status(
    token: String,
    mission_id: String,
    status: String,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    if session.role_name != "the_wanderer" && session.role_name != "the_taskmaster" && !is_admin(&session) {
        return Err("Only wanderer or taskmaster can update mission status".to_string());
    }

    if status == "completed" {
        return Err("Use the conclusion request workflow to mark missions as completed".to_string());
    }

    let mid = Uuid::parse_str(&mission_id).map_err(|_| "Invalid mission ID".to_string())?;

    astronaut_queries::update_mission_status(mid, &status)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "UPDATE_MISSION_STATUS", Some("missions"), Some(mid), None, Some(serde_json::json!({ "status": status }))).await;
    Ok(())
}

#[tauri::command]
pub async fn assign_crew(
    token: String,
    mission_id: String,
    astronaut_ids: Vec<String>,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "the_wanderer")?;

    let mid = Uuid::parse_str(&mission_id).map_err(|_| "Invalid mission ID".to_string())?;

    for aid_str in &astronaut_ids {
        let aid = Uuid::parse_str(aid_str).map_err(|_| "Invalid astronaut ID".to_string())?;
        astronaut_queries::insert_mission_crew(mid, aid)
            .await
            .map_err(|e| format!("DB error: {}", e))?;
    }

    let _ = write_audit_log(Some(session.user_id), "ASSIGN_CREW", Some("mission_crew"), Some(mid), None, Some(serde_json::json!({ "mission_id": mission_id, "count": astronaut_ids.len() }))).await;
    Ok(())
}

#[tauri::command]
pub async fn submit_mission_report(
    token: String,
    mission_id: String,
    report_type: String,
    content: String,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    let mid = Uuid::parse_str(&mission_id).map_err(|_| "Invalid mission ID".to_string())?;

    let id = astronaut_queries::insert_mission_report(mid, &report_type, &content, session.user_id)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "SUBMIT_MISSION_REPORT", Some("mission_reports"), Some(id), None, None).await;
    Ok(id.to_string())
}

#[tauri::command]
pub async fn get_mission_reports(
    token: String,
    mission_id: String,
) -> Result<Vec<serde_json::Value>, String> {
    let _session = validate_session_command(&token).await?;
    let mid = Uuid::parse_str(&mission_id).map_err(|_| "Invalid mission ID".to_string())?;

    let rows = astronaut_queries::get_mission_reports_by_mission(mid)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn create_exploration_journal(
    token: String,
    mission_id: Option<String>,
    title: String,
    content: String,
    is_public: bool,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    let mid = mission_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());

    let id = astronaut_queries::insert_journal(mid, &title, &content, is_public, session.user_id)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "CREATE_JOURNAL", Some("exploration_journals"), Some(id), None, Some(serde_json::json!({ "title": title }))).await;
    Ok(id.to_string())
}

#[tauri::command]
pub async fn get_journals(token: String) -> Result<Vec<serde_json::Value>, String> {
    let session = validate_session_command(&token).await?;

    let rows = astronaut_queries::get_journals_for_user(session.user_id)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn get_sectors(token: String) -> Result<Vec<serde_json::Value>, String> {
    let _session = validate_session_command(&token).await?;

    let rows = astronaut_queries::get_all_sectors()
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn create_sector(
    token: String,
    name: String,
    description: Option<String>,
    boundaries: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "the_wanderer")?;

    let id = astronaut_queries::insert_sector(
        &name,
        description.as_deref(),
        boundaries.as_deref(),
        session.user_id,
    )
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "CREATE_SECTOR", Some("sectors"), Some(id), None, Some(serde_json::json!({ "name": name }))).await;
    Ok(id.to_string())
}

#[tauri::command]
pub async fn rename_sector(
    token: String,
    sector_id: String,
    new_name: String,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "the_wanderer")?;

    let sid = Uuid::parse_str(&sector_id).map_err(|_| "Invalid sector ID".to_string())?;

    astronaut_queries::update_sector_name(sid, &new_name)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "RENAME_SECTOR", Some("sectors"), Some(sid), None, Some(serde_json::json!({ "new_name": new_name }))).await;
    Ok(())
}

#[tauri::command]
pub async fn get_ships(token: String) -> Result<Vec<serde_json::Value>, String> {
    let _session = validate_session_command(&token).await?;

    let rows = astronaut_queries::get_all_ships()
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn get_astronaut_stats(
    token: String,
    astronaut_id: Option<String>,
) -> Result<serde_json::Value, String> {
    let session = validate_session_command(&token).await?;
    let target_id = astronaut_id
        .as_deref()
        .and_then(|s| Uuid::parse_str(s).ok())
        .unwrap_or(session.user_id);

    let stats = astronaut_queries::get_astronaut_stats(target_id)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    Ok(serde_json::to_value(stats).unwrap_or_default())
}

#[tauri::command]
pub async fn submit_conclusion_request(
    token: String,
    mission_id: String,
    report_summary: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    if session.role_name != "astronaut" && !is_admin(&session) {
        return Err("Only astronauts can submit conclusion requests".to_string());
    }

    let summary = report_summary
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| "Report summary is required".to_string())?;

    astronaut_queries::ensure_conclusion_requests_table()
        .await
        .map_err(|e| format!("DB error creating table: {}", e))?;

    let mid = Uuid::parse_str(&mission_id).map_err(|_| "Invalid mission ID".to_string())?;

    let id = astronaut_queries::insert_conclusion_request(mid, session.user_id, &summary)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(
        Some(session.user_id),
        "SUBMIT_CONCLUSION_REQUEST",
        Some("mission_conclusion_requests"),
        Some(id),
        None,
        Some(serde_json::json!({ "mission_id": mission_id })),
    )
    .await;

    Ok(id.to_string())
}

#[tauri::command]
pub async fn get_conclusion_requests(token: String) -> Result<Vec<serde_json::Value>, String> {
    let session = validate_session_command(&token).await?;

    astronaut_queries::ensure_conclusion_requests_table()
        .await
        .map_err(|e| format!("DB error creating table: {}", e))?;

    let rows = if permissions::has_permission(&session.role_name, "the_wanderer")
        || permissions::has_permission(&session.role_name, "the_taskmaster")
    {
        astronaut_queries::get_all_conclusion_requests().await
    } else {
        astronaut_queries::get_user_conclusion_requests(session.user_id).await
    }
    .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows
        .into_iter()
        .map(|r| serde_json::to_value(r).unwrap_or_default())
        .collect())
}

#[tauri::command]
pub async fn review_conclusion_request(
    token: String,
    request_id: String,
    decision: String,
    review_notes: Option<String>,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    if session.role_name != "the_wanderer" && session.role_name != "the_taskmaster" && !is_admin(&session) {
        return Err(
            "Only the_wanderer or the_taskmaster can review conclusion requests".to_string(),
        );
    }

    astronaut_queries::ensure_conclusion_requests_table()
        .await
        .map_err(|e| format!("DB error creating table: {}", e))?;

    let rid = Uuid::parse_str(&request_id).map_err(|_| "Invalid request ID".to_string())?;

    if decision != "approved" && decision != "rejected" {
        return Err("Decision must be 'approved' or 'rejected'".to_string());
    }

    let mission_id = astronaut_queries::get_conclusion_request_mission_id(rid)
        .await
        .map_err(|e| format!("DB error: {}", e))?
        .ok_or_else(|| "Conclusion request not found".to_string())?;

    // Continuity check: only the wanderer who created the mission can approve the conclusion request
    let mission_created_by = astronaut_queries::get_mission_created_by(mission_id)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    match mission_created_by {
        Some(creator_id) if creator_id != session.user_id => {
            return Err("Only the user who created this mission can approve the conclusion request".to_string());
        }
        None => {
            return Err("Mission has no recorded creator; approval authority cannot be validated".to_string());
        }
        _ => {}
    }

    astronaut_queries::update_conclusion_request_review(
        rid,
        &decision,
        session.user_id,
        review_notes.as_deref(),
    )
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    if decision == "approved" {
        astronaut_queries::complete_mission(mission_id)
            .await
            .map_err(|e| format!("DB error: {}", e))?;

        // Increment mission_counter for the astronaut who submitted the conclusion request
        if let Ok(Some(submitter_id)) = astronaut_queries::get_conclusion_request_submitter(rid).await {
            let _ = astronaut_queries::increment_mission_counter(submitter_id).await;
        }
    }

    let _ = write_audit_log(
        Some(session.user_id),
        "REVIEW_CONCLUSION_REQUEST",
        Some("mission_conclusion_requests"),
        Some(rid),
        None,
        Some(serde_json::json!({ "decision": decision })),
    )
    .await;

    Ok(())
}

#[tauri::command]
pub async fn get_planets(token: String) -> Result<Vec<serde_json::Value>, String> {
    let _session = validate_session_command(&token).await?;

    let rows = astronaut_queries::get_all_planets()
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn create_planet(
    token: String,
    name: String,
    description: Option<String>,
    star_system: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "the_wanderer")?;

    let id = astronaut_queries::insert_planet(
        &name,
        description.as_deref(),
        star_system.as_deref(),
        session.user_id,
    )
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "CREATE_PLANET", Some("planets"), Some(id), None, Some(serde_json::json!({ "name": name }))).await;
    Ok(id.to_string())
}

#[tauri::command]
pub async fn rename_planet(
    token: String,
    planet_id: String,
    new_name: String,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "the_wanderer")?;

    let pid = Uuid::parse_str(&planet_id).map_err(|_| "Invalid planet ID".to_string())?;

    astronaut_queries::update_planet_name(pid, &new_name)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "RENAME_PLANET", Some("planets"), Some(pid), None, Some(serde_json::json!({ "new_name": new_name }))).await;
    Ok(())
}
