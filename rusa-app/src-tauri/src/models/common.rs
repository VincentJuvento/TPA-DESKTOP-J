use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuditEntry {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub action: String,
    pub table_name: Option<String>,
    pub record_id: Option<Uuid>,
    pub old_data: Option<serde_json::Value>,
    pub new_data: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self { success: true, data: Some(data), error: None }
    }
    pub fn err(msg: &str) -> ApiResponse<()> {
        ApiResponse { success: false, data: None, error: Some(msg.to_string()) }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

#[derive(Debug, Serialize)]
pub struct AppError {
    pub message: String,
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        eprintln!("Database Error: {}", err);
        AppError { message: "A database error occurred while processing your request.".to_string() }
    }
}

impl From<String> for AppError {
    fn from(message: String) -> Self {
        AppError { message }
    }
}

impl From<&str> for AppError {
    fn from(message: &str) -> Self {
        AppError { message: message.to_string() }
    }
}
