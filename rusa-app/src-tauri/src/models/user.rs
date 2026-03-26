use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub full_name: String,
    pub role_id: Uuid,
    pub location: Option<String>,
    pub tel_fax: Option<String>,
    pub department: Option<String>,
    pub department_email: Option<String>,
    pub is_active: Option<bool>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct UserWithRole {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub full_name: String,
    pub role_id: Uuid,
    pub role_name: String,
    pub role_display_name: String,
    pub tier: i32,
    pub subsystem: Option<String>,
    pub location: Option<String>,
    pub tel_fax: Option<String>,
    pub department: Option<String>,
    pub department_email: Option<String>,
    pub is_active: Option<bool>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct Role {
    pub id: Uuid,
    pub name: String,
    pub display_name: String,
    pub tier: i32,
    pub subsystem: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub full_name: String,
    pub role_name: String,
    pub location: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionData {
    pub user_id: Uuid,
    pub username: String,
    pub full_name: String,
    pub email: String,
    pub role_name: String,
    pub role_display_name: String,
    pub tier: i32,
    pub subsystem: Option<String>,
    pub location: Option<String>,
    pub tel_fax: Option<String>,
    pub department: Option<String>,
    pub department_email: Option<String>,
    pub token: String,
    /// Role names that this user's role inherits permissions from.
    pub inherited_permissions: Vec<String>,
}
