use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct Message {
    pub id: Uuid,
    pub from_user_id: Uuid,
    pub subject: String,
    pub body: String,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub sent_at: Option<DateTime<Utc>>,
    pub recalled_at: Option<DateTime<Utc>>,
    pub is_draft: Option<bool>,
    pub is_broadcast: Option<bool>,
    pub broadcast_sender: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MessageWithSender {
    pub id: Uuid,
    pub from_user_id: Uuid,
    pub from_name: String,
    pub from_username: String,
    pub subject: String,
    pub body: String,
    pub sent_at: Option<DateTime<Utc>>,
    pub recalled_at: Option<DateTime<Utc>>,
    pub is_draft: bool,
    pub recipients: Vec<MessageRecipientInfo>,
    pub read_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct MessageRecipient {
    pub id: Uuid,
    pub message_id: Uuid,
    pub recipient_id: Uuid,
    pub recipient_type: String,
    pub read_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MessageRecipientInfo {
    pub user_id: Uuid,
    pub username: String,
    pub full_name: String,
    pub recipient_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub to: Vec<Uuid>,
    pub cc: Vec<Uuid>,
    pub bcc: Vec<Uuid>,
    pub subject: String,
    pub body: String,
    pub scheduled_at: Option<DateTime<Utc>>,
}
