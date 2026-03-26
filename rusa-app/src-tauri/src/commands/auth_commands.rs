use crate::auth::{invalidate_session_cache, permissions, validate_session_command};
use crate::db;
use crate::models::user::SessionData;
use crate::queries::auth::{
    create_session, delete_session, find_user_by_username, write_audit_log,
};
use redis::AsyncCommands;

#[tauri::command]
pub async fn login(username: String, password: String) -> Result<SessionData, String> {
    let (user, hash, role_name, role_display_name, tier, subsystem) =
        find_user_by_username(&username)
            .await
            .map_err(|e| format!("DB error: {}", e))?
            .ok_or_else(|| "Invalid username or password".to_string())?;

    let valid =
        bcrypt::verify(&password, &hash).map_err(|e| format!("Auth error: {}", e))?;
    if !valid {
        return Err("Invalid username or password".to_string());
    }

    let token = create_session(user.id)
        .await
        .map_err(|e| format!("Session error: {}", e))?;

    let session = SessionData {
        user_id: user.id,
        username: user.username.clone(),
        full_name: user.full_name.clone(),
        email: user.email.clone(),
        role_name: role_name.clone(),
        role_display_name: role_display_name.clone(),
        tier,
        subsystem: subsystem.clone(),
        location: user.location.clone(),
        tel_fax: user.tel_fax.clone(),
        department: user.department.clone(),
        department_email: user.department_email.clone(),
        token: token.clone(),
        inherited_permissions: permissions::build_inherited_permissions(&role_name),
    };

    // Cache session in Redis
    if let Ok(mut conn) = db::get_redis().get_multiplexed_async_connection().await {
        if let Ok(json) = serde_json::to_string(&session) {
            let _ = conn
                .set_ex::<_, _, ()>(format!("session:{}", token), json, 86400u64)
                .await;
        }
    }

    let _ = write_audit_log(
        Some(user.id),
        "LOGIN",
        Some("sessions"),
        None,
        None,
        Some(serde_json::json!({ "username": user.username })),
    )
    .await;

    Ok(session)
}

#[tauri::command]
pub async fn logout(token: String) -> Result<(), String> {
    // Validate session first to get user_id for audit
    let session = validate_session_command(&token).await.ok();

    delete_session(&token)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    invalidate_session_cache(&token).await;

    if let Some(s) = session {
        let _ = write_audit_log(
            Some(s.user_id),
            "LOGOUT",
            Some("sessions"),
            None,
            None,
            Some(serde_json::json!({ "username": s.username })),
        )
        .await;
    }

    Ok(())
}

#[tauri::command]
pub async fn get_current_user(token: String) -> Result<SessionData, String> {
    validate_session_command(&token).await
}
