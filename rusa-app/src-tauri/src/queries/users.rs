use crate::db::get_db;
use crate::models::user::{Role, User, UserWithRole};
use uuid::Uuid;

pub async fn list_users() -> Result<Vec<UserWithRole>, sqlx::Error> {
    let rows = sqlx::query_as::<_, UserWithRole>(
        r#"
        SELECT u.id, u.username, u.email, u.full_name, u.role_id,
               r.name as role_name, r.display_name as role_display_name, r.tier, r.subsystem,
               u.location, u.tel_fax, u.department, u.department_email,
               u.is_active, u.created_at
        FROM users u
        JOIN roles r ON u.role_id = r.id
        WHERE u.deleted_at IS NULL
        ORDER BY u.full_name
        "#,
    )
    .fetch_all(get_db())
    .await?;

    Ok(rows)
}

pub async fn get_user_by_id(user_id: Uuid) -> Result<Option<UserWithRole>, sqlx::Error> {
    let row = sqlx::query_as::<_, UserWithRole>(
        r#"
        SELECT u.id, u.username, u.email, u.full_name, u.role_id,
               r.name as role_name, r.display_name as role_display_name, r.tier, r.subsystem,
               u.location, u.tel_fax, u.department, u.department_email,
               u.is_active, u.created_at
        FROM users u
        JOIN roles r ON u.role_id = r.id
        WHERE u.id = $1 AND u.deleted_at IS NULL
        "#,
    )
    .bind(user_id)
    .fetch_optional(get_db())
    .await?;

    Ok(row)
}

pub async fn create_user(
    username: &str,
    email: &str,
    password_hash: &str,
    full_name: &str,
    role_name: &str,
    location: Option<&str>,
) -> Result<User, sqlx::Error> {
    let role = sqlx::query_as::<_, Role>(
        "SELECT id, name, display_name, tier, subsystem, created_at FROM roles WHERE name = $1",
    )
    .bind(role_name)
    .fetch_one(get_db())
    .await?;

    let row = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (username, email, password_hash, full_name, role_id, location)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, username, email, full_name, role_id, location, tel_fax, department, department_email, is_active, created_at, updated_at, deleted_at
        "#,
    )
    .bind(username)
    .bind(email)
    .bind(password_hash)
    .bind(full_name)
    .bind(role.id)
    .bind(location)
    .fetch_one(get_db())
    .await?;

    Ok(row)
}

pub async fn list_roles() -> Result<Vec<Role>, sqlx::Error> {
    let rows = sqlx::query_as::<_, Role>(
        "SELECT id, name, display_name, tier, subsystem, created_at FROM roles ORDER BY tier, name",
    )
    .fetch_all(get_db())
    .await?;

    Ok(rows)
}

pub async fn deactivate_user(user_id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE users SET is_active = false, updated_at = NOW() WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(user_id)
    .execute(get_db())
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn soft_delete_user(user_id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE users SET deleted_at = NOW(), updated_at = NOW() WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(user_id)
    .execute(get_db())
    .await?;

    Ok(result.rows_affected() > 0)
}
