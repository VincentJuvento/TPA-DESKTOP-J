use crate::auth::{permissions, require_role_name, validate_session_command};
use crate::models::common::AppError;
use crate::queries::auth::write_audit_log;
use crate::queries::sanitary as sanitary_queries;
use uuid::Uuid;

#[tauri::command]
pub async fn get_sanitary_tasks(token: String) -> Result<Vec<serde_json::Value>, AppError> {
    let session = validate_session_command(&token).await?;
    sanitary_queries::ensure_sanitary_task_columns().await?;

    let rows = if permissions::has_permission(&session.role_name, "head_of_sanitary") {
        sanitary_queries::get_all_sanitary_tasks().await
    } else {
        sanitary_queries::get_user_sanitary_tasks(session.user_id).await
    }?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn assign_sanitary_task(
    token: String,
    assigned_to: String,
    title: String,
    description: Option<String>,
    division: Option<String>,
    due_date: Option<String>,
) -> Result<String, AppError> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "head_of_sanitary")?;
    sanitary_queries::ensure_sanitary_task_columns().await?;

    let atid = Uuid::parse_str(&assigned_to).map_err(|_| "Invalid user ID")?;
    let ddate = due_date.as_deref()
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());

    let id = sanitary_queries::insert_sanitary_task(
        &title,
        description.as_deref(),
        division.as_deref(),
        atid,
        session.user_id,
        ddate,
    )
    .await?;

    let _ = write_audit_log(Some(session.user_id), "ASSIGN_SANITARY_TASK", Some("sanitary_tasks"), Some(id), None, Some(serde_json::json!({ "title": title, "assigned_to": assigned_to }))).await;
    Ok(id.to_string())
}

#[tauri::command]
pub async fn update_sanitary_task(
    token: String,
    task_id: String,
    status: String,
) -> Result<(), AppError> {
    let session = validate_session_command(&token).await?;
    let tid = Uuid::parse_str(&task_id).map_err(|_| "Invalid task ID")?;
    sanitary_queries::ensure_sanitary_task_columns().await?;

    let task = sanitary_queries::get_sanitary_task_by_id(tid, session.user_id).await?;
    let task = task.ok_or("Task not found or access denied")?;
    let is_assigner = task.assigned_by.map(|id| id == session.user_id).unwrap_or(false);
    let is_head = permissions::has_permission(&session.role_name, "head_of_sanitary");

    if !is_assigner && !is_head && status == "completed" {
        return Err(AppError::from("Only the task assigner can mark a task as completed"));
    }
    if !is_assigner && !is_head && task.status.as_deref() == Some("conclusion_requested") {
        return Err(AppError::from("Status cannot be changed while awaiting assigner review"));
    }

    sanitary_queries::update_sanitary_task_status(tid, &status, session.user_id, &session.full_name, &session.role_name)
        .await?;

    let _ = write_audit_log(Some(session.user_id), "UPDATE_SANITARY_TASK", Some("sanitary_tasks"), Some(tid), None, Some(serde_json::json!({ "status": status }))).await;
    Ok(())
}

#[tauri::command]
pub async fn get_sanitary_task(token: String, task_id: String) -> Result<serde_json::Value, AppError> {
    let session = validate_session_command(&token).await?;
    sanitary_queries::ensure_sanitary_task_columns().await?;
    let tid = Uuid::parse_str(&task_id).map_err(|_| "Invalid task ID")?;

    let row = sanitary_queries::get_sanitary_task_by_id(tid, session.user_id).await?;
    row.map(|r| serde_json::to_value(r).unwrap_or_default())
        .ok_or_else(|| AppError::from("Task not found"))
}

#[tauri::command]
pub async fn append_sanitary_task_activity_log(
    token: String,
    task_id: String,
    content: String,
) -> Result<(), AppError> {
    let session = validate_session_command(&token).await?;
    sanitary_queries::ensure_sanitary_task_columns().await?;
    let tid = Uuid::parse_str(&task_id).map_err(|_| "Invalid task ID")?;

    let task = sanitary_queries::get_sanitary_task_by_id(tid, session.user_id).await?;
    if task.is_none() {
        return Err(AppError::from("Task not found or access denied"));
    }

    sanitary_queries::append_sanitary_task_log(tid, session.user_id, &session.full_name, &content).await?;
    let _ = write_audit_log(Some(session.user_id), "APPEND_SANITARY_TASK_LOG", Some("sanitary_tasks"), Some(tid), None, None).await;
    Ok(())
}

#[tauri::command]
pub async fn request_sanitary_task_conclusion(
    token: String,
    task_id: String,
    notes: String,
) -> Result<(), AppError> {
    let session = validate_session_command(&token).await?;
    sanitary_queries::ensure_sanitary_task_columns().await?;
    let tid = Uuid::parse_str(&task_id).map_err(|_| "Invalid task ID")?;

    if notes.trim().is_empty() {
        return Err(AppError::from("Conclusion notes are required"));
    }

    let task = sanitary_queries::get_sanitary_task_by_id(tid, session.user_id).await?;
    let task = task.ok_or("Task not found or access denied")?;
    if task.assigned_to != Some(session.user_id) {
        return Err(AppError::from("Only assigned subordinate can request conclusion"));
    }
    let current = task.status.as_deref().unwrap_or("pending");
    if current == "conclusion_requested" || current == "completed" {
        return Err(AppError::from("Task is already in conclusion or completed state"));
    }

    sanitary_queries::request_sanitary_task_conclusion(tid, session.user_id, &session.full_name, &notes).await?;
    let _ = write_audit_log(Some(session.user_id), "REQUEST_SANITARY_TASK_CONCLUSION", Some("sanitary_tasks"), Some(tid), None, None).await;
    Ok(())
}

#[tauri::command]
pub async fn review_sanitary_task_conclusion(
    token: String,
    task_id: String,
    decision: String,
    review_notes: Option<String>,
) -> Result<(), AppError> {
    let session = validate_session_command(&token).await?;
    sanitary_queries::ensure_sanitary_task_columns().await?;
    let tid = Uuid::parse_str(&task_id).map_err(|_| "Invalid task ID")?;

    let task = sanitary_queries::get_sanitary_task_by_id(tid, session.user_id).await?;
    let task = task.ok_or("Task not found or access denied")?;
    if task.assigned_by != Some(session.user_id) {
        return Err(AppError::from("Only assigner can review conclusion"));
    }
    if task.status.as_deref() != Some("conclusion_requested") {
        return Err(AppError::from("Task is not awaiting conclusion review"));
    }

    let approve = match decision.as_str() {
        "approve" => true,
        "reject" => false,
        _ => return Err(AppError::from("Invalid decision. Use 'approve' or 'reject'")),
    };

    sanitary_queries::review_sanitary_task_conclusion(tid, session.user_id, &session.full_name, approve, review_notes.as_deref()).await?;
    let _ = write_audit_log(Some(session.user_id), "REVIEW_SANITARY_TASK_CONCLUSION", Some("sanitary_tasks"), Some(tid), None, Some(serde_json::json!({ "decision": decision }))).await;
    Ok(())
}

#[tauri::command]
pub async fn get_sanitary_inventory(token: String) -> Result<Vec<serde_json::Value>, AppError> {
    let _session = validate_session_command(&token).await?;

    let rows = sanitary_queries::get_all_sanitary_inventory().await?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn update_sanitary_inventory(
    token: String,
    item_name: String,
    category: Option<String>,
    quantity: i32,
    unit: Option<String>,
) -> Result<String, AppError> {
    let session = validate_session_command(&token).await?;

    let id = sanitary_queries::insert_sanitary_inventory(
        &item_name,
        category.as_deref(),
        quantity,
        unit.as_deref(),
        session.user_id,
    )
    .await?;

    let _ = write_audit_log(Some(session.user_id), "UPDATE_SANITARY_INVENTORY", Some("sanitary_inventory"), Some(id), None, Some(serde_json::json!({ "item_name": item_name, "quantity": quantity }))).await;
    Ok(id.to_string())
}

#[tauri::command]
pub async fn add_disposal_log(
    token: String,
    item_name: String,
    quantity: f64,
    unit: Option<String>,
    disposal_method: Option<String>,
    hazard_level: Option<String>,
    notes: Option<String>,
) -> Result<String, AppError> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "disposal_crew")?;

    let quantity_dec = rust_decimal::Decimal::try_from(quantity)
        .map_err(|e| format!("Invalid quantity value '{}': {}", quantity, e))?;

    let id = sanitary_queries::insert_disposal_log(
        &item_name,
        quantity_dec,
        unit.as_deref(),
        disposal_method.as_deref(),
        hazard_level.as_deref(),
        notes.as_deref(),
        session.user_id,
    )
    .await?;

    let _ = write_audit_log(Some(session.user_id), "ADD_DISPOSAL_LOG", Some("disposal_logs"), Some(id), None, Some(serde_json::json!({ "item_name": item_name, "quantity": quantity }))).await;
    Ok(id.to_string())
}

#[tauri::command]
pub async fn get_disposal_logs(token: String) -> Result<Vec<serde_json::Value>, AppError> {
    let _session = validate_session_command(&token).await?;

    let rows = sanitary_queries::get_all_disposal_logs().await?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn add_wastewater_log(
    token: String,
    volume_treated: f64,
    unit: Option<String>,
    treatment_method: Option<String>,
    ph_level: Option<f64>,
    quality_notes: Option<String>,
) -> Result<String, AppError> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "wastewater_crew")?;

    let volume_dec = rust_decimal::Decimal::try_from(volume_treated)
        .map_err(|e| format!("Invalid volume_treated value '{}': {}", volume_treated, e))?;
    let ph_dec = ph_level
        .map(|v| rust_decimal::Decimal::try_from(v).map_err(|e| format!("Invalid ph_level value '{}': {}", v, e)))
        .transpose()?;

    let id = sanitary_queries::insert_wastewater_log(
        volume_dec,
        unit.as_deref(),
        treatment_method.as_deref(),
        ph_dec,
        quality_notes.as_deref(),
        session.user_id,
    )
    .await?;

    let _ = write_audit_log(Some(session.user_id), "ADD_WASTEWATER_LOG", Some("wastewater_logs"), Some(id), None, None).await;
    Ok(id.to_string())
}

#[tauri::command]
pub async fn get_wastewater_logs(token: String) -> Result<Vec<serde_json::Value>, AppError> {
    let _session = validate_session_command(&token).await?;

    let rows = sanitary_queries::get_all_wastewater_logs().await?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn submit_division_transfer(
    token: String,
    from_division: String,
    to_division: String,
    reason: Option<String>,
) -> Result<String, AppError> {
    let session = validate_session_command(&token).await?;

    let id = sanitary_queries::insert_division_transfer(
        &from_division,
        &to_division,
        reason.as_deref(),
        session.user_id,
    )
    .await?;

    let _ = write_audit_log(Some(session.user_id), "SUBMIT_DIVISION_TRANSFER", Some("division_transfers"), Some(id), None, Some(serde_json::json!({ "from": from_division, "to": to_division }))).await;
    Ok(id.to_string())
}

#[tauri::command]
pub async fn review_division_transfer(
    token: String,
    request_id: String,
    decision: String,
    notes: Option<String>,
) -> Result<(), AppError> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "head_of_sanitary")?;

    let rid = Uuid::parse_str(&request_id).map_err(|_| "Invalid request ID")?;

    sanitary_queries::update_division_transfer(rid, &decision, session.user_id, notes.as_deref())
        .await?;

    let _ = write_audit_log(Some(session.user_id), "REVIEW_DIVISION_TRANSFER", Some("division_transfers"), Some(rid), None, Some(serde_json::json!({ "decision": decision }))).await;
    Ok(())
}

#[tauri::command]
pub async fn set_division_quota(
    token: String,
    division: String,
    quota_type: String,
    target_value: i32,
    period: Option<String>,
) -> Result<String, AppError> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "head_of_sanitary")?;

    let id = sanitary_queries::insert_division_quota(
        &division,
        &quota_type,
        target_value,
        period.as_deref(),
        session.user_id,
    )
    .await?;

    let _ = write_audit_log(Some(session.user_id), "SET_DIVISION_QUOTA", Some("division_quotas"), Some(id), None, Some(serde_json::json!({ "division": division, "quota_type": quota_type, "target_value": target_value }))).await;
    Ok(id.to_string())
}

#[tauri::command]
pub async fn create_inspection_report(
    token: String,
    location: String,
    inspection_date: String,
    findings: String,
    violations: Option<String>,
    recommendations: Option<String>,
) -> Result<String, AppError> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "sanitary_inspector")?;

    let idate = chrono::NaiveDate::parse_from_str(&inspection_date, "%Y-%m-%d")
        .map_err(|_| "Invalid inspection_date format")?;

    let id = sanitary_queries::insert_inspection_report(
        &location,
        idate,
        &findings,
        violations.as_deref(),
        recommendations.as_deref(),
        session.user_id,
    )
    .await?;

    let _ = write_audit_log(Some(session.user_id), "CREATE_INSPECTION_REPORT", Some("inspection_reports"), Some(id), None, Some(serde_json::json!({ "location": location }))).await;
    Ok(id.to_string())
}

#[tauri::command]
pub async fn get_inspection_reports(token: String) -> Result<Vec<serde_json::Value>, AppError> {
    let _session = validate_session_command(&token).await?;

    let rows = sanitary_queries::get_all_inspection_reports().await?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn send_inspection_to_head(
    token: String,
    report_id: String,
) -> Result<(), AppError> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "sanitary_inspector")?;

    let rid = Uuid::parse_str(&report_id).map_err(|_| "Invalid report ID")?;

    sanitary_queries::send_inspection_to_head(rid, session.user_id).await?;

    let _ = write_audit_log(Some(session.user_id), "SEND_INSPECTION_TO_HEAD", Some("inspection_reports"), Some(rid), None, None).await;
    Ok(())
}
