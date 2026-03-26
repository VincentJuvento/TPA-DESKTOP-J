use crate::auth::validate_session_command;
use crate::db;
use crate::queries::auth::write_audit_log;
use crate::queries::messages::{
    get_inbox as query_inbox, get_message_recipients, get_sent as query_sent,
    mark_as_read, recall_message as query_recall, send_message as query_send,
};
use tauri::Emitter;
use uuid::Uuid;

#[tauri::command]
pub async fn send_message(
    app_handle: tauri::AppHandle,
    token: String,
    to: Vec<String>,
    cc: Vec<String>,
    bcc: Vec<String>,
    subject: String,
    body: String,
    scheduled_at: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;

    let to_uuids: Vec<Uuid> = to
        .iter()
        .filter_map(|s| Uuid::parse_str(s).ok())
        .collect();
    let cc_uuids: Vec<Uuid> = cc
        .iter()
        .filter_map(|s| Uuid::parse_str(s).ok())
        .collect();
    let mut bcc_uuids: Vec<Uuid> = bcc
        .iter()
        .filter_map(|s| Uuid::parse_str(s).ok())
        .collect();

    if to_uuids.is_empty() && cc_uuids.is_empty() && bcc_uuids.is_empty() {
        return Err("No valid recipients. Please select at least one recipient.".to_string());
    }

    let mut all_recips: Vec<Uuid> = Vec::new();
    all_recips.extend(to_uuids.iter().copied());
    all_recips.extend(cc_uuids.iter().copied());
    all_recips.extend(bcc_uuids.iter().copied());
    all_recips.sort();
    all_recips.dedup();

    #[derive(sqlx::FromRow)]
    struct RecipientRoleRow {
        role_name: String,
    }

    let security_roles = ["earth_security_head", "earth_security_staff", "galactic_security_head", "galactic_security_staff"];
    let mut includes_security = false;
    let mut includes_guardian = false;
    if !all_recips.is_empty() {
        let rows: Vec<RecipientRoleRow> = sqlx::query_as::<_, RecipientRoleRow>(
            "SELECT r.name as role_name FROM users u JOIN roles r ON u.role_id = r.id WHERE u.id = ANY($1) AND u.deleted_at IS NULL",
        )
        .bind(&all_recips)
        .fetch_all(db::get_db())
        .await
        .map_err(|e| format!("DB error: {}", e))?;

        includes_security = rows.iter().any(|r| security_roles.contains(&r.role_name.as_str()));
        includes_guardian = rows.iter().any(|r| r.role_name == "the_guardian");
    }

    if session.role_name == "the_overseer" && includes_security {
        return Err("Overseer has read-only access to the Security line and cannot send messages into it".to_string());
    }

    let sender_is_security = security_roles.contains(&session.role_name.as_str());
    let should_copy_overseer =
        (session.role_name == "the_guardian" && includes_security) || (sender_is_security && includes_guardian);

    if should_copy_overseer {
        let overseer: Option<(Uuid,)> = sqlx::query_as(
            "SELECT u.id FROM users u JOIN roles r ON u.role_id = r.id WHERE r.name = 'the_overseer' AND u.deleted_at IS NULL AND u.is_active = true LIMIT 1",
        )
        .fetch_optional(db::get_db())
        .await
        .map_err(|e| format!("DB error: {}", e))?;

        if let Some((oid,)) = overseer {
            let already = to_uuids.contains(&oid) || cc_uuids.contains(&oid) || bcc_uuids.contains(&oid);
            if !already {
                bcc_uuids.push(oid);
            }
        }
    }

    let sched = scheduled_at
        .as_deref()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc));

    let message_id = query_send(
        session.user_id,
        &subject,
        &body,
        sched,
        &to_uuids,
        &cc_uuids,
        &bcc_uuids,
    )
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(
        Some(session.user_id),
        "SEND_MESSAGE",
        Some("messages"),
        Some(message_id),
        None,
        Some(serde_json::json!({ "subject": subject, "to": to })),
    )
    .await;

    let should_emit_now = sched.map(|dt| dt <= chrono::Utc::now()).unwrap_or(true);
    if should_emit_now {
        let payload = serde_json::json!({
            "message_id": message_id,
            "from": session.username,
            "subject": subject,
        });
        let _ = app_handle.emit("new_message", payload);
    }

    Ok(message_id.to_string())
}

#[tauri::command]
pub async fn get_inbox(token: String) -> Result<Vec<serde_json::Value>, String> {
    let session = validate_session_command(&token).await?;
    let messages = query_inbox(session.user_id)
        .await
        .map_err(|e| format!("DB error: {}", e))?;
    let values = messages
        .into_iter()
        .map(|m| serde_json::to_value(m).unwrap_or(serde_json::Value::Null))
        .collect();
    Ok(values)
}

#[tauri::command]
pub async fn get_sent(token: String) -> Result<Vec<serde_json::Value>, String> {
    let session = validate_session_command(&token).await?;
    let messages = query_sent(session.user_id)
        .await
        .map_err(|e| format!("DB error: {}", e))?;
    let values = messages
        .into_iter()
        .map(|m| serde_json::to_value(m).unwrap_or(serde_json::Value::Null))
        .collect();
    Ok(values)
}

#[tauri::command]
pub async fn get_message(
    token: String,
    message_id: String,
) -> Result<serde_json::Value, String> {
    let session = validate_session_command(&token).await?;
    let mid = Uuid::parse_str(&message_id).map_err(|_| "Invalid message ID".to_string())?;

    #[derive(sqlx::FromRow, serde::Serialize)]
    struct MessageRow {
        id: Uuid,
        from_user_id: Uuid,
        from_username: String,
        from_full_name: String,
        subject: String,
        body: String,
        scheduled_at: Option<chrono::DateTime<chrono::Utc>>,
        sent_at: Option<chrono::DateTime<chrono::Utc>>,
        recalled_at: Option<chrono::DateTime<chrono::Utc>>,
        is_draft: Option<bool>,
        is_broadcast: Option<bool>,
        broadcast_sender: Option<String>,
        created_at: Option<chrono::DateTime<chrono::Utc>>,
    }

    let row = sqlx::query_as::<_, MessageRow>(
        r#"
        SELECT m.id, m.from_user_id, u.username as from_username, u.full_name as from_full_name,
               m.subject, m.body, m.scheduled_at, m.sent_at, m.recalled_at, m.is_draft,
               m.is_broadcast, m.broadcast_sender, m.created_at
        FROM messages m
        JOIN users u ON m.from_user_id = u.id
        WHERE m.id = $1 AND m.deleted_at IS NULL
          AND (m.from_user_id = $2
               OR EXISTS (
                   SELECT 1 FROM message_recipients mr
                   WHERE mr.message_id = m.id AND mr.recipient_id = $2 AND mr.deleted_at IS NULL
               ))
        "#,
    )
    .bind(mid)
    .bind(session.user_id)
    .fetch_optional(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?
    .ok_or_else(|| "Message not found or access denied".to_string())?;

    let recipients = get_message_recipients(mid)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let mut value = serde_json::to_value(row).unwrap_or(serde_json::Value::Null);
    if let serde_json::Value::Object(ref mut map) = value {
        map.insert(
            "recipients".to_string(),
            serde_json::to_value(recipients).unwrap_or(serde_json::Value::Array(vec![])),
        );
    }

    Ok(value)
}

#[tauri::command]
pub async fn recall_message(token: String, message_id: String) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    let mid = Uuid::parse_str(&message_id).map_err(|_| "Invalid message ID".to_string())?;

    let recalled = query_recall(mid, session.user_id)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    if !recalled {
        return Err("Message not found, already recalled, or you are not the sender".to_string());
    }

    let _ = write_audit_log(
        Some(session.user_id),
        "RECALL_MESSAGE",
        Some("messages"),
        Some(mid),
        None,
        None,
    )
    .await;

    Ok(())
}

#[tauri::command]
pub async fn mark_message_read(token: String, message_id: String) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    let mid = Uuid::parse_str(&message_id).map_err(|_| "Invalid message ID".to_string())?;

    mark_as_read(mid, session.user_id)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    Ok(())
}
