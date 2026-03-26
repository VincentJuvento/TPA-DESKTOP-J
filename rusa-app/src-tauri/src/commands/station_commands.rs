use crate::auth::{require_role, validate_session_command};
use crate::db;
use crate::queries::auth::write_audit_log;
use crate::queries::governance as governance_queries;
use uuid::Uuid;

#[derive(sqlx::FromRow, serde::Serialize)]
struct StationRow {
    id: Uuid,
    name: String,
    station_type: Option<String>,
    location: Option<String>,
    status: Option<String>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct StationInventoryRow {
    id: Uuid,
    station_id: Option<Uuid>,
    category: Option<String>,
    item_name: String,
    quantity: Option<i32>,
    unit: Option<String>,
    updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct MapAnnotationRow {
    id: Uuid,
    station_id: Option<Uuid>,
    section_name: String,
    description: Option<String>,
    x_position: Option<rust_decimal::Decimal>,
    y_position: Option<rust_decimal::Decimal>,
    annotated_by: Option<Uuid>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct PersonnelLogRow {
    id: Uuid,
    station_id: Option<Uuid>,
    user_id: Option<Uuid>,
    event_type: Option<String>,
    notes: Option<String>,
    logged_by: Option<Uuid>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct StationFindingRow {
    id: Uuid,
    station_id: Option<Uuid>,
    title: String,
    description: Option<String>,
    is_private: Option<bool>,
    reported_by: Option<Uuid>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct StationSupplyRequestRow {
    id: Uuid,
    station_id: Option<Uuid>,
    title: String,
    items: Option<serde_json::Value>,
    total_cost: Option<rust_decimal::Decimal>,
    status: Option<String>,
    requested_by: Option<Uuid>,
    reviewed_by: Option<Uuid>,
    notes: Option<String>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[tauri::command]
pub async fn get_stations(token: String) -> Result<Vec<serde_json::Value>, String> {
    let _session = validate_session_command(&token).await?;

    let rows = sqlx::query_as::<_, StationRow>(
        "SELECT id, name, station_type, location, status, created_at FROM stations WHERE deleted_at IS NULL ORDER BY name"
    )
    .fetch_all(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn get_station_inventory(
    token: String,
    station_id: String,
) -> Result<Vec<serde_json::Value>, String> {
    let _session = validate_session_command(&token).await?;
    let sid = Uuid::parse_str(&station_id).map_err(|_| "Invalid station ID".to_string())?;

    let rows = sqlx::query_as::<_, StationInventoryRow>(
        "SELECT id, station_id, category, item_name, quantity, unit, updated_at FROM station_inventory WHERE station_id = $1 AND deleted_at IS NULL ORDER BY item_name"
    )
    .bind(sid)
    .fetch_all(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn update_station_inventory(
    token: String,
    station_id: String,
    category: String,
    item_name: String,
    quantity: i32,
    unit: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    let sid = Uuid::parse_str(&station_id).map_err(|_| "Invalid station ID".to_string())?;

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO station_inventory (station_id, category, item_name, quantity, unit, logged_by) VALUES ($1,$2,$3,$4,$5,$6) RETURNING id"
    )
    .bind(sid).bind(&category).bind(&item_name).bind(quantity).bind(&unit).bind(session.user_id)
    .fetch_one(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "UPDATE_STATION_INVENTORY", Some("station_inventory"), Some(row.0), None, Some(serde_json::json!({ "item_name": item_name, "quantity": quantity }))).await;
    Ok(row.0.to_string())
}

#[tauri::command]
pub async fn add_map_annotation(
    token: String,
    station_id: String,
    section_name: String,
    description: Option<String>,
    x_position: Option<f64>,
    y_position: Option<f64>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    let sid = Uuid::parse_str(&station_id).map_err(|_| "Invalid station ID".to_string())?;

    let x_dec = x_position
        .map(|v| rust_decimal::Decimal::try_from(v).map_err(|e| format!("Invalid x_position value '{}': {}", v, e)))
        .transpose()?;
    let y_dec = y_position
        .map(|v| rust_decimal::Decimal::try_from(v).map_err(|e| format!("Invalid y_position value '{}': {}", v, e)))
        .transpose()?;

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO station_map_annotations (station_id, section_name, description, x_position, y_position, annotated_by) VALUES ($1,$2,$3,$4,$5,$6) RETURNING id"
    )
    .bind(sid).bind(&section_name).bind(&description).bind(x_dec).bind(y_dec).bind(session.user_id)
    .fetch_one(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "ADD_MAP_ANNOTATION", Some("station_map_annotations"), Some(row.0), None, Some(serde_json::json!({ "section_name": section_name }))).await;
    Ok(row.0.to_string())
}

#[tauri::command]
pub async fn get_map_annotations(
    token: String,
    station_id: String,
) -> Result<Vec<serde_json::Value>, String> {
    let _session = validate_session_command(&token).await?;
    let sid = Uuid::parse_str(&station_id).map_err(|_| "Invalid station ID".to_string())?;

    let rows = sqlx::query_as::<_, MapAnnotationRow>(
        "SELECT id, station_id, section_name, description, x_position, y_position, annotated_by, created_at FROM station_map_annotations WHERE station_id = $1 AND deleted_at IS NULL ORDER BY section_name"
    )
    .bind(sid)
    .fetch_all(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn log_personnel_event(
    token: String,
    station_id: String,
    user_id: String,
    event_type: String,
    notes: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    let sid = Uuid::parse_str(&station_id).map_err(|_| "Invalid station ID".to_string())?;
    let uid = Uuid::parse_str(&user_id).map_err(|_| "Invalid user ID".to_string())?;

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO station_personnel_log (station_id, user_id, event_type, notes, logged_by) VALUES ($1,$2,$3,$4,$5) RETURNING id"
    )
    .bind(sid).bind(uid).bind(&event_type).bind(&notes).bind(session.user_id)
    .fetch_one(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "LOG_PERSONNEL_EVENT", Some("station_personnel_log"), Some(row.0), None, Some(serde_json::json!({ "event_type": event_type }))).await;
    Ok(row.0.to_string())
}

#[tauri::command]
pub async fn get_personnel_log(
    token: String,
    station_id: String,
) -> Result<Vec<serde_json::Value>, String> {
    let _session = validate_session_command(&token).await?;
    let sid = Uuid::parse_str(&station_id).map_err(|_| "Invalid station ID".to_string())?;

    let rows = sqlx::query_as::<_, PersonnelLogRow>(
        "SELECT id, station_id, user_id, event_type, notes, logged_by, created_at FROM station_personnel_log WHERE station_id = $1 ORDER BY created_at DESC"
    )
    .bind(sid)
    .fetch_all(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn submit_station_findings(
    token: String,
    station_id: String,
    title: String,
    description: Option<String>,
    is_private: bool,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    let sid = Uuid::parse_str(&station_id).map_err(|_| "Invalid station ID".to_string())?;

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO station_findings (station_id, title, description, is_private, reported_by) VALUES ($1,$2,$3,$4,$5) RETURNING id"
    )
    .bind(sid).bind(&title).bind(&description).bind(is_private).bind(session.user_id)
    .fetch_one(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "SUBMIT_STATION_FINDINGS", Some("station_findings"), Some(row.0), None, Some(serde_json::json!({ "title": title }))).await;
    Ok(row.0.to_string())
}

#[tauri::command]
pub async fn get_station_findings(
    token: String,
    station_id: Option<String>,
) -> Result<Vec<serde_json::Value>, String> {
    let session = validate_session_command(&token).await?;

    let rows = if let Some(sid_str) = station_id {
        let sid = Uuid::parse_str(&sid_str).map_err(|_| "Invalid station ID".to_string())?;
        sqlx::query_as::<_, StationFindingRow>(
            "SELECT id, station_id, title, description, is_private, reported_by, created_at FROM station_findings WHERE station_id = $1 AND (is_private = false OR reported_by = $2) AND deleted_at IS NULL ORDER BY created_at DESC"
        ).bind(sid).bind(session.user_id).fetch_all(db::get_db()).await
    } else {
        sqlx::query_as::<_, StationFindingRow>(
            "SELECT id, station_id, title, description, is_private, reported_by, created_at FROM station_findings WHERE (is_private = false OR reported_by = $1) AND deleted_at IS NULL ORDER BY created_at DESC"
        ).bind(session.user_id).fetch_all(db::get_db()).await
    }.map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn get_station_supply_requests(
    token: String,
    station_id: Option<String>,
) -> Result<Vec<serde_json::Value>, String> {
    let _session = validate_session_command(&token).await?;

    let rows = if let Some(sid_str) = station_id {
        let sid = Uuid::parse_str(&sid_str).map_err(|_| "Invalid station ID".to_string())?;
        sqlx::query_as::<_, StationSupplyRequestRow>(
            "SELECT id, station_id, title, items, total_cost, status, requested_by, reviewed_by, notes, created_at FROM station_supply_requests WHERE station_id = $1 AND deleted_at IS NULL ORDER BY created_at DESC"
        ).bind(sid).fetch_all(db::get_db()).await
    } else {
        sqlx::query_as::<_, StationSupplyRequestRow>(
            "SELECT id, station_id, title, items, total_cost, status, requested_by, reviewed_by, notes, created_at FROM station_supply_requests WHERE deleted_at IS NULL ORDER BY created_at DESC"
        ).fetch_all(db::get_db()).await
    }.map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn submit_station_supply_request(
    token: String,
    station_id: String,
    title: String,
    items: Option<String>,
    total_cost: Option<f64>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    let sid = Uuid::parse_str(&station_id).map_err(|_| "Invalid station ID".to_string())?;

    let cost_dec = total_cost
        .map(|v| rust_decimal::Decimal::try_from(v).map_err(|e| format!("Invalid total_cost value '{}': {}", v, e)))
        .transpose()?;
    let items_json = items.as_deref().and_then(|s| {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return None;
        }
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(trimmed) {
            return Some(v);
        }
        let lines = trimmed
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(|l| serde_json::Value::String(l.to_string()))
            .collect::<Vec<_>>();
        Some(serde_json::Value::Array(lines))
    });

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO station_supply_requests (station_id, title, items, total_cost, status, requested_by) VALUES ($1,$2,$3,$4,'pending',$5) RETURNING id"
    )
    .bind(sid).bind(&title).bind(&items_json).bind(cost_dec).bind(session.user_id)
    .fetch_one(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "SUBMIT_STATION_SUPPLY_REQUEST", Some("station_supply_requests"), Some(row.0), None, Some(serde_json::json!({ "title": title }))).await;
    Ok(row.0.to_string())
}

#[tauri::command]
pub async fn review_station_supply_request(
    token: String,
    request_id: String,
    decision: String,
    notes: Option<String>,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    require_role(&session, 3)?;

    let rid = Uuid::parse_str(&request_id).map_err(|_| "Invalid request ID".to_string())?;

    sqlx::query("UPDATE station_supply_requests SET status = $1, reviewed_by = $2, notes = $3 WHERE id = $4 AND deleted_at IS NULL")
        .bind(&decision).bind(session.user_id).bind(&notes).bind(rid)
        .execute(db::get_db())
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "REVIEW_STATION_SUPPLY_REQUEST", Some("station_supply_requests"), Some(rid), None, Some(serde_json::json!({ "decision": decision }))).await;
    Ok(())
}

#[tauri::command]
pub async fn submit_station_abandonment(
    token: String,
    station_id: String,
    reason: String,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    let sid = Uuid::parse_str(&station_id).map_err(|_| "Invalid station ID".to_string())?;

    // Insert the abandonment request
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO station_abandonment_requests (station_id, reason, status, requested_by) VALUES ($1,$2,'pending',$3) RETURNING id"
    )
    .bind(sid).bind(&reason).bind(session.user_id)
    .fetch_one(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let abandonment_id = row.0;

    // Fetch the station name for the vote title
    let station_name: Option<String> = sqlx::query_scalar(
        "SELECT name FROM stations WHERE id = $1"
    )
    .bind(sid)
    .fetch_optional(db::get_db())
    .await
    .ok()
    .flatten();

    let name_label = station_name.as_deref().unwrap_or("Unknown Station");

    // Auto-initiate a director vote for this abandonment request
    let vote_title = format!("Station Abandonment: {}", name_label);
    let vote_desc = format!(
        "Abandonment request submitted by user {}. Reason: {}",
        session.user_id, reason
    );
    let vote_id = governance_queries::insert_vote_typed(
        &vote_title,
        Some(&vote_desc),
        session.user_id,
        "pressing_issue",
    )
    .await
    .map_err(|e| format!("DB error creating vote: {}", e))?;

    // Link the vote back to the abandonment request
    sqlx::query(
        "UPDATE station_abandonment_requests SET vote_id = $1 WHERE id = $2"
    )
    .bind(vote_id)
    .bind(abandonment_id)
    .execute(db::get_db())
    .await
    .map_err(|e| format!("DB error linking vote: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "SUBMIT_STATION_ABANDONMENT", Some("station_abandonment_requests"), Some(abandonment_id), None, Some(serde_json::json!({ "station_id": station_id, "reason": reason, "vote_id": vote_id.to_string() }))).await;
    Ok(abandonment_id.to_string())
}
