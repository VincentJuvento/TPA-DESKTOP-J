use crate::auth::{permissions, require_role_name, validate_session_command};
use crate::queries::auth::write_audit_log;
use crate::queries::data as data_queries;
use crate::queries::messages as message_queries;
use base64::Engine;
use tauri::Emitter;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct DataResponseAttachmentInput {
    pub filename: String,
    pub mime_type: Option<String>,
    pub base64: String,
}

// Any authenticated user can submit
#[tauri::command]
pub async fn submit_data_request(
    token: String,
    title: String,
    requested_data_items: String,
    reason_of_request: String,
    description: Option<String>,
    data_type: Option<String>,
    requester_location: String,
    requester_tel_fax: String,
    requester_department: String,
    requester_department_email: String,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;

    if title.trim().is_empty() {
        return Err("Title is required".to_string());
    }
    if requested_data_items.trim().is_empty() {
        return Err("Requested data must be itemized".to_string());
    }
    if reason_of_request.trim().is_empty() {
        return Err("Reason of request is required".to_string());
    }
    if requester_location.trim().is_empty() {
        return Err("Location is required".to_string());
    }
    if requester_tel_fax.trim().is_empty() {
        return Err("Tel/Fax is required".to_string());
    }
    if requester_department.trim().is_empty() {
        return Err("Department is required".to_string());
    }
    if requester_department_email.trim().is_empty() {
        return Err("Department email is required".to_string());
    }

    let synthesized_description = description.unwrap_or_else(|| {
        format!(
            "Requested Data:\n{}\n\nReason:\n{}",
            requested_data_items.trim(),
            reason_of_request.trim()
        )
    });

    let id = data_queries::insert_data_request(
        &title,
        &synthesized_description,
        data_type.as_deref(),
        &requested_data_items,
        &reason_of_request,
        &requester_location,
        &requester_tel_fax,
        &requester_department,
        &requester_department_email,
        session.user_id,
        &session.full_name,
        &session.full_name,
    )
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "SUBMIT_DATA_REQUEST", Some("data_requests"), Some(id), None, Some(serde_json::json!({ "title": title }))).await;
    Ok(id.to_string())
}

// Role-scoped fetch:
// - the_statistician: sees ALL requests
// - data_analyst: sees only 'approved', 'processing', and 'analyst_submitted' requests
// - others: see only their own requests
#[tauri::command]
pub async fn get_data_requests(token: String) -> Result<Vec<serde_json::Value>, String> {
    let session = validate_session_command(&token).await?;

    let rows = if permissions::has_permission(&session.role_name, "the_statistician") {
        data_queries::get_all_data_requests().await
    } else if permissions::has_permission(&session.role_name, "data_analyst") {
        data_queries::get_analyst_data_requests(session.user_id).await
    } else {
        data_queries::get_user_data_requests(session.user_id).await
    }
    .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

// Step 2: The Statistician approves or rejects, and optionally assigns to a data analyst
#[tauri::command]
pub async fn review_data_request(
    token: String,
    request_id: String,
    status: String,
    notes: Option<String>,
    assigned_to: Option<String>,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "the_statistician")?;

    let rid = Uuid::parse_str(&request_id).map_err(|_| "Invalid request ID".to_string())?;
    let aid = assigned_to.as_deref().and_then(|s| Uuid::parse_str(s).ok());

    let updated = data_queries::update_data_request_review(rid, &status, session.user_id, notes.as_deref(), aid)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    if !updated {
        return Err("Request is not pending review".to_string());
    }

    let _ = write_audit_log(Some(session.user_id), "REVIEW_DATA_REQUEST", Some("data_requests"), Some(rid), None, Some(serde_json::json!({ "status": status }))).await;
    Ok(())
}

// Step 2b: Data Analyst marks a request as processing (starts working on it)
#[tauri::command]
pub async fn start_processing(token: String, request_id: String) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "data_analyst")?;

    let rid = Uuid::parse_str(&request_id).map_err(|_| "Invalid request ID".to_string())?;

    let data_type = data_queries::get_data_request_data_type(rid)
        .await
        .map_err(|e| format!("DB error: {}", e))?;
    if data_type.as_deref().map(|dt| dt.to_lowercase() == "medical").unwrap_or(false) {
        let _ = write_audit_log(Some(session.user_id), "MEDICAL_DATA_ACCESS_DENIED", Some("data_requests"), Some(rid), None, None).await;
        return Err("Access denied: medical data requests cannot be processed by data analysts".to_string());
    }

    let updated = data_queries::update_data_request_processing(rid, session.user_id)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    if !updated {
        return Err("Request is not approved for processing (or assigned to another analyst)".to_string());
    }

    let _ = write_audit_log(Some(session.user_id), "START_PROCESSING", Some("data_requests"), Some(rid), None, None).await;
    Ok(())
}

// Step 3: Data Analyst processes and submits results back to The Statistician
#[tauri::command]
pub async fn analyst_submit_response(
    token: String,
    request_id: String,
    response_markdown: String,
    response_status: String,
    response_explanation: Option<String>,
    analyst_notes: Option<String>,
    provided_by: Vec<String>,
    attachments: Vec<DataResponseAttachmentInput>,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "data_analyst")?;

    let rid = Uuid::parse_str(&request_id).map_err(|_| "Invalid request ID".to_string())?;

    let data_type = data_queries::get_data_request_data_type(rid)
        .await
        .map_err(|e| format!("DB error: {}", e))?;
    if data_type.as_deref().map(|dt| dt.to_lowercase() == "medical").unwrap_or(false) {
        let _ = write_audit_log(Some(session.user_id), "MEDICAL_DATA_ACCESS_DENIED", Some("data_requests"), Some(rid), None, None).await;
        return Err("Access denied: medical data requests cannot be processed by data analysts".to_string());
    }

    if response_markdown.trim().is_empty() && response_status.trim().to_lowercase() == "provided" {
        return Err("Provided data output is required".to_string());
    }
    if response_status.trim().to_lowercase() == "rejected" && response_explanation.as_deref().unwrap_or("").trim().is_empty() {
        return Err("Explanation is required for rejected responses".to_string());
    }

    let mut provided_ids: Vec<Uuid> = provided_by
        .iter()
        .filter_map(|s| Uuid::parse_str(s).ok())
        .collect();
    if !provided_ids.contains(&session.user_id) {
        provided_ids.push(session.user_id);
    }

    let provided_names = data_queries::get_user_full_names(&provided_ids)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let updated = data_queries::update_data_request_analyst_response_v2(
        rid,
        &response_markdown,
        &response_status,
        response_explanation.as_deref(),
        analyst_notes.as_deref(),
        session.user_id,
        &provided_ids,
        &provided_names,
    )
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    if !updated {
        return Err("Request is not available for response (or assigned to another analyst)".to_string());
    }

    if !attachments.is_empty() {
        let mut decoded = Vec::with_capacity(attachments.len());
        for a in attachments {
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(&a.base64)
                .map_err(|_| "Invalid attachment encoding".to_string())?;
            if bytes.is_empty() {
                return Err("Attachment is empty".to_string());
            }
            decoded.push((a.filename, a.mime_type, bytes));
        }

        data_queries::insert_data_response_attachments(rid, session.user_id, decoded)
            .await
            .map_err(|e| format!("DB error: {}", e))?;
    }

    let _ = write_audit_log(Some(session.user_id), "ANALYST_SUBMIT_RESPONSE", Some("data_requests"), Some(rid), None, None).await;
    Ok(())
}

// Step 3b: Data Analyst rejects a data request with a reason
#[tauri::command]
pub async fn analyst_reject_request(
    token: String,
    request_id: String,
    rejection_reason: String,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "data_analyst")?;

    if rejection_reason.trim().is_empty() {
        return Err("Rejection reason is required".to_string());
    }

    let rid = Uuid::parse_str(&request_id).map_err(|_| "Invalid request ID".to_string())?;

    let data_type = data_queries::get_data_request_data_type(rid)
        .await
        .map_err(|e| format!("DB error: {}", e))?;
    if data_type.as_deref().map(|dt| dt.to_lowercase() == "medical").unwrap_or(false) {
        let _ = write_audit_log(Some(session.user_id), "MEDICAL_DATA_ACCESS_DENIED", Some("data_requests"), Some(rid), None, None).await;
        return Err("Access denied: medical data requests cannot be processed by data analysts".to_string());
    }

    let provided_ids = vec![session.user_id];
    let provided_names = data_queries::get_user_full_names(&provided_ids)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let updated = data_queries::update_data_request_analyst_response_v2(
        rid,
        "",
        "rejected",
        Some(&rejection_reason),
        None,
        session.user_id,
        &provided_ids,
        &provided_names,
    )
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    if !updated {
        return Err("Request is not available for response (or assigned to another analyst)".to_string());
    }

    let _ = write_audit_log(Some(session.user_id), "ANALYST_REJECT_REQUEST", Some("data_requests"), Some(rid), None, Some(serde_json::json!({ "reason": rejection_reason }))).await;
    Ok(())
}

// Step 4: The Statistician delivers to requester after final review
#[tauri::command]
pub async fn deliver_data_response(
    app_handle: tauri::AppHandle,
    token: String,
    request_id: String,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "the_statistician")?;

    let rid = Uuid::parse_str(&request_id).map_err(|_| "Invalid request ID".to_string())?;

    let payload = data_queries::get_data_response_delivery_payload(rid)
        .await
        .map_err(|e| format!("DB error: {}", e))?
        .ok_or_else(|| "Request not ready for delivery".to_string())?;

    let subject = format!("Data Response Delivered — {} (Request {})", payload.title, rid);

    let mut body = String::new();
    body.push_str("Data Response\n\n");
    body.push_str(&format!("Request ID: {}\n", rid));
    body.push_str(&format!("Requested By: {}\n", payload.requested_by_name));
    body.push_str(&format!("Department: {}\n", payload.requester_department));
    body.push_str(&format!("Department Email: {}\n", payload.requester_department_email));
    body.push_str(&format!(
        "Date of Response: {}\n",
        payload
            .response_submitted_at
            .map(|d| d.date_naive().to_string())
            .unwrap_or_else(|| "—".to_string())
    ));
    body.push_str(&format!(
        "Status: {}\n\n",
        payload.response_status.to_uppercase()
    ));
    if payload.response_status.to_lowercase() == "rejected" {
        body.push_str("Explanation:\n");
        body.push_str(payload.response_explanation.as_deref().unwrap_or("—"));
        body.push_str("\n\n");
    }
    body.push_str("Provided Data Output:\n");
    body.push_str(payload.response_markdown.as_deref().unwrap_or("—"));
    body.push_str("\n\n");
    body.push_str("Attachments:\nOpen Data Services → My Data Requests → Attachments\n");

    let message_id = message_queries::send_message(
        session.user_id,
        &subject,
        &body,
        None,
        &[payload.requested_by],
        &[],
        &[],
    )
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let delivered = data_queries::mark_data_response_delivered(rid, session.user_id, message_id)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    if !delivered {
        let _ = message_queries::recall_message(message_id, session.user_id).await;
        return Err("Request not ready for delivery".to_string());
    }

    let event_payload = serde_json::json!({
        "message_id": message_id,
        "from": session.username,
        "subject": subject,
    });
    let _ = app_handle.emit("new_message", event_payload);

    let _ = write_audit_log(Some(session.user_id), "DELIVER_DATA_RESPONSE", Some("data_requests"), Some(rid), None, None).await;
    Ok(())
}

#[tauri::command]
pub async fn acknowledge_data_response(token: String, request_id: String) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    let rid = Uuid::parse_str(&request_id).map_err(|_| "Invalid request ID".to_string())?;

    let ok = data_queries::acknowledge_data_response(rid, session.user_id)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    if !ok {
        return Err("Cannot acknowledge: not delivered, not owner, or already acknowledged".to_string());
    }

    let _ = write_audit_log(Some(session.user_id), "ACKNOWLEDGE_DATA_RESPONSE", Some("data_requests"), Some(rid), None, None).await;
    Ok(())
}

#[tauri::command]
pub async fn list_data_response_attachments(
    token: String,
    request_id: String,
) -> Result<Vec<serde_json::Value>, String> {
    let session = validate_session_command(&token).await?;
    let rid = Uuid::parse_str(&request_id).map_err(|_| "Invalid request ID".to_string())?;

    let (owner_id, status, data_type) = data_queries::get_data_request_owner_status(rid)
        .await
        .map_err(|e| format!("DB error: {}", e))?
        .ok_or_else(|| "Request not found".to_string())?;

    let is_medical = data_type.as_deref().map(|dt| dt.to_lowercase() == "medical").unwrap_or(false);
    let allowed = if permissions::has_permission(&session.role_name, "the_statistician") {
        true
    } else if permissions::has_permission(&session.role_name, "data_analyst") {
        !is_medical
    } else {
        owner_id == session.user_id && status.as_deref() == Some("delivered")
    };

    if !allowed {
        return Err("Access denied".to_string());
    }

    let rows = data_queries::list_data_response_attachments(rid)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows
        .into_iter()
        .map(|r| serde_json::to_value(r).unwrap_or_default())
        .collect())
}

#[tauri::command]
pub async fn download_data_response_attachment(
    token: String,
    attachment_id: String,
) -> Result<serde_json::Value, String> {
    let session = validate_session_command(&token).await?;
    let aid = Uuid::parse_str(&attachment_id).map_err(|_| "Invalid attachment ID".to_string())?;

    let att = data_queries::get_data_response_attachment(aid)
        .await
        .map_err(|e| format!("DB error: {}", e))?
        .ok_or_else(|| "Attachment not found".to_string())?;

    let (owner_id, status, data_type) = data_queries::get_data_request_owner_status(att.request_id)
        .await
        .map_err(|e| format!("DB error: {}", e))?
        .ok_or_else(|| "Request not found".to_string())?;

    let is_medical = data_type.as_deref().map(|dt| dt.to_lowercase() == "medical").unwrap_or(false);
    let allowed = if permissions::has_permission(&session.role_name, "the_statistician") {
        true
    } else if permissions::has_permission(&session.role_name, "data_analyst") {
        !is_medical
    } else {
        owner_id == session.user_id && status.as_deref() == Some("delivered")
    };

    if !allowed {
        return Err("Access denied".to_string());
    }

    let b64 = base64::engine::general_purpose::STANDARD.encode(att.bytes);
    Ok(serde_json::json!({
        "filename": att.filename,
        "mime_type": att.mime_type,
        "base64": b64
    }))
}
