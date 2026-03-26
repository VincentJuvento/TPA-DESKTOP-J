use crate::db::get_db;
use uuid::Uuid;

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct BlueprintProposalRow {
    pub id: Uuid,
    pub ship_name: String,
    pub blueprint_description: String,
    pub design_specs: Option<String>,
    pub status: String,
    pub vote_id: Option<Uuid>,
    pub ship_id: Option<Uuid>,
    pub submitted_by: Option<Uuid>,
    pub reviewed_by: Option<Uuid>,
    pub review_notes: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub async fn ensure_blueprint_proposals_table() -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS blueprint_proposals (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            ship_name TEXT NOT NULL,
            blueprint_description TEXT NOT NULL,
            design_specs TEXT,
            status TEXT NOT NULL DEFAULT 'pending'
                CHECK (status IN ('pending', 'under_vote', 'approved', 'rejected')),
            vote_id UUID REFERENCES votes(id),
            ship_id UUID REFERENCES ships(id),
            submitted_by UUID REFERENCES users(id),
            reviewed_by UUID REFERENCES users(id),
            review_notes TEXT,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            deleted_at TIMESTAMPTZ
        )"#,
    )
    .execute(get_db())
    .await?;
    Ok(())
}

pub async fn insert_blueprint_proposal(
    ship_name: &str,
    blueprint_description: &str,
    design_specs: Option<&str>,
    ship_id: Option<Uuid>,
    submitted_by: Uuid,
) -> Result<Uuid, sqlx::Error> {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO blueprint_proposals (ship_name, blueprint_description, design_specs, ship_id, submitted_by, status) \
         VALUES ($1, $2, $3, $4, $5, 'pending') RETURNING id",
    )
    .bind(ship_name)
    .bind(blueprint_description)
    .bind(design_specs)
    .bind(ship_id)
    .bind(submitted_by)
    .fetch_one(get_db())
    .await?;
    Ok(row.0)
}

pub async fn link_blueprint_proposal_to_vote(
    proposal_id: Uuid,
    vote_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE blueprint_proposals SET vote_id = $1, status = 'under_vote' WHERE id = $2",
    )
    .bind(vote_id)
    .bind(proposal_id)
    .execute(get_db())
    .await?;
    Ok(())
}

pub async fn get_all_blueprint_proposals() -> Result<Vec<BlueprintProposalRow>, sqlx::Error> {
    sqlx::query_as::<_, BlueprintProposalRow>(
        "SELECT id, ship_name, blueprint_description, design_specs, status, vote_id, ship_id, \
         submitted_by, reviewed_by, review_notes, created_at \
         FROM blueprint_proposals WHERE deleted_at IS NULL ORDER BY created_at DESC",
    )
    .fetch_all(get_db())
    .await
}

pub async fn get_user_blueprint_proposals(
    user_id: Uuid,
) -> Result<Vec<BlueprintProposalRow>, sqlx::Error> {
    sqlx::query_as::<_, BlueprintProposalRow>(
        "SELECT id, ship_name, blueprint_description, design_specs, status, vote_id, ship_id, \
         submitted_by, reviewed_by, review_notes, created_at \
         FROM blueprint_proposals WHERE submitted_by = $1 AND deleted_at IS NULL ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(get_db())
    .await
}

pub async fn get_blueprint_vote_status(
    proposal_id: Uuid,
) -> Result<Option<String>, sqlx::Error> {
    let row: Option<(Option<String>,)> = sqlx::query_as(
        "SELECT v.status FROM blueprint_proposals bp \
         LEFT JOIN votes v ON bp.vote_id = v.id \
         WHERE bp.id = $1",
    )
    .bind(proposal_id)
    .fetch_optional(get_db())
    .await?;
    Ok(row.and_then(|(s,)| s))
}

pub async fn update_blueprint_proposal_review(
    proposal_id: Uuid,
    status: &str,
    reviewed_by: Uuid,
    review_notes: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE blueprint_proposals SET status = $1, reviewed_by = $2, review_notes = $3 \
         WHERE id = $4 AND deleted_at IS NULL",
    )
    .bind(status)
    .bind(reviewed_by)
    .bind(review_notes)
    .bind(proposal_id)
    .execute(get_db())
    .await?;
    Ok(())
}

/// Check whether the given ship has an approved blueprint proposal.
pub async fn ship_has_approved_blueprint(ship_id: Uuid) -> Result<bool, sqlx::Error> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM blueprint_proposals WHERE ship_id = $1 AND status = 'approved' AND deleted_at IS NULL",
    )
    .bind(ship_id)
    .fetch_one(get_db())
    .await?;
    Ok(row.0 > 0)
}
