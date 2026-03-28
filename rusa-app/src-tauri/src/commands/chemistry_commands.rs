use crate::auth::{is_admin, validate_session_command};
use crate::db;
use crate::queries::auth::write_audit_log;
use uuid::Uuid;

#[derive(sqlx::FromRow, serde::Serialize)]
struct MatterRow {
    id: Uuid,
    name: String,
    classification: Option<String>,
    matter_type: Option<String>,
    description: Option<String>,
    properties: Option<String>,
    discovery_experiment_id: Option<Uuid>,
    discovered_by: Option<Uuid>,
    approved_by: Option<Uuid>,
    approved_at: Option<chrono::DateTime<chrono::Utc>>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Retrieve all entries in the matter_archive.
/// Accessible by chemists, the_observer, and administrators.
#[tauri::command]
pub async fn get_matter_archive(token: String) -> Result<Vec<serde_json::Value>, String> {
    let session = validate_session_command(&token).await?;

    // Only chemist-subsystem roles may access the matter archive.
    let allowed = matches!(
        session.role_name.as_str(),
        "chemist" | "physicist" | "the_observer"
    ) || is_admin(&session);

    if !allowed {
        return Err("Access denied: only chemists, physicists, and The Observer may view the matter archive".to_string());
    }

    let rows = sqlx::query_as::<_, MatterRow>(
        "SELECT id, name, classification, matter_type, description, properties, \
         discovery_experiment_id, discovered_by, approved_by, approved_at, created_at \
         FROM matter_archive WHERE deleted_at IS NULL ORDER BY name",
    )
    .fetch_all(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows
        .into_iter()
        .map(|r| serde_json::to_value(r).unwrap_or_default())
        .collect())
}

/// Add a daily log to a chemistry experiment.
/// Enforces that the log is linked to at least one approved test from the test archive —
/// chemistry logs MUST reference a specific test (procedure + expected outcome).
#[tauri::command]
pub async fn add_chemistry_log(
    token: String,
    experiment_id: String,
    log_date: String,
    matter_tested: Option<String>,
    personnel_present: Option<String>,
    linked_test_id: String,
    notes: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;

    if linked_test_id.trim().is_empty() {
        return Err(
            "A linked test from the Test Archive is required for every chemistry log entry".to_string(),
        );
    }

    // Validate the linked_test_id is a valid UUID and the test is approved.
    let tid = Uuid::parse_str(linked_test_id.trim())
        .map_err(|_| "Invalid test ID — must be a valid UUID from the Test Archive".to_string())?;

    let test_ok: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM test_proposals WHERE id = $1 AND status = 'approved' AND deleted_at IS NULL",
    )
    .bind(tid)
    .fetch_optional(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    if test_ok.is_none() {
        return Err(
            "The specified test does not exist in the approved Test Archive".to_string(),
        );
    }

    let eid = Uuid::parse_str(&experiment_id)
        .map_err(|_| "Invalid experiment ID".to_string())?;

    // Ensure this is a chemistry (new_matter) experiment and it is in_progress.
    let exp: Option<(Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT experiment_type, status FROM experiments WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(eid)
    .fetch_optional(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let (exp_type, exp_status) = exp.ok_or_else(|| "Experiment not found".to_string())?;

    if exp_type.as_deref() != Some("new_matter") {
        return Err("This command is only for new_matter chemistry experiments".to_string());
    }

    if exp_status.as_deref() != Some("in_progress") {
        return Err("Experiment must be in_progress to add a log".to_string());
    }

    let ldate = chrono::DateTime::parse_from_rfc3339(&log_date)
        .map(|d| d.with_timezone(&chrono::Utc))
        .map_err(|_| "Invalid log_date format".to_string())?;

    // Store as a JSON array consistent with research_commands::add_experiment_log.
    let linked_json =
        serde_json::to_string(&vec![linked_test_id.trim()]).unwrap_or_default();

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO experiment_logs \
         (experiment_id, log_date, personnel_present, species_matter_tested, \
          linked_test_ids, notes, logged_by) \
         VALUES ($1,$2,$3,$4,$5,$6,$7) RETURNING id",
    )
    .bind(eid)
    .bind(ldate)
    .bind(&personnel_present)
    .bind(&matter_tested)
    .bind(&linked_json)
    .bind(&notes)
    .bind(session.user_id)
    .fetch_one(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(
        Some(session.user_id),
        "ADD_CHEMISTRY_LOG",
        Some("experiment_logs"),
        Some(row.0),
        None,
        Some(serde_json::json!({ "experiment_id": experiment_id, "linked_test_id": linked_test_id })),
    )
    .await;

    Ok(row.0.to_string())
}

/// The Observer approves or rejects the conclusion of a new_matter experiment.
/// On approval the matter is officially added to the matter_archive.
/// The Observer who approved the initial experiment proposal must be the same
/// one who approves the conclusion (continuity check).
#[tauri::command]
pub async fn approve_chemistry_conclusion(
    token: String,
    experiment_id: String,
    decision: String,
    matter_name: String,
    matter_classification: Option<String>,
    matter_type: Option<String>,
    matter_properties: Option<String>,
    review_notes: Option<String>,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;

    if session.role_name != "the_observer" && !is_admin(&session) {
        return Err(
            "Only The Observer can approve chemistry experiment conclusions".to_string(),
        );
    }

    let eid = Uuid::parse_str(&experiment_id)
        .map_err(|_| "Invalid experiment ID".to_string())?;

    let row: Option<(Option<String>, Option<String>, Option<Uuid>, Option<Uuid>)> =
        sqlx::query_as(
            "SELECT experiment_type, status, reviewed_by, proposed_by \
             FROM experiments WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(eid)
        .fetch_optional(db::get_db())
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let (exp_type, status, reviewed_by, proposed_by) =
        row.ok_or_else(|| "Experiment not found".to_string())?;

    if exp_type.as_deref() != Some("new_matter") {
        return Err(
            "This command is only for new_matter chemistry experiments".to_string(),
        );
    }

    if status.as_deref() != Some("conclusion_requested") {
        return Err("Experiment is not awaiting conclusion approval".to_string());
    }

    // Continuity: the director who approved the initial proposal must approve the conclusion.
    if !is_admin(&session) {
        match reviewed_by {
            Some(approver_id) if approver_id != session.user_id => {
                return Err(
                    "Only the director who approved the initial proposal can approve the conclusion"
                        .to_string(),
                );
            }
            None => {
                return Err(
                    "This experiment has no recorded proposal approver; conclusion cannot be approved"
                        .to_string(),
                );
            }
            _ => {}
        }
    }

    match decision.as_str() {
        "approve" => {
            if matter_name.trim().is_empty() {
                return Err("Matter name is required when approving".to_string());
            }

            sqlx::query(
                "UPDATE experiments SET status = 'completed', conclusion_approved = true, \
                 conclusion_approved_by = $1, conclusion_approved_at = NOW(), review_notes = $2 \
                 WHERE id = $3 AND deleted_at IS NULL",
            )
            .bind(session.user_id)
            .bind(&review_notes)
            .bind(eid)
            .execute(db::get_db())
            .await
            .map_err(|e| format!("DB error updating experiment: {}", e))?;

            // Add the newly discovered matter to the archive.
            sqlx::query(
                "INSERT INTO matter_archive \
                 (name, classification, matter_type, properties, \
                  discovery_experiment_id, discovered_by, approved_by, approved_at) \
                 VALUES ($1,$2,$3,$4,$5,$6,$7,NOW())",
            )
            .bind(matter_name.trim())
            .bind(&matter_classification)
            .bind(&matter_type)
            .bind(&matter_properties)
            .bind(eid)
            .bind(proposed_by)
            .bind(session.user_id)
            .execute(db::get_db())
            .await
            .map_err(|e| format!("DB error adding to matter archive: {}", e))?;
        }
        "reject" => {
            sqlx::query(
                "UPDATE experiments SET status = 'in_progress', \
                 conclusion_requested_at = NULL, conclusion_requested_by = NULL, \
                 review_notes = $1 WHERE id = $2 AND deleted_at IS NULL",
            )
            .bind(&review_notes)
            .bind(eid)
            .execute(db::get_db())
            .await
            .map_err(|e| format!("DB error: {}", e))?;
        }
        _ => return Err("Decision must be 'approve' or 'reject'".to_string()),
    }

    let _ = write_audit_log(
        Some(session.user_id),
        "APPROVE_CHEMISTRY_CONCLUSION",
        Some("experiments"),
        Some(eid),
        None,
        Some(serde_json::json!({ "decision": decision, "matter_name": matter_name })),
    )
    .await;

    Ok(())
}

/// Observer dashboard focused on chemistry: active new_matter experiments and pending conclusions.
#[tauri::command]
pub async fn get_chemistry_observer_dashboard(
    token: String,
) -> Result<serde_json::Value, String> {
    let session = validate_session_command(&token).await?;

    if session.role_name != "the_observer" && !is_admin(&session) {
        return Err("Only The Observer can access this dashboard".to_string());
    }

    #[derive(sqlx::FromRow, serde::Serialize)]
    struct ExpRow {
        id: Uuid,
        title: String,
        description: Option<String>,
        experiment_type: Option<String>,
        status: Option<String>,
        proposed_by: Option<Uuid>,
        reviewed_by: Option<Uuid>,
        start_date: Option<chrono::NaiveDate>,
        conclusion_requested_at: Option<chrono::DateTime<chrono::Utc>>,
    }

    let experiments = sqlx::query_as::<_, ExpRow>(
        "SELECT id, title, description, experiment_type, status, proposed_by, reviewed_by, \
         start_date, conclusion_requested_at \
         FROM experiments \
         WHERE experiment_type = 'new_matter' AND deleted_at IS NULL \
           AND status NOT IN ('completed', 'cancelled', 'rejected') \
         ORDER BY created_at DESC",
    )
    .fetch_all(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let pending_conclusions: Vec<_> = experiments
        .iter()
        .filter(|e| e.status.as_deref() == Some("conclusion_requested"))
        .map(|e| serde_json::to_value(e).unwrap_or_default())
        .collect();

    let active: Vec<_> = experiments
        .into_iter()
        .map(|e| serde_json::to_value(e).unwrap_or_default())
        .collect();

    Ok(serde_json::json!({
        "active_experiments": active,
        "pending_conclusions": pending_conclusions,
    }))
}
