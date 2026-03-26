use crate::db::get_db;
use uuid::Uuid;

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct GeneralRequestRow {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub request_type: Option<String>,
    pub status: Option<String>,
    pub requires_vote: Option<bool>,
    pub vote_id: Option<Uuid>,
    pub requested_by: Option<Uuid>,
    pub reviewed_by: Option<Uuid>,
    pub review_notes: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub async fn ensure_general_requests_table() -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS general_requests (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            title VARCHAR(300) NOT NULL,
            description TEXT NOT NULL,
            request_type VARCHAR(50) NOT NULL DEFAULT 'general'
                CHECK (request_type IN ('general', 'pressing_issue')),
            status VARCHAR(50) NOT NULL DEFAULT 'pending'
                CHECK (status IN ('pending', 'under_vote', 'approved', 'rejected')),
            requires_vote BOOLEAN NOT NULL DEFAULT TRUE,
            vote_id UUID REFERENCES votes(id),
            requested_by UUID NOT NULL REFERENCES users(id),
            reviewed_by UUID REFERENCES users(id),
            review_notes TEXT,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW(),
            deleted_at TIMESTAMPTZ
        )"#,
    )
    .execute(get_db())
    .await?;
    Ok(())
}

pub async fn insert_general_request(
    title: &str,
    description: &str,
    request_type: &str,
    requested_by: Uuid,
) -> Result<Uuid, sqlx::Error> {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO general_requests (title, description, request_type, status, requires_vote, requested_by) \
         VALUES ($1, $2, $3, 'pending', TRUE, $4) RETURNING id",
    )
    .bind(title)
    .bind(description)
    .bind(request_type)
    .bind(requested_by)
    .fetch_one(get_db())
    .await?;
    Ok(row.0)
}

pub async fn link_general_request_to_vote(
    request_id: Uuid,
    vote_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE general_requests SET vote_id = $1, status = 'under_vote' WHERE id = $2",
    )
    .bind(vote_id)
    .bind(request_id)
    .execute(get_db())
    .await?;
    Ok(())
}

pub async fn get_all_general_requests() -> Result<Vec<GeneralRequestRow>, sqlx::Error> {
    sqlx::query_as::<_, GeneralRequestRow>(
        "SELECT id, title, description, request_type, status, requires_vote, vote_id, \
         requested_by, reviewed_by, review_notes, created_at \
         FROM general_requests WHERE deleted_at IS NULL ORDER BY created_at DESC",
    )
    .fetch_all(get_db())
    .await
}

pub async fn get_user_general_requests(user_id: Uuid) -> Result<Vec<GeneralRequestRow>, sqlx::Error> {
    sqlx::query_as::<_, GeneralRequestRow>(
        "SELECT id, title, description, request_type, status, requires_vote, vote_id, \
         requested_by, reviewed_by, review_notes, created_at \
         FROM general_requests WHERE requested_by = $1 AND deleted_at IS NULL ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(get_db())
    .await
}

pub async fn get_general_request_vote_status(
    request_id: Uuid,
) -> Result<Option<String>, sqlx::Error> {
    let row: Option<(Option<String>,)> = sqlx::query_as(
        "SELECT v.status FROM general_requests gr \
         LEFT JOIN votes v ON gr.vote_id = v.id \
         WHERE gr.id = $1",
    )
    .bind(request_id)
    .fetch_optional(get_db())
    .await?;
    Ok(row.and_then(|(s,)| s))
}

pub async fn update_general_request_review(
    request_id: Uuid,
    status: &str,
    reviewed_by: Uuid,
    notes: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE general_requests SET status = $1, reviewed_by = $2, review_notes = $3, updated_at = NOW() \
         WHERE id = $4 AND deleted_at IS NULL",
    )
    .bind(status)
    .bind(reviewed_by)
    .bind(notes)
    .bind(request_id)
    .execute(get_db())
    .await?;
    Ok(())
}
