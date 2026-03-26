use crate::db::get_db;
use crate::models::message::{Message, MessageRecipient};
use uuid::Uuid;

pub async fn get_inbox(user_id: Uuid) -> Result<Vec<Message>, sqlx::Error> {
    let rows = sqlx::query_as::<_, Message>(
        r#"
        SELECT m.id, m.from_user_id, m.subject, m.body, m.scheduled_at, m.sent_at,
               m.recalled_at, m.is_draft, m.is_broadcast, m.broadcast_sender, m.created_at, m.deleted_at
        FROM messages m
        JOIN message_recipients mr ON m.id = mr.message_id
        WHERE mr.recipient_id = $1
          AND mr.deleted_at IS NULL
          AND m.deleted_at IS NULL
          AND m.is_draft = false
          AND m.recalled_at IS NULL
          AND m.sent_at <= NOW()
        ORDER BY m.sent_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(get_db())
    .await?;

    Ok(rows)
}

pub async fn get_sent(user_id: Uuid) -> Result<Vec<Message>, sqlx::Error> {
    let rows = sqlx::query_as::<_, Message>(
        r#"
        SELECT id, from_user_id, subject, body, scheduled_at, sent_at,
               recalled_at, is_draft, is_broadcast, broadcast_sender, created_at, deleted_at
        FROM messages
        WHERE from_user_id = $1
          AND deleted_at IS NULL
          AND is_draft = false
        ORDER BY sent_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(get_db())
    .await?;

    Ok(rows)
}

pub async fn get_drafts(user_id: Uuid) -> Result<Vec<Message>, sqlx::Error> {
    let rows = sqlx::query_as::<_, Message>(
        r#"
        SELECT id, from_user_id, subject, body, scheduled_at, sent_at,
               recalled_at, is_draft, is_broadcast, broadcast_sender, created_at, deleted_at
        FROM messages
        WHERE from_user_id = $1
          AND deleted_at IS NULL
          AND is_draft = true
        ORDER BY created_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(get_db())
    .await?;

    Ok(rows)
}

pub async fn send_message(
    from_user_id: Uuid,
    subject: &str,
    body: &str,
    scheduled_at: Option<chrono::DateTime<chrono::Utc>>,
    to: &[Uuid],
    cc: &[Uuid],
    bcc: &[Uuid],
) -> Result<Uuid, sqlx::Error> {
    let row: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO messages (from_user_id, subject, body, scheduled_at, sent_at, notified_at, is_draft)
        VALUES (
          $1,
          $2,
          $3,
          $4,
          COALESCE($4, NOW()),
          CASE WHEN $4 IS NULL OR $4 <= NOW() THEN NOW() ELSE NULL END,
          false
        )
        RETURNING id
        "#,
    )
    .bind(from_user_id)
    .bind(subject)
    .bind(body)
    .bind(scheduled_at)
    .fetch_one(get_db())
    .await?;

    let message_id = row.0;

    // Batch insert all recipients in one query using UNNEST
    let mut ids = Vec::new();
    let mut types = Vec::new();
    for &r in to { ids.push(r); types.push("to"); }
    for &r in cc { ids.push(r); types.push("cc"); }
    for &r in bcc { ids.push(r); types.push("bcc"); }

    if !ids.is_empty() {
        let message_ids = vec![message_id; ids.len()];
        sqlx::query(
            r#"
            INSERT INTO message_recipients (message_id, recipient_id, recipient_type)
            SELECT * FROM UNNEST($1::uuid[], $2::uuid[], $3::text[])
            "#,
        )
        .bind(&message_ids)
        .bind(&ids)
        .bind(&types)
        .execute(get_db())
        .await?;
    }

    Ok(message_id)
}

pub async fn recall_message(message_id: Uuid, from_user_id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE messages SET recalled_at = NOW()
        WHERE id = $1 AND from_user_id = $2 AND recalled_at IS NULL AND is_draft = false
        "#,
    )
    .bind(message_id)
    .bind(from_user_id)
    .execute(get_db())
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn mark_as_read(message_id: Uuid, recipient_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE message_recipients SET read_at = NOW()
        WHERE message_id = $1 AND recipient_id = $2 AND read_at IS NULL
        "#,
    )
    .bind(message_id)
    .bind(recipient_id)
    .execute(get_db())
    .await?;
    Ok(())
}

pub async fn get_message_recipients(
    message_id: Uuid,
) -> Result<Vec<MessageRecipient>, sqlx::Error> {
    let rows = sqlx::query_as::<_, MessageRecipient>(
        r#"
        SELECT id, message_id, recipient_id, recipient_type, read_at, deleted_at
        FROM message_recipients
        WHERE message_id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(message_id)
    .fetch_all(get_db())
    .await?;

    Ok(rows)
}
