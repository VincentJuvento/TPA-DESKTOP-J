use crate::db::get_db;
use uuid::Uuid;

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct DataRequestRow {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub data_type: Option<String>,
    pub status: Option<String>,
    pub requested_by: Option<Uuid>,
    pub reviewed_by: Option<Uuid>,
    pub review_notes: Option<String>,
    pub response_data: Option<String>,
    pub responded_by: Option<Uuid>,
    pub analyst_notes: Option<String>,
    pub analyst_rejection_reason: Option<String>,
    pub assigned_to: Option<Uuid>,
    pub delivered_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub requester_location: Option<String>,
    pub requester_tel_fax: Option<String>,
    pub requester_department: Option<String>,
    pub requester_department_email: Option<String>,
    pub submitted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub requested_data_items: Option<String>,
    pub reason_of_request: Option<String>,
    pub requested_by_name: Option<String>,
    pub requested_by_signature: Option<String>,
    pub requested_by_signed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub response_status: Option<String>,
    pub response_explanation: Option<String>,
    pub response_markdown: Option<String>,
    pub response_submitted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub provided_by_names: Option<serde_json::Value>,
    pub delivered_by: Option<Uuid>,
    pub delivered_message_id: Option<Uuid>,
    pub requester_acknowledged_by: Option<Uuid>,
    pub requester_acknowledged_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub async fn insert_data_request(
    title: &str,
    description: &str,
    data_type: Option<&str>,
    requested_data_items: &str,
    reason_of_request: &str,
    requester_location: &str,
    requester_tel_fax: &str,
    requester_department: &str,
    requester_department_email: &str,
    requested_by: Uuid,
    requested_by_name: &str,
    requested_by_signature: &str,
) -> Result<Uuid, sqlx::Error> {
    let row: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO data_requests (
            title, description, data_type, status, requested_by,
            requested_data_items, reason_of_request,
            requester_location, requester_tel_fax, requester_department, requester_department_email,
            requested_by_name, requested_by_signature, requested_by_signed_at, submitted_at
        )
        VALUES ($1,$2,$3,'pending',$4,$5,$6,$7,$8,$9,$10,$11,$12,NOW(),NOW())
        RETURNING id
        "#,
    )
    .bind(title)
    .bind(description)
    .bind(data_type)
    .bind(requested_by)
    .bind(requested_data_items)
    .bind(reason_of_request)
    .bind(requester_location)
    .bind(requester_tel_fax)
    .bind(requester_department)
    .bind(requester_department_email)
    .bind(requested_by_name)
    .bind(requested_by_signature)
    .fetch_one(get_db())
    .await?;
    Ok(row.0)
}

pub async fn get_all_data_requests() -> Result<Vec<DataRequestRow>, sqlx::Error> {
    sqlx::query_as::<_, DataRequestRow>(
        r#"
        SELECT
          id, title, description, data_type, status, requested_by, reviewed_by, review_notes,
          response_data, responded_by, analyst_notes, analyst_rejection_reason, assigned_to,
          delivered_at, created_at,
          requester_location, requester_tel_fax, requester_department, requester_department_email,
          submitted_at, requested_data_items, reason_of_request,
          requested_by_name, requested_by_signature, requested_by_signed_at,
          response_status, response_explanation, response_markdown, response_submitted_at,
          provided_by_names,
          delivered_by, delivered_message_id,
          requester_acknowledged_by, requester_acknowledged_at
        FROM data_requests
        WHERE deleted_at IS NULL
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(get_db())
    .await
}

pub async fn get_analyst_data_requests(user_id: Uuid) -> Result<Vec<DataRequestRow>, sqlx::Error> {
    sqlx::query_as::<_, DataRequestRow>(
        r#"
        SELECT
          id, title, description, data_type, status, requested_by, reviewed_by, review_notes,
          response_data, responded_by, analyst_notes, analyst_rejection_reason, assigned_to,
          delivered_at, created_at,
          requester_location, requester_tel_fax, requester_department, requester_department_email,
          submitted_at, requested_data_items, reason_of_request,
          requested_by_name, requested_by_signature, requested_by_signed_at,
          response_status, response_explanation, response_markdown, response_submitted_at,
          provided_by_names,
          delivered_by, delivered_message_id,
          requester_acknowledged_by, requester_acknowledged_at
        FROM data_requests
        WHERE status IN ('approved', 'processing', 'analyst_submitted', 'analyst_rejected')
          AND (assigned_to IS NULL OR assigned_to = $1)
          AND (data_type IS NULL OR LOWER(data_type) != 'medical')
          AND deleted_at IS NULL
        ORDER BY created_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(get_db())
    .await
}

pub async fn get_user_data_requests(user_id: Uuid) -> Result<Vec<DataRequestRow>, sqlx::Error> {
    sqlx::query_as::<_, DataRequestRow>(
        r#"
        SELECT
          id, title, description, data_type, status, requested_by, reviewed_by, review_notes,
          CASE WHEN status = 'delivered' THEN response_data END as response_data,
          CASE WHEN status = 'delivered' THEN responded_by END as responded_by,
          CASE WHEN status = 'delivered' THEN analyst_notes END as analyst_notes,
          analyst_rejection_reason, assigned_to,
          delivered_at, created_at,
          requester_location, requester_tel_fax, requester_department, requester_department_email,
          submitted_at, requested_data_items, reason_of_request,
          requested_by_name, requested_by_signature, requested_by_signed_at,
          CASE WHEN status = 'delivered' THEN response_status END as response_status,
          CASE WHEN status = 'delivered' THEN response_explanation END as response_explanation,
          CASE WHEN status = 'delivered' THEN response_markdown END as response_markdown,
          CASE WHEN status = 'delivered' THEN response_submitted_at END as response_submitted_at,
          CASE WHEN status = 'delivered' THEN provided_by_names END as provided_by_names,
          delivered_by, delivered_message_id,
          requester_acknowledged_by, requester_acknowledged_at
        FROM data_requests
        WHERE requested_by = $1 AND deleted_at IS NULL
        ORDER BY created_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(get_db())
    .await
}

pub async fn get_data_request_data_type(request_id: Uuid) -> Result<Option<String>, sqlx::Error> {
    let row: Option<(Option<String>,)> = sqlx::query_as(
        "SELECT data_type FROM data_requests WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(request_id)
    .fetch_optional(get_db())
    .await?;
    Ok(row.and_then(|(dt,)| dt))
}

pub async fn update_data_request_review(
    request_id: Uuid,
    status: &str,
    reviewed_by: Uuid,
    notes: Option<&str>,
    assigned_to: Option<Uuid>,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE data_requests SET status = $1, reviewed_by = $2, review_notes = $3, assigned_to = COALESCE($4, assigned_to) WHERE id = $5 AND status = 'pending' AND deleted_at IS NULL",
    )
    .bind(status)
    .bind(reviewed_by)
    .bind(notes)
    .bind(assigned_to)
    .bind(request_id)
    .execute(get_db())
    .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn update_data_request_processing(
    request_id: Uuid,
    user_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE data_requests SET status = 'processing', assigned_to = COALESCE(assigned_to, $1) WHERE id = $2 AND status = 'approved' AND (assigned_to IS NULL OR assigned_to = $1) AND deleted_at IS NULL",
    )
    .bind(user_id)
    .bind(request_id)
    .execute(get_db())
    .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn update_data_request_analyst_response(
    request_id: Uuid,
    response_data: &str,
    analyst_notes: Option<&str>,
    responded_by: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE data_requests SET response_data = $1, analyst_notes = $2, responded_by = $3, status = 'analyst_submitted' WHERE id = $4 AND status IN ('approved', 'processing') AND deleted_at IS NULL",
    )
    .bind(response_data)
    .bind(analyst_notes)
    .bind(responded_by)
    .bind(request_id)
    .execute(get_db())
    .await?;
    Ok(())
}

pub async fn update_data_request_delivered(request_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE data_requests SET status = 'delivered', delivered_at = NOW() WHERE id = $1 AND status = 'analyst_submitted' AND deleted_at IS NULL",
    )
    .bind(request_id)
    .execute(get_db())
    .await?;
    Ok(())
}

pub async fn acknowledge_data_response(request_id: Uuid, user_id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE data_requests
        SET requester_acknowledged_by = $1,
            requester_acknowledged_at = NOW()
        WHERE id = $2
          AND requested_by = $1
          AND status = 'delivered'
          AND requester_acknowledged_at IS NULL
          AND deleted_at IS NULL
        "#,
    )
    .bind(user_id)
    .bind(request_id)
    .execute(get_db())
    .await?;
    Ok(result.rows_affected() > 0)
}

#[derive(sqlx::FromRow)]
pub struct DataResponseDeliveryPayload {
    pub requested_by: Uuid,
    pub title: String,
    pub requested_by_name: String,
    pub requester_department: String,
    pub requester_department_email: String,
    pub response_status: String,
    pub response_explanation: Option<String>,
    pub response_markdown: Option<String>,
    pub response_submitted_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub async fn get_data_response_delivery_payload(
    request_id: Uuid,
) -> Result<Option<DataResponseDeliveryPayload>, sqlx::Error> {
    sqlx::query_as::<_, DataResponseDeliveryPayload>(
        r#"
        SELECT
          requested_by,
          title,
          COALESCE(requested_by_name, '') as requested_by_name,
          COALESCE(requester_department, '') as requester_department,
          COALESCE(requester_department_email, '') as requester_department_email,
          COALESCE(response_status, 'provided') as response_status,
          response_explanation,
          response_markdown,
          response_submitted_at
        FROM data_requests
        WHERE id = $1 AND status = 'analyst_submitted' AND deleted_at IS NULL
        "#,
    )
    .bind(request_id)
    .fetch_optional(get_db())
    .await
}

pub async fn mark_data_response_delivered(
    request_id: Uuid,
    delivered_by: Uuid,
    message_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE data_requests
        SET status = 'delivered',
            final_reviewed_by = $1,
            final_reviewed_at = NOW(),
            delivered_by = $1,
            delivered_at = NOW(),
            delivered_message_id = $2
        WHERE id = $3
          AND status = 'analyst_submitted'
          AND deleted_at IS NULL
        "#,
    )
    .bind(delivered_by)
    .bind(message_id)
    .bind(request_id)
    .execute(get_db())
    .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn update_data_request_analyst_rejection(
    request_id: Uuid,
    rejection_reason: &str,
    responded_by: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE data_requests SET status = 'analyst_rejected', analyst_rejection_reason = $1, responded_by = $2 WHERE id = $3 AND status IN ('approved', 'processing') AND (data_type IS NULL OR LOWER(data_type) != 'medical') AND deleted_at IS NULL",
    )
    .bind(rejection_reason)
    .bind(responded_by)
    .bind(request_id)
    .execute(get_db())
    .await?;
    Ok(())
}

pub async fn get_user_full_names(user_ids: &[Uuid]) -> Result<Vec<String>, sqlx::Error> {
    if user_ids.is_empty() {
        return Ok(vec![]);
    }

    #[derive(sqlx::FromRow)]
    struct Row {
        id: Uuid,
        full_name: String,
    }

    let rows = sqlx::query_as::<_, Row>(
        "SELECT id, full_name FROM users WHERE id = ANY($1::uuid[]) AND deleted_at IS NULL",
    )
    .bind(user_ids)
    .fetch_all(get_db())
    .await?;

    let mut map = std::collections::HashMap::new();
    for r in rows {
        map.insert(r.id, r.full_name);
    }

    Ok(user_ids
        .iter()
        .map(|id| map.get(id).cloned().unwrap_or_else(|| id.to_string()))
        .collect())
}

pub async fn update_data_request_analyst_response_v2(
    request_id: Uuid,
    response_markdown: &str,
    response_status: &str,
    response_explanation: Option<&str>,
    analyst_notes: Option<&str>,
    responded_by: Uuid,
    provided_by: &[Uuid],
    provided_by_names: &[String],
) -> Result<bool, sqlx::Error> {
    let rs = response_status.trim().to_lowercase();
    let rexpl = response_explanation.map(|s| s.trim()).filter(|s| !s.is_empty());

    let mut response_data = if rs == "rejected" {
        let mut out = "REJECTED".to_string();
        if let Some(e) = rexpl {
            out.push_str(": ");
            out.push_str(e);
        }
        out
    } else {
        response_markdown.to_string()
    };
    if response_data.len() > 600 {
        response_data.truncate(600);
        response_data.push_str("…");
    }

    let provided_json = serde_json::Value::Array(
        provided_by
            .iter()
            .map(|u| serde_json::Value::String(u.to_string()))
            .collect(),
    );
    let provided_names_json = serde_json::Value::Array(
        provided_by_names
            .iter()
            .map(|n| serde_json::Value::String(n.clone()))
            .collect(),
    );

    let result = sqlx::query(
        r#"
        UPDATE data_requests
        SET response_markdown = $1,
            response_status = $2,
            response_explanation = $3,
            analyst_notes = $4,
            responded_by = $5,
            assigned_to = COALESCE(assigned_to, $5),
            response_submitted_at = NOW(),
            provided_by = $6,
            provided_by_names = $7,
            response_data = $8,
            status = 'analyst_submitted'
        WHERE id = $9
          AND status IN ('approved', 'processing')
          AND (assigned_to IS NULL OR assigned_to = $5)
          AND deleted_at IS NULL
        "#,
    )
    .bind(response_markdown)
    .bind(&rs)
    .bind(rexpl)
    .bind(analyst_notes)
    .bind(responded_by)
    .bind(provided_json)
    .bind(provided_names_json)
    .bind(response_data)
    .bind(request_id)
    .execute(get_db())
    .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn insert_data_response_attachments(
    request_id: Uuid,
    uploaded_by: Uuid,
    attachments: Vec<(String, Option<String>, Vec<u8>)>,
) -> Result<(), sqlx::Error> {
    for (filename, mime_type, bytes) in attachments {
        sqlx::query(
            r#"
            INSERT INTO data_response_attachments (request_id, uploaded_by, filename, mime_type, bytes)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(request_id)
        .bind(uploaded_by)
        .bind(filename)
        .bind(mime_type)
        .bind(bytes)
        .execute(get_db())
        .await?;
    }
    Ok(())
}

pub async fn get_data_request_owner_status(
    request_id: Uuid,
) -> Result<Option<(Uuid, Option<String>, Option<String>)>, sqlx::Error> {
    let row: Option<(Uuid, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT requested_by, status, data_type FROM data_requests WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(request_id)
    .fetch_optional(get_db())
    .await?;
    Ok(row)
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct DataResponseAttachmentMeta {
    pub id: Uuid,
    pub filename: String,
    pub mime_type: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub async fn list_data_response_attachments(
    request_id: Uuid,
) -> Result<Vec<DataResponseAttachmentMeta>, sqlx::Error> {
    sqlx::query_as::<_, DataResponseAttachmentMeta>(
        r#"
        SELECT id, filename, mime_type, created_at
        FROM data_response_attachments
        WHERE request_id = $1 AND deleted_at IS NULL
        ORDER BY created_at ASC
        "#,
    )
    .bind(request_id)
    .fetch_all(get_db())
    .await
}

#[derive(sqlx::FromRow)]
pub struct DataResponseAttachmentBytes {
    pub request_id: Uuid,
    pub filename: String,
    pub mime_type: Option<String>,
    pub bytes: Vec<u8>,
}

pub async fn get_data_response_attachment(
    attachment_id: Uuid,
) -> Result<Option<DataResponseAttachmentBytes>, sqlx::Error> {
    sqlx::query_as::<_, DataResponseAttachmentBytes>(
        r#"
        SELECT request_id, filename, mime_type, bytes
        FROM data_response_attachments
        WHERE id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(attachment_id)
    .fetch_optional(get_db())
    .await
}
