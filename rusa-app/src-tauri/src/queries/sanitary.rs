use crate::db::get_db;
use uuid::Uuid;

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct SanitaryTaskRow {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub division: Option<String>,
    pub status: Option<String>,
    pub assigned_to: Option<Uuid>,
    pub assigned_by: Option<Uuid>,
    pub due_date: Option<chrono::NaiveDate>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct SanitaryInventoryRow {
    pub id: Uuid,
    pub item_name: String,
    pub category: Option<String>,
    pub quantity: Option<i32>,
    pub unit: Option<String>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct DisposalLogRow {
    pub id: Uuid,
    pub item_name: String,
    pub quantity: Option<rust_decimal::Decimal>,
    pub unit: Option<String>,
    pub disposal_method: Option<String>,
    pub hazard_level: Option<String>,
    pub notes: Option<String>,
    pub logged_by: Option<Uuid>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct WastewaterLogRow {
    pub id: Uuid,
    pub volume_treated: Option<rust_decimal::Decimal>,
    pub unit: Option<String>,
    pub treatment_method: Option<String>,
    pub ph_level: Option<rust_decimal::Decimal>,
    pub quality_notes: Option<String>,
    pub logged_by: Option<Uuid>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct DivisionTransferRow {
    pub id: Uuid,
    pub from_division: String,
    pub to_division: String,
    pub reason: Option<String>,
    pub status: Option<String>,
    pub requested_by: Option<Uuid>,
    pub reviewed_by: Option<Uuid>,
    pub notes: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct InspectionReportRow {
    pub id: Uuid,
    pub location: String,
    pub inspection_date: chrono::NaiveDate,
    pub findings: Option<String>,
    pub violations: Option<String>,
    pub recommendations: Option<String>,
    pub inspector_id: Option<Uuid>,
    pub sent_to_head: Option<bool>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub async fn get_all_sanitary_tasks() -> Result<Vec<SanitaryTaskRow>, sqlx::Error> {
    sqlx::query_as::<_, SanitaryTaskRow>(
        "SELECT id, title, description, division, status, assigned_to, assigned_by, due_date, created_at FROM sanitary_tasks WHERE deleted_at IS NULL ORDER BY created_at DESC",
    )
    .fetch_all(get_db())
    .await
}

pub async fn get_user_sanitary_tasks(user_id: Uuid) -> Result<Vec<SanitaryTaskRow>, sqlx::Error> {
    sqlx::query_as::<_, SanitaryTaskRow>(
        "SELECT id, title, description, division, status, assigned_to, assigned_by, due_date, created_at FROM sanitary_tasks WHERE assigned_to = $1 AND deleted_at IS NULL ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(get_db())
    .await
}

pub async fn insert_sanitary_task(
    title: &str,
    description: Option<&str>,
    division: Option<&str>,
    assigned_to: Uuid,
    assigned_by: Uuid,
    due_date: Option<chrono::NaiveDate>,
) -> Result<Uuid, sqlx::Error> {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO sanitary_tasks (title, description, division, status, assigned_to, assigned_by, due_date) VALUES ($1,$2,$3,'pending',$4,$5,$6) RETURNING id",
    )
    .bind(title)
    .bind(description)
    .bind(division)
    .bind(assigned_to)
    .bind(assigned_by)
    .bind(due_date)
    .fetch_one(get_db())
    .await?;
    Ok(row.0)
}

pub async fn update_sanitary_task_status(
    task_id: Uuid,
    status: &str,
    user_id: Uuid,
    role_name: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE sanitary_tasks SET status = $1 WHERE id = $2 AND (assigned_to = $3 OR $4 = 'head_of_sanitary') AND deleted_at IS NULL",
    )
    .bind(status)
    .bind(task_id)
    .bind(user_id)
    .bind(role_name)
    .execute(get_db())
    .await?;
    Ok(())
}

pub async fn get_all_sanitary_inventory() -> Result<Vec<SanitaryInventoryRow>, sqlx::Error> {
    sqlx::query_as::<_, SanitaryInventoryRow>(
        "SELECT id, item_name, category, quantity, unit, updated_at FROM sanitary_inventory WHERE deleted_at IS NULL ORDER BY item_name",
    )
    .fetch_all(get_db())
    .await
}

pub async fn insert_sanitary_inventory(
    item_name: &str,
    category: Option<&str>,
    quantity: i32,
    unit: Option<&str>,
    updated_by: Uuid,
) -> Result<Uuid, sqlx::Error> {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO sanitary_inventory (item_name, category, quantity, unit, updated_by) VALUES ($1,$2,$3,$4,$5) RETURNING id",
    )
    .bind(item_name)
    .bind(category)
    .bind(quantity)
    .bind(unit)
    .bind(updated_by)
    .fetch_one(get_db())
    .await?;
    Ok(row.0)
}

pub async fn insert_disposal_log(
    item_name: &str,
    quantity: rust_decimal::Decimal,
    unit: Option<&str>,
    disposal_method: Option<&str>,
    hazard_level: Option<&str>,
    notes: Option<&str>,
    logged_by: Uuid,
) -> Result<Uuid, sqlx::Error> {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO disposal_logs (item_name, quantity, unit, disposal_method, hazard_level, notes, logged_by) VALUES ($1,$2,$3,$4,$5,$6,$7) RETURNING id",
    )
    .bind(item_name)
    .bind(quantity)
    .bind(unit)
    .bind(disposal_method)
    .bind(hazard_level)
    .bind(notes)
    .bind(logged_by)
    .fetch_one(get_db())
    .await?;
    Ok(row.0)
}

pub async fn get_all_disposal_logs() -> Result<Vec<DisposalLogRow>, sqlx::Error> {
    sqlx::query_as::<_, DisposalLogRow>(
        "SELECT id, item_name, quantity, unit, disposal_method, hazard_level, notes, logged_by, created_at FROM disposal_logs ORDER BY created_at DESC",
    )
    .fetch_all(get_db())
    .await
}

pub async fn insert_wastewater_log(
    volume_treated: rust_decimal::Decimal,
    unit: Option<&str>,
    treatment_method: Option<&str>,
    ph_level: Option<rust_decimal::Decimal>,
    quality_notes: Option<&str>,
    logged_by: Uuid,
) -> Result<Uuid, sqlx::Error> {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO wastewater_logs (volume_treated, unit, treatment_method, ph_level, quality_notes, logged_by) VALUES ($1,$2,$3,$4,$5,$6) RETURNING id",
    )
    .bind(volume_treated)
    .bind(unit)
    .bind(treatment_method)
    .bind(ph_level)
    .bind(quality_notes)
    .bind(logged_by)
    .fetch_one(get_db())
    .await?;
    Ok(row.0)
}

pub async fn get_all_wastewater_logs() -> Result<Vec<WastewaterLogRow>, sqlx::Error> {
    sqlx::query_as::<_, WastewaterLogRow>(
        "SELECT id, volume_treated, unit, treatment_method, ph_level, quality_notes, logged_by, created_at FROM wastewater_logs ORDER BY created_at DESC",
    )
    .fetch_all(get_db())
    .await
}

pub async fn insert_division_transfer(
    from_division: &str,
    to_division: &str,
    reason: Option<&str>,
    requested_by: Uuid,
) -> Result<Uuid, sqlx::Error> {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO division_transfers (from_division, to_division, reason, status, requested_by) VALUES ($1,$2,$3,'pending',$4) RETURNING id",
    )
    .bind(from_division)
    .bind(to_division)
    .bind(reason)
    .bind(requested_by)
    .fetch_one(get_db())
    .await?;
    Ok(row.0)
}

pub async fn update_division_transfer(
    request_id: Uuid,
    decision: &str,
    reviewed_by: Uuid,
    notes: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE division_transfers SET status = $1, reviewed_by = $2, notes = $3 WHERE id = $4",
    )
    .bind(decision)
    .bind(reviewed_by)
    .bind(notes)
    .bind(request_id)
    .execute(get_db())
    .await?;
    Ok(())
}

pub async fn insert_division_quota(
    division: &str,
    quota_type: &str,
    target_value: i32,
    period: Option<&str>,
    set_by: Uuid,
) -> Result<Uuid, sqlx::Error> {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO division_quotas (division, quota_type, target_value, period, set_by) VALUES ($1,$2,$3,$4,$5) RETURNING id",
    )
    .bind(division)
    .bind(quota_type)
    .bind(target_value)
    .bind(period)
    .bind(set_by)
    .fetch_one(get_db())
    .await?;
    Ok(row.0)
}

pub async fn insert_inspection_report(
    location: &str,
    inspection_date: chrono::NaiveDate,
    findings: &str,
    violations: Option<&str>,
    recommendations: Option<&str>,
    inspector_id: Uuid,
) -> Result<Uuid, sqlx::Error> {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO inspection_reports (location, inspection_date, findings, violations, recommendations, inspector_id) VALUES ($1,$2,$3,$4,$5,$6) RETURNING id",
    )
    .bind(location)
    .bind(inspection_date)
    .bind(findings)
    .bind(violations)
    .bind(recommendations)
    .bind(inspector_id)
    .fetch_one(get_db())
    .await?;
    Ok(row.0)
}

pub async fn get_all_inspection_reports() -> Result<Vec<InspectionReportRow>, sqlx::Error> {
    sqlx::query_as::<_, InspectionReportRow>(
        "SELECT id, location, inspection_date, findings, violations, recommendations, inspector_id, sent_to_head, created_at FROM inspection_reports WHERE deleted_at IS NULL ORDER BY inspection_date DESC",
    )
    .fetch_all(get_db())
    .await
}

pub async fn send_inspection_to_head(
    report_id: Uuid,
    inspector_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE inspection_reports SET sent_to_head = true WHERE id = $1 AND inspector_id = $2",
    )
    .bind(report_id)
    .bind(inspector_id)
    .execute(get_db())
    .await?;
    Ok(())
}
