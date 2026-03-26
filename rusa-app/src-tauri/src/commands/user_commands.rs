use crate::auth::{require_role, validate_session_command};
use crate::db;
use crate::queries::auth::write_audit_log;
use crate::queries::users::{
    create_user as query_create_user, deactivate_user as query_deactivate,
    list_roles as query_list_roles, list_users as query_list_users,
    soft_delete_user as query_soft_delete,
};
use uuid::Uuid;

#[tauri::command]
pub async fn get_all_users(token: String) -> Result<Vec<serde_json::Value>, String> {
    let session = validate_session_command(&token).await?;
    require_role(&session, 1)?;

    let users = query_list_users()
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let values = users
        .into_iter()
        .map(|u| serde_json::to_value(u).unwrap_or(serde_json::Value::Null))
        .collect();
    Ok(values)
}

#[tauri::command]
pub async fn get_roles(token: String) -> Result<Vec<serde_json::Value>, String> {
    let _session = validate_session_command(&token).await?;

    let roles = query_list_roles()
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let values = roles
        .into_iter()
        .map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null))
        .collect();
    Ok(values)
}

#[tauri::command]
pub async fn create_user(
    token: String,
    username: String,
    email: String,
    password: String,
    full_name: String,
    role_name: String,
    location: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    require_role(&session, 3)?;

    let hash =
        bcrypt::hash(&password, bcrypt::DEFAULT_COST).map_err(|e| format!("Hash error: {}", e))?;

    let user = query_create_user(
        &username,
        &email,
        &hash,
        &full_name,
        &role_name,
        location.as_deref(),
    )
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(
        Some(session.user_id),
        "CREATE_USER",
        Some("users"),
        Some(user.id),
        None,
        Some(serde_json::json!({ "username": username, "role": role_name })),
    )
    .await;

    Ok(user.id.to_string())
}

#[tauri::command]
pub async fn deactivate_user(token: String, user_id: String) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    require_role(&session, 3)?;

    let uid = Uuid::parse_str(&user_id).map_err(|_| "Invalid user ID".to_string())?;

    let affected = query_deactivate(uid)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    if !affected {
        return Err("User not found".to_string());
    }

    let _ = write_audit_log(
        Some(session.user_id),
        "DEACTIVATE_USER",
        Some("users"),
        Some(uid),
        None,
        None,
    )
    .await;

    Ok(())
}

#[tauri::command]
pub async fn delete_user(token: String, user_id: String) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    require_role(&session, 4)?;

    let uid = Uuid::parse_str(&user_id).map_err(|_| "Invalid user ID".to_string())?;

    // Get old data for audit
    let _old = sqlx::query("SELECT username, email FROM users WHERE id = $1")
        .bind(uid)
        .fetch_optional(db::get_db())
        .await
        .ok();

    let affected = query_soft_delete(uid)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    if !affected {
        return Err("User not found".to_string());
    }

    let _ = write_audit_log(
        Some(session.user_id),
        "DELETE_USER",
        Some("users"),
        Some(uid),
        None,
        Some(serde_json::json!({ "soft_deleted": true })),
    )
    .await;

    Ok(())
}
