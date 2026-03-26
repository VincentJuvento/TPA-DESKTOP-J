use crate::auth::{permissions, require_role_name, validate_session_command};
use crate::queries::auth::write_audit_log;
use crate::queries::budget as budget_queries;
use crate::queries::governance as governance_queries;
use uuid::Uuid;

#[tauri::command]
pub async fn get_budget_requests(token: String) -> Result<Vec<serde_json::Value>, String> {
    let session = validate_session_command(&token).await?;

    let rows = if permissions::has_permission(&session.role_name, "the_accountant") {
        budget_queries::get_all_budget_requests().await
    } else {
        budget_queries::get_user_budget_requests(session.user_id).await
    }
    .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn review_budget_request(
    token: String,
    request_id: String,
    status: String,
    notes: Option<String>,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "the_accountant")?;

    let rid = Uuid::parse_str(&request_id).map_err(|_| "Invalid request ID".to_string())?;

    // Budget requests require a passed directors' vote before approval
    if status == "approved" {
        let vote_status = budget_queries::get_budget_request_vote_status(rid)
            .await
            .map_err(|e| format!("DB error: {}", e))?;

        match vote_status.as_deref() {
            Some("passed") => {} // vote has passed — allow approval
            Some(vs) => {
                return Err(format!(
                    "Cannot approve: the directors' vote has not passed yet (vote status: {}). \
                     All directors must vote and the vote must pass before this budget request \
                     can be approved.",
                    vs
                ))
            }
            None => {
                return Err(
                    "Cannot approve: no directors' vote is linked to this budget request. \
                     Use 'Initiate Budget Vote' to start a collective vote before approving."
                        .to_string(),
                )
            }
        }
    }

    budget_queries::update_budget_request_review(rid, &status, notes.as_deref())
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "REVIEW_BUDGET_REQUEST", Some("budget_requests"), Some(rid), None, Some(serde_json::json!({ "status": status }))).await;
    Ok(())
}

/// Initiate a collective directors' vote for a budget request.
/// Only the accountant or a director (tier >= 3) can start this.
#[tauri::command]
pub async fn initiate_budget_vote(
    token: String,
    request_id: String,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;

    // Accountant or director can initiate a budget vote
    if session.role_name != "the_accountant" && session.tier < 3 {
        return Err("Only the accountant or a director can initiate a budget vote".to_string());
    }

    let rid = Uuid::parse_str(&request_id).map_err(|_| "Invalid request ID".to_string())?;

    // Fetch request title for the vote title
    #[derive(sqlx::FromRow)]
    struct TitleRow {
        title: String,
    }
    let req = sqlx::query_as::<_, TitleRow>(
        "SELECT title FROM budget_requests WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(rid)
    .fetch_optional(crate::db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?
    .ok_or_else(|| "Budget request not found".to_string())?;

    let vote_title = format!("Budget Request Vote: {}", req.title);
    let vote_desc = format!(
        "Collective directors' vote required for budget request ID {}. \
         All directors must participate before this request can be approved.",
        rid
    );

    let vote_id = governance_queries::insert_vote_typed(
        &vote_title,
        Some(&vote_desc),
        session.user_id,
        "budget",
    )
    .await
    .map_err(|e| format!("DB error creating vote: {}", e))?;

    budget_queries::link_budget_request_to_vote(rid, vote_id)
        .await
        .map_err(|e| format!("DB error linking vote: {}", e))?;

    let _ = write_audit_log(
        Some(session.user_id),
        "INITIATE_BUDGET_VOTE",
        Some("budget_requests"),
        Some(rid),
        None,
        Some(serde_json::json!({ "vote_id": vote_id.to_string() })),
    )
    .await;

    Ok(vote_id.to_string())
}

#[tauri::command]
pub async fn flag_budget_request(
    token: String,
    request_id: String,
    reason: String,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "the_accountant")?;

    let rid = Uuid::parse_str(&request_id).map_err(|_| "Invalid request ID".to_string())?;

    budget_queries::flag_budget_request(rid, &reason)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "FLAG_BUDGET_REQUEST", Some("budget_requests"), Some(rid), None, Some(serde_json::json!({ "reason": reason }))).await;
    Ok(())
}

#[tauri::command]
pub async fn get_expenditure_reports(token: String) -> Result<Vec<serde_json::Value>, String> {
    let session = validate_session_command(&token).await?;

    let rows = if permissions::has_permission(&session.role_name, "the_accountant") {
        budget_queries::get_all_expenditure_reports().await
    } else {
        budget_queries::get_user_expenditure_reports(session.user_id).await
    }
    .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn flag_expenditure_report(
    token: String,
    report_id: String,
    reason: String,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "the_accountant")?;

    let rid = Uuid::parse_str(&report_id).map_err(|_| "Invalid report ID".to_string())?;

    budget_queries::flag_expenditure_report(rid, &reason)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "FLAG_EXPENDITURE_REPORT", Some("expenditure_reports"), Some(rid), None, Some(serde_json::json!({ "reason": reason }))).await;
    Ok(())
}

#[tauri::command]
pub async fn submit_investigation(
    token: String,
    title: String,
    description: String,
    related_report_id: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "the_accountant")?;

    let rid = related_report_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());

    let id = budget_queries::insert_investigation(&title, &description, rid, session.user_id)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "SUBMIT_INVESTIGATION", Some("budget_investigations"), Some(id), None, Some(serde_json::json!({ "title": title }))).await;
    Ok(id.to_string())
}
