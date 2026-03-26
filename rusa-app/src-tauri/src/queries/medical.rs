use crate::db::get_db;
use uuid::Uuid;

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct ShiftRow {
    pub id: Uuid,
    pub staff_id: Option<Uuid>,
    pub shift_start: Option<chrono::DateTime<chrono::Utc>>,
    pub shift_end: Option<chrono::DateTime<chrono::Utc>>,
    pub notes: Option<String>,
    pub allocated_by: Option<Uuid>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct MedicalInventoryRow {
    pub id: Uuid,
    pub item_name: String,
    pub category: Option<String>,
    pub quantity: Option<i32>,
    pub unit: Option<String>,
    pub expiry_date: Option<chrono::NaiveDate>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct PatientRecordRow {
    pub id: Uuid,
    pub patient_id: Option<Uuid>,
    pub diagnosis: Option<String>,
    pub treatment: Option<String>,
    pub medications: Option<String>,
    pub notes: Option<String>,
    pub treated_by: Option<Uuid>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct SpecializationRow {
    pub id: Uuid,
    pub staff_id: Option<Uuid>,
    pub specialization: String,
    pub certified_at: Option<chrono::NaiveDate>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct PatientRow {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct AppointmentRow {
    pub id: Uuid,
    pub patient_id: Option<Uuid>,
    pub scheduled_at: Option<chrono::DateTime<chrono::Utc>>,
    pub status: Option<String>,
    pub findings: Option<String>,
    pub psychiatrist_id: Option<Uuid>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct RecoveryLogRow {
    pub id: Uuid,
    pub patient_id: Option<Uuid>,
    pub entry_date: Option<chrono::NaiveDate>,
    pub status: Option<String>,
    pub notes: Option<String>,
    pub psychiatrist_id: Option<Uuid>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub async fn insert_medical_shift(
    staff_id: Uuid,
    shift_start: chrono::DateTime<chrono::Utc>,
    shift_end: chrono::DateTime<chrono::Utc>,
    notes: Option<&str>,
    allocated_by: Uuid,
) -> Result<Uuid, sqlx::Error> {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO medical_shifts (staff_id, shift_start, shift_end, notes, allocated_by) VALUES ($1,$2,$3,$4,$5) RETURNING id",
    )
    .bind(staff_id)
    .bind(shift_start)
    .bind(shift_end)
    .bind(notes)
    .bind(allocated_by)
    .fetch_one(get_db())
    .await?;
    Ok(row.0)
}

pub async fn get_all_medical_shifts() -> Result<Vec<ShiftRow>, sqlx::Error> {
    sqlx::query_as::<_, ShiftRow>(
        "SELECT id, staff_id, shift_start, shift_end, notes, allocated_by, created_at FROM medical_shifts ORDER BY shift_start DESC",
    )
    .fetch_all(get_db())
    .await
}

pub async fn get_user_medical_shifts(staff_id: Uuid) -> Result<Vec<ShiftRow>, sqlx::Error> {
    sqlx::query_as::<_, ShiftRow>(
        "SELECT id, staff_id, shift_start, shift_end, notes, allocated_by, created_at FROM medical_shifts WHERE staff_id = $1 ORDER BY shift_start DESC",
    )
    .bind(staff_id)
    .fetch_all(get_db())
    .await
}

pub async fn get_all_medical_inventory() -> Result<Vec<MedicalInventoryRow>, sqlx::Error> {
    sqlx::query_as::<_, MedicalInventoryRow>(
        "SELECT id, item_name, category, quantity, unit, expiry_date, updated_at FROM medical_inventory WHERE deleted_at IS NULL ORDER BY item_name",
    )
    .fetch_all(get_db())
    .await
}

pub async fn insert_medical_inventory(
    item_name: &str,
    category: Option<&str>,
    quantity: i32,
    unit: Option<&str>,
    expiry_date: Option<chrono::NaiveDate>,
    updated_by: Uuid,
) -> Result<Uuid, sqlx::Error> {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO medical_inventory (item_name, category, quantity, unit, expiry_date, updated_by) VALUES ($1,$2,$3,$4,$5,$6) RETURNING id",
    )
    .bind(item_name)
    .bind(category)
    .bind(quantity)
    .bind(unit)
    .bind(expiry_date)
    .bind(updated_by)
    .fetch_one(get_db())
    .await?;
    Ok(row.0)
}

pub async fn insert_patient_record(
    patient_id: Uuid,
    diagnosis: Option<&str>,
    treatment: Option<&str>,
    medications: Option<&str>,
    notes: Option<&str>,
    treated_by: Uuid,
) -> Result<Uuid, sqlx::Error> {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO patient_records (patient_id, diagnosis, treatment, medications, notes, treated_by) VALUES ($1,$2,$3,$4,$5,$6) RETURNING id",
    )
    .bind(patient_id)
    .bind(diagnosis)
    .bind(treatment)
    .bind(medications)
    .bind(notes)
    .bind(treated_by)
    .fetch_one(get_db())
    .await?;
    Ok(row.0)
}

pub async fn get_patient_records(patient_id: Uuid) -> Result<Vec<PatientRecordRow>, sqlx::Error> {
    sqlx::query_as::<_, PatientRecordRow>(
        "SELECT id, patient_id, diagnosis, treatment, medications, notes, treated_by, created_at FROM patient_records WHERE patient_id = $1 AND deleted_at IS NULL ORDER BY created_at DESC",
    )
    .bind(patient_id)
    .fetch_all(get_db())
    .await
}

pub async fn insert_staff_specialization(
    staff_id: Uuid,
    specialization: &str,
    certified_at: Option<chrono::NaiveDate>,
) -> Result<Uuid, sqlx::Error> {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO staff_specializations (staff_id, specialization, certified_at) VALUES ($1,$2,$3) RETURNING id",
    )
    .bind(staff_id)
    .bind(specialization)
    .bind(certified_at)
    .fetch_one(get_db())
    .await?;
    Ok(row.0)
}

pub async fn get_all_staff_specializations() -> Result<Vec<SpecializationRow>, sqlx::Error> {
    sqlx::query_as::<_, SpecializationRow>(
        "SELECT id, staff_id, specialization, certified_at, created_at FROM staff_specializations ORDER BY specialization",
    )
    .fetch_all(get_db())
    .await
}

pub async fn insert_psychiatric_patient(patient_id: Uuid) -> Result<Uuid, sqlx::Error> {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO psychiatric_patients (patient_id) VALUES ($1) RETURNING id",
    )
    .bind(patient_id)
    .fetch_one(get_db())
    .await?;
    Ok(row.0)
}

pub async fn get_all_psychiatric_patients() -> Result<Vec<PatientRow>, sqlx::Error> {
    sqlx::query_as::<_, PatientRow>(
        "SELECT id, patient_id, created_at FROM psychiatric_patients WHERE deleted_at IS NULL ORDER BY created_at DESC",
    )
    .fetch_all(get_db())
    .await
}

pub async fn insert_appointment(
    patient_id: Uuid,
    scheduled_at: chrono::DateTime<chrono::Utc>,
    psychiatrist_id: Uuid,
) -> Result<Uuid, sqlx::Error> {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO appointments (patient_id, scheduled_at, status, psychiatrist_id) VALUES ($1,$2,'scheduled',$3) RETURNING id",
    )
    .bind(patient_id)
    .bind(scheduled_at)
    .bind(psychiatrist_id)
    .fetch_one(get_db())
    .await?;
    Ok(row.0)
}

pub async fn get_all_appointments() -> Result<Vec<AppointmentRow>, sqlx::Error> {
    sqlx::query_as::<_, AppointmentRow>(
        "SELECT id, patient_id, scheduled_at, status, findings, psychiatrist_id, created_at FROM appointments WHERE deleted_at IS NULL ORDER BY scheduled_at DESC",
    )
    .fetch_all(get_db())
    .await
}

pub async fn get_patient_appointments(
    patient_id: Uuid,
) -> Result<Vec<AppointmentRow>, sqlx::Error> {
    sqlx::query_as::<_, AppointmentRow>(
        "SELECT id, patient_id, scheduled_at, status, findings, psychiatrist_id, created_at FROM appointments WHERE patient_id = $1 AND deleted_at IS NULL ORDER BY scheduled_at DESC",
    )
    .bind(patient_id)
    .fetch_all(get_db())
    .await
}

pub async fn complete_appointment(
    appointment_id: Uuid,
    findings: Option<&str>,
    psychiatrist_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE appointments SET status = 'completed', findings = $1 WHERE id = $2 AND psychiatrist_id = $3 AND deleted_at IS NULL",
    )
    .bind(findings)
    .bind(appointment_id)
    .bind(psychiatrist_id)
    .execute(get_db())
    .await?;
    Ok(())
}

pub async fn insert_recovery_log(
    patient_id: Uuid,
    psychiatrist_id: Uuid,
    entry_date: chrono::NaiveDate,
    status: &str,
    notes: Option<&str>,
) -> Result<Uuid, sqlx::Error> {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO patient_recovery_log (patient_id, psychiatrist_id, entry_date, status, notes) VALUES ($1,$2,$3,$4,$5) RETURNING id",
    )
    .bind(patient_id)
    .bind(psychiatrist_id)
    .bind(entry_date)
    .bind(status)
    .bind(notes)
    .fetch_one(get_db())
    .await?;
    Ok(row.0)
}

pub async fn get_recovery_log_for_patient(
    patient_id: Uuid,
) -> Result<Vec<RecoveryLogRow>, sqlx::Error> {
    sqlx::query_as::<_, RecoveryLogRow>(
        "SELECT id, patient_id, entry_date, status, notes, psychiatrist_id, created_at FROM patient_recovery_log WHERE patient_id = $1 ORDER BY entry_date DESC",
    )
    .bind(patient_id)
    .fetch_all(get_db())
    .await
}
