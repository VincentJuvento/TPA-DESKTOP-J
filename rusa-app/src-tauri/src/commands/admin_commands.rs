use crate::auth::{permissions, require_role_name, validate_session_command};
use crate::db;
use crate::queries::auth::write_audit_log;
use crate::queries::users::{create_user as query_create_user, soft_delete_user};
use uuid::Uuid;

#[derive(sqlx::FromRow, serde::Serialize)]
struct AuditLogRow {
    id: Uuid,
    user_id: Option<Uuid>,
    action: String,
    table_name: Option<String>,
    record_id: Option<Uuid>,
    old_data: Option<serde_json::Value>,
    new_data: Option<serde_json::Value>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[tauri::command]
pub async fn admin_create_director(
    token: String,
    username: String,
    email: String,
    password: String,
    full_name: String,
    role_name: String,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "the_administrator")?;

    let hash =
        bcrypt::hash(&password, bcrypt::DEFAULT_COST).map_err(|e| format!("Hash error: {}", e))?;

    let user = query_create_user(&username, &email, &hash, &full_name, &role_name, None)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(
        Some(session.user_id),
        "ADMIN_CREATE_DIRECTOR",
        Some("users"),
        Some(user.id),
        None,
        Some(serde_json::json!({ "username": username, "role": role_name })),
    )
    .await;

    Ok(user.id.to_string())
}

#[tauri::command]
pub async fn admin_terminate_director(
    token: String,
    director_id: String,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "the_administrator")?;

    let did = Uuid::parse_str(&director_id).map_err(|_| "Invalid director ID".to_string())?;

    let affected = soft_delete_user(did)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    if !affected {
        return Err("Director not found".to_string());
    }

    let _ = write_audit_log(
        Some(session.user_id),
        "ADMIN_TERMINATE_DIRECTOR",
        Some("users"),
        Some(did),
        None,
        Some(serde_json::json!({ "soft_deleted": true })),
    )
    .await;

    Ok(())
}

#[tauri::command]
pub async fn terminate_personnel(
    token: String,
    user_id: String,
    reason: Option<String>,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;

    let uid = Uuid::parse_str(&user_id).map_err(|_| "Invalid user ID".to_string())?;

    // anchorman can terminate non-directors; administrator can terminate anyone
    if !permissions::has_permission(&session.role_name, "the_administrator")
        && !permissions::has_permission(&session.role_name, "the_anchorman")
    {
        return Err("Only administrator or anchorman can terminate personnel".to_string());
    }

    // If not administrator, ensure target is not a director (tier < 3)
    if !permissions::has_permission(&session.role_name, "the_administrator")
        && permissions::has_permission(&session.role_name, "the_anchorman")
    {
        #[derive(sqlx::FromRow)]
        struct TierCheck {
            tier: i32,
        }
        let check = sqlx::query_as::<_, TierCheck>(
            "SELECT r.tier FROM users u JOIN roles r ON u.role_id = r.id WHERE u.id = $1 AND u.deleted_at IS NULL"
        )
        .bind(uid)
        .fetch_optional(db::get_db())
        .await
        .map_err(|e| format!("DB error: {}", e))?;

        if let Some(c) = check {
            if c.tier >= 3 {
                return Err("Anchorman cannot terminate directors. Use administrator.".to_string());
            }
        }
    }

    let affected = soft_delete_user(uid)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    if !affected {
        return Err("User not found".to_string());
    }

    let _ = write_audit_log(
        Some(session.user_id),
        "TERMINATE_PERSONNEL",
        Some("users"),
        Some(uid),
        None,
        Some(serde_json::json!({ "reason": reason, "terminated_by": session.role_name })),
    )
    .await;

    Ok(())
}

#[tauri::command]
pub async fn get_audit_log(
    token: String,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<serde_json::Value>, String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "the_administrator")?;

    let lim = limit.unwrap_or(100).min(1000);
    let off = offset.unwrap_or(0);

    let rows = sqlx::query_as::<_, AuditLogRow>(
        "SELECT id, user_id, action, table_name, record_id, old_data, new_data, created_at FROM audit_log ORDER BY created_at DESC LIMIT $1 OFFSET $2"
    )
    .bind(lim).bind(off)
    .fetch_all(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn override_vote(
    token: String,
    vote_id: String,
    outcome: String,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "the_administrator")?;

    let vid = Uuid::parse_str(&vote_id).map_err(|_| "Invalid vote ID".to_string())?;

    sqlx::query("UPDATE votes SET status = $1, interrupted_by = $2 WHERE id = $3")
        .bind(&outcome).bind(session.user_id).bind(vid)
        .execute(db::get_db())
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(
        Some(session.user_id),
        "OVERRIDE_VOTE",
        Some("votes"),
        Some(vid),
        None,
        Some(serde_json::json!({ "outcome": outcome })),
    )
    .await;

    Ok(())
}
