pub mod permissions;

use crate::db;
use crate::models::user::SessionData;
use redis::AsyncCommands;

pub async fn validate_session_command(token: &str) -> Result<SessionData, String> {
    let cache_key = format!("session:{}", token);

    // Try Redis cache first
    if let Ok(mut conn) = db::get_redis().get_multiplexed_async_connection().await {
        let cached: Result<Option<String>, _> = conn.get(&cache_key).await;
        if let Ok(Some(json)) = cached {
            if let Ok(session) = serde_json::from_str::<SessionData>(&json) {
                return Ok(session);
            }
        }
    }

    // Fall back to DB
    let user_with_role = crate::queries::auth::validate_session(token)
        .await
        .map_err(|e| format!("DB error: {}", e))?
        .ok_or_else(|| "Invalid or expired session".to_string())?;

    let session = SessionData {
        user_id: user_with_role.id,
        username: user_with_role.username.clone(),
        full_name: user_with_role.full_name.clone(),
        email: user_with_role.email.clone(),
        role_name: user_with_role.role_name.clone(),
        role_display_name: user_with_role.role_display_name.clone(),
        tier: user_with_role.tier,
        subsystem: user_with_role.subsystem.clone(),
        location: user_with_role.location.clone(),
        tel_fax: user_with_role.tel_fax.clone(),
        department: user_with_role.department.clone(),
        department_email: user_with_role.department_email.clone(),
        token: token.to_string(),
        inherited_permissions: permissions::build_inherited_permissions(&user_with_role.role_name),
    };

    // Cache in Redis
    if let Ok(mut conn) = db::get_redis().get_multiplexed_async_connection().await {
        if let Ok(json) = serde_json::to_string(&session) {
            let _ = conn.set_ex::<_, _, ()>(&cache_key, json, 86400u64).await;
        }
    }

    Ok(session)
}

pub fn require_role(session: &SessionData, min_tier: i32) -> Result<(), String> {
    if session.tier >= min_tier {
        Ok(())
    } else {
        Err(format!(
            "Insufficient permissions. Required tier {}, your tier is {}.",
            min_tier, session.tier
        ))
    }
}

pub fn require_role_name(session: &SessionData, role_name: &str) -> Result<(), String> {
    if permissions::has_permission(&session.role_name, role_name) {
        Ok(())
    } else {
        Err(format!("This action requires the '{}' role.", role_name))
    }
}

/// Check if the user holds `required_role` directly or via inherited permissions.
/// Equivalent to `require_role_name` but with a more descriptive name.
pub fn require_permission(session: &SessionData, required_role: &str) -> Result<(), String> {
    require_role_name(session, required_role)
}

/// Rejects galactic security roles from accessing Earth-only features such as
/// the lost &amp; found archive ("if something is lost in space, it stays lost").
pub fn deny_galactic_security(session: &SessionData) -> Result<(), String> {
    if session.role_name == "galactic_security_head"
        || session.role_name == "galactic_security_staff"
    {
        return Err(
            "Galactic Security does not maintain a lost & found system; if something is lost in space, it stays lost."
                .to_string(),
        );
    }
    Ok(())
}

/// Returns `true` if the session belongs to `the_administrator` (tier 4).
pub fn is_admin(session: &SessionData) -> bool {
    session.role_name == "the_administrator" || session.tier >= 4
}

pub async fn invalidate_session_cache(token: &str) {
    let cache_key = format!("session:{}", token);
    if let Ok(mut conn) = db::get_redis().get_multiplexed_async_connection().await {
        let _: Result<(), _> = conn.del(&cache_key).await;
    }
}
