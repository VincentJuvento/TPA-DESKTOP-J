use crate::db::get_db;
use uuid::Uuid;

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct BudgetRequestRow {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub amount: Option<rust_decimal::Decimal>,
    pub items: Option<serde_json::Value>,
    pub status: Option<String>,
    pub requested_by: Option<Uuid>,
    pub accountant_notes: Option<String>,
    pub vote_id: Option<Uuid>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct ExpenditureReportRow {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub total_amount: Option<rust_decimal::Decimal>,
    pub items: Option<serde_json::Value>,
    pub invoice_data: Option<String>,
    pub status: Option<String>,
    pub reported_by: Option<Uuid>,
    pub is_flagged: Option<bool>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct InvestigationRow {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub related_report_id: Option<Uuid>,
    pub status: Option<String>,
    pub submitted_by: Option<Uuid>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub async fn insert_budget_request(
    title: &str,
    description: &str,
    amount: rust_decimal::Decimal,
    items: Option<serde_json::Value>,
    requested_by: Uuid,
) -> Result<Uuid, sqlx::Error> {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO budget_requests (title, description, amount, items, status, requested_by) VALUES ($1,$2,$3,$4,'pending',$5) RETURNING id",
    )
    .bind(title)
    .bind(description)
    .bind(amount)
    .bind(items)
    .bind(requested_by)
    .fetch_one(get_db())
    .await?;
    Ok(row.0)
}

pub async fn insert_expenditure_report(
    title: &str,
    description: &str,
    total_amount: rust_decimal::Decimal,
    items: Option<serde_json::Value>,
    invoice_data: Option<&str>,
    reported_by: Uuid,
) -> Result<Uuid, sqlx::Error> {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO expenditure_reports (title, description, total_amount, items, invoice_data, status, reported_by) VALUES ($1,$2,$3,$4,$5,'pending',$6) RETURNING id",
    )
    .bind(title)
    .bind(description)
    .bind(total_amount)
    .bind(items)
    .bind(invoice_data)
    .bind(reported_by)
    .fetch_one(get_db())
    .await?;
    Ok(row.0)
}

pub async fn get_all_budget_requests() -> Result<Vec<BudgetRequestRow>, sqlx::Error> {
    sqlx::query_as::<_, BudgetRequestRow>(
        "SELECT id, title, description, amount, items, status, requested_by, accountant_notes, vote_id, created_at FROM budget_requests WHERE deleted_at IS NULL ORDER BY created_at DESC",
    )
    .fetch_all(get_db())
    .await
}

pub async fn get_user_budget_requests(user_id: Uuid) -> Result<Vec<BudgetRequestRow>, sqlx::Error> {
    sqlx::query_as::<_, BudgetRequestRow>(
        "SELECT id, title, description, amount, items, status, requested_by, accountant_notes, vote_id, created_at FROM budget_requests WHERE requested_by = $1 AND deleted_at IS NULL ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(get_db())
    .await
}

pub async fn update_budget_request_review(
    request_id: Uuid,
    status: &str,
    notes: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE budget_requests SET status = $1, accountant_notes = $2 WHERE id = $3 AND deleted_at IS NULL",
    )
    .bind(status)
    .bind(notes)
    .bind(request_id)
    .execute(get_db())
    .await?;
    Ok(())
}

pub async fn flag_budget_request(
    request_id: Uuid,
    reason: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE budget_requests SET status = 'flagged', accountant_notes = $1 WHERE id = $2 AND deleted_at IS NULL",
    )
    .bind(reason)
    .bind(request_id)
    .execute(get_db())
    .await?;
    Ok(())
}

pub async fn get_all_expenditure_reports() -> Result<Vec<ExpenditureReportRow>, sqlx::Error> {
    sqlx::query_as::<_, ExpenditureReportRow>(
        "SELECT id, title, description, total_amount, items, invoice_data, status, reported_by, is_flagged, created_at FROM expenditure_reports WHERE deleted_at IS NULL ORDER BY created_at DESC",
    )
    .fetch_all(get_db())
    .await
}

pub async fn get_user_expenditure_reports(
    user_id: Uuid,
) -> Result<Vec<ExpenditureReportRow>, sqlx::Error> {
    sqlx::query_as::<_, ExpenditureReportRow>(
        "SELECT id, title, description, total_amount, items, invoice_data, status, reported_by, is_flagged, created_at FROM expenditure_reports WHERE reported_by = $1 AND deleted_at IS NULL ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(get_db())
    .await
}

pub async fn flag_expenditure_report(
    report_id: Uuid,
    reason: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE expenditure_reports SET is_flagged = true, flag_reason = $1, status = 'flagged_for_investigation' WHERE id = $2 AND deleted_at IS NULL",
    )
    .bind(reason)
    .bind(report_id)
    .execute(get_db())
    .await?;
    Ok(())
}

pub async fn get_budget_request_vote_status(
    request_id: Uuid,
) -> Result<Option<String>, sqlx::Error> {
    let row: Option<(Option<String>,)> = sqlx::query_as(
        "SELECT v.status FROM budget_requests br \
         LEFT JOIN votes v ON br.vote_id = v.id \
         WHERE br.id = $1 AND br.deleted_at IS NULL",
    )
    .bind(request_id)
    .fetch_optional(get_db())
    .await?;
    Ok(row.and_then(|(s,)| s))
}

pub async fn link_budget_request_to_vote(
    request_id: Uuid,
    vote_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE budget_requests SET vote_id = $1, requires_vote = TRUE WHERE id = $2 AND deleted_at IS NULL",
    )
    .bind(vote_id)
    .bind(request_id)
    .execute(get_db())
    .await?;
    Ok(())
}

pub async fn insert_investigation(
    title: &str,
    description: &str,
    related_report_id: Option<Uuid>,
    submitted_by: Uuid,
) -> Result<Uuid, sqlx::Error> {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO budget_investigations (title, description, related_report_id, status, submitted_by) VALUES ($1,$2,$3,'open',$4) RETURNING id",
    )
    .bind(title)
    .bind(description)
    .bind(related_report_id)
    .bind(submitted_by)
    .fetch_one(get_db())
    .await?;
    Ok(row.0)
}
