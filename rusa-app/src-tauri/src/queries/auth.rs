use crate::db::get_db;
use crate::models::user::{User, UserWithRole};
use uuid::Uuid;
use chrono::Utc;

#[derive(sqlx::FromRow)]
struct UserAuthRow {
    id: Uuid,
    username: String,
    email: String,
    password_hash: String,
    full_name: String,
    role_id: Uuid,
    location: Option<String>,
    tel_fax: Option<String>,
    department: Option<String>,
    department_email: Option<String>,
    is_active: Option<bool>,
    created_at: Option<chrono::DateTime<Utc>>,
    updated_at: Option<chrono::DateTime<Utc>>,
    deleted_at: Option<chrono::DateTime<Utc>>,
    role_name: String,
    role_display_name: String,
    tier: i32,
    subsystem: Option<String>,
}

pub async fn find_user_by_username(
    username: &str,
) -> Result<Option<(User, String, String, String, i32, Option<String>)>, sqlx::Error> {
    let row = sqlx::query_as::<_, UserAuthRow>(
        r#"
        SELECT u.id, u.username, u.email, u.password_hash, u.full_name, u.role_id,
               u.location, u.tel_fax, u.department, u.department_email,
               u.is_active, u.created_at, u.updated_at, u.deleted_at,
               r.name as role_name, r.display_name as role_display_name, r.tier, r.subsystem
        FROM users u
        JOIN roles r ON u.role_id = r.id
        WHERE u.username = $1 AND u.deleted_at IS NULL AND u.is_active = true
        "#,
    )
    .bind(username)
    .fetch_optional(get_db())
    .await?;

    if let Some(row) = row {
        let user = User {
            id: row.id,
            username: row.username.clone(),
            email: row.email,
            full_name: row.full_name,
            role_id: row.role_id,
            location: row.location,
            tel_fax: row.tel_fax,
            department: row.department,
            department_email: row.department_email,
            is_active: row.is_active,
            created_at: row.created_at,
            updated_at: row.updated_at,
            deleted_at: row.deleted_at,
        };
        // (user, password_hash, role_name, role_display_name, tier, subsystem)
        Ok(Some((user, row.password_hash, row.role_name, row.role_display_name, row.tier, row.subsystem)))
    } else {
        Ok(None)
    }
}

pub async fn create_session(user_id: Uuid) -> Result<String, sqlx::Error> {
    let token = generate_token();
    let expires_at = Utc::now() + chrono::Duration::hours(24);

    sqlx::query(
        "INSERT INTO sessions (user_id, token, expires_at) VALUES ($1, $2, $3)",
    )
    .bind(user_id)
    .bind(&token)
    .bind(expires_at)
    .execute(get_db())
    .await?;

    Ok(token)
}

pub async fn validate_session(token: &str) -> Result<Option<UserWithRole>, sqlx::Error> {
    let row = sqlx::query_as::<_, UserWithRole>(
        r#"
        SELECT u.id, u.username, u.email, u.full_name, u.role_id,
               r.name as role_name, r.display_name as role_display_name, r.tier, r.subsystem,
               u.location, u.tel_fax, u.department, u.department_email,
               u.is_active, u.created_at
        FROM sessions s
        JOIN users u ON s.user_id = u.id
        JOIN roles r ON u.role_id = r.id
        WHERE s.token = $1 AND s.expires_at > NOW() AND u.deleted_at IS NULL
        "#,
    )
    .bind(token)
    .fetch_optional(get_db())
    .await?;

    Ok(row)
}

pub async fn delete_session(token: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM sessions WHERE token = $1")
        .bind(token)
        .execute(get_db())
        .await?;
    Ok(())
}

pub async fn write_audit_log(
    user_id: Option<Uuid>,
    action: &str,
    table_name: Option<&str>,
    record_id: Option<Uuid>,
    old_data: Option<serde_json::Value>,
    new_data: Option<serde_json::Value>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO audit_log (user_id, action, table_name, record_id, old_data, new_data)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(user_id)
    .bind(action)
    .bind(table_name)
    .bind(record_id)
    .bind(old_data)
    .bind(new_data)
    .execute(get_db())
    .await?;
    Ok(())
}

fn generate_token() -> String {
    use rand::RngCore;
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}
