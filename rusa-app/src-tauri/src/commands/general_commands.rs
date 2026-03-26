use crate::auth::{require_role, validate_session_command};
use crate::queries::auth::write_audit_log;
use crate::queries::general as general_queries;
use crate::queries::governance as governance_queries;
use uuid::Uuid;

/// Any authenticated user can submit a general request.
/// On submission, a directors' vote is automatically initiated.
#[tauri::command]
pub async fn submit_general_request(
    token: String,
    title: String,
    description: String,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;

    general_queries::ensure_general_requests_table()
        .await
        .map_err(|e| format!("DB error creating table: {}", e))?;

    // Insert the general request
    let request_id = general_queries::insert_general_request(
        &title,
        &description,
        "general",
        session.user_id,
    )
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    // Auto-initiate a directors' vote for this request
    let vote_title = format!("General Request: {}", title);
    let vote_desc = format!(
        "Auto-initiated vote for general request submitted by user {}. Description: {}",
        session.user_id, description
    );

    let vote_id = governance_queries::insert_vote_typed(
        &vote_title,
        Some(&vote_desc),
        session.user_id,
        "general",
    )
    .await
    .map_err(|e| format!("DB error creating vote: {}", e))?;

    // Link the vote to the request
    general_queries::link_general_request_to_vote(request_id, vote_id)
        .await
        .map_err(|e| format!("DB error linking vote: {}", e))?;

    let _ = write_audit_log(
        Some(session.user_id),
        "SUBMIT_GENERAL_REQUEST",
        Some("general_requests"),
        Some(request_id),
        None,
        Some(serde_json::json!({ "title": title, "vote_id": vote_id.to_string() })),
    )
    .await;

    Ok(request_id.to_string())
}

/// Fetch general requests. Directors see all; others see their own.
#[tauri::command]
pub async fn get_general_requests(token: String) -> Result<Vec<serde_json::Value>, String> {
    let session = validate_session_command(&token).await?;

    general_queries::ensure_general_requests_table()
        .await
        .map_err(|e| format!("DB error creating table: {}", e))?;

    let rows = if session.tier >= 3 {
        general_queries::get_all_general_requests().await
    } else {
        general_queries::get_user_general_requests(session.user_id).await
    }
    .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows
        .into_iter()
        .map(|r| serde_json::to_value(r).unwrap_or_default())
        .collect())
}

/// Directors can approve or reject a general request, but only after the linked vote has passed.
#[tauri::command]
pub async fn review_general_request(
    token: String,
    request_id: String,
    status: String,
    notes: Option<String>,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    require_role(&session, 3)?;

    if status != "approved" && status != "rejected" {
        return Err("Status must be 'approved' or 'rejected'".to_string());
    }

    general_queries::ensure_general_requests_table()
        .await
        .map_err(|e| format!("DB error creating table: {}", e))?;

    let rid = Uuid::parse_str(&request_id).map_err(|_| "Invalid request ID".to_string())?;

    // Only allow approval when the linked vote has passed
    if status == "approved" {
        let vote_status = general_queries::get_general_request_vote_status(rid)
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
                    "Cannot approve: no vote is linked to this request or the vote does not exist."
                        .to_string(),
                )
            }
        }
    }

    general_queries::update_general_request_review(
        rid,
        &status,
        session.user_id,
        notes.as_deref(),
    )
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(
        Some(session.user_id),
        "REVIEW_GENERAL_REQUEST",
        Some("general_requests"),
        Some(rid),
        None,
        Some(serde_json::json!({ "status": status })),
    )
    .await;

    Ok(())
}
