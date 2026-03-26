use crate::auth::{permissions, require_role, require_role_name, validate_session_command};
use crate::queries::auth::write_audit_log;
use crate::queries::governance as governance_queries;
use uuid::Uuid;

#[tauri::command]
pub async fn initiate_vote(
    token: String,
    title: String,
    description: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    require_role(&session, 3)?;

    let id = governance_queries::insert_vote(&title, description.as_deref(), session.user_id)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "INITIATE_VOTE", Some("votes"), Some(id), None, Some(serde_json::json!({ "title": title }))).await;
    Ok(id.to_string())
}

#[tauri::command]
pub async fn cast_vote(
    token: String,
    vote_id: String,
    decision: String,
    reason: String,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    // Directors only vote
    if session.tier < 3 {
        return Err("Only directors can cast votes".to_string());
    }

    if reason.trim().is_empty() {
        return Err("A reason is required when casting a vote".to_string());
    }

    let vid = Uuid::parse_str(&vote_id).map_err(|_| "Invalid vote ID".to_string())?;

    // Check that the vote is still open and within the current window
    let vote_check = governance_queries::check_vote_status(vid)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let (vstatus, vdeadline) = vote_check.ok_or_else(|| "Vote not found".to_string())?;
    if vstatus != "open" {
        return Err(format!("Vote is no longer open (status: {})", vstatus));
    }
    if let Some(dl) = vdeadline {
        if chrono::Utc::now() > dl {
            return Err("Voting window has expired".to_string());
        }
    }

    // Upsert: allow switching within the time window
    governance_queries::upsert_vote_ballot(vid, session.user_id, &decision, &reason)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    // Try to resolve immediately if quorum (8 directors) is now met
    governance_queries::try_resolve_vote_on_quorum(vid)
        .await
        .ok(); // best-effort — don't fail the cast on resolution error

    let _ = write_audit_log(Some(session.user_id), "CAST_VOTE", Some("vote_ballots"), Some(vid), None, Some(serde_json::json!({ "vote_id": vote_id, "decision": decision, "reason": reason }))).await;
    Ok(())
}

#[tauri::command]
pub async fn get_votes(token: String) -> Result<Vec<serde_json::Value>, String> {
    let _session = validate_session_command(&token).await?;

    // Lazily resolve expired votes
    governance_queries::resolve_expired_votes().await.ok(); // best-effort, don't fail on error

    let rows = governance_queries::get_all_votes()
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn get_vote_details(
    token: String,
    vote_id: String,
) -> Result<serde_json::Value, String> {
    let _session = validate_session_command(&token).await?;
    let vid = Uuid::parse_str(&vote_id).map_err(|_| "Invalid vote ID".to_string())?;

    let row = governance_queries::get_vote_details(vid)
        .await
        .map_err(|e| format!("DB error: {}", e))?
        .ok_or_else(|| "Vote not found".to_string())?;

    Ok(serde_json::to_value(row).unwrap_or_default())
}

#[tauri::command]
pub async fn interrupt_vote(token: String, vote_id: String) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "the_administrator")?;

    let vid = Uuid::parse_str(&vote_id).map_err(|_| "Invalid vote ID".to_string())?;

    governance_queries::interrupt_vote(vid, session.user_id)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "INTERRUPT_VOTE", Some("votes"), Some(vid), None, None).await;
    Ok(())
}

#[tauri::command]
pub async fn create_meeting(
    token: String,
    title: String,
    description: Option<String>,
    scheduled_at: String,
    location: Option<String>,
    attendee_ids: Vec<String>,
    to_ids: Vec<String>,
    cc_ids: Vec<String>,
    bcc_ids: Vec<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    require_role(&session, 3)?;

    if to_ids.is_empty() {
        return Err("At least one To recipient is required".to_string());
    }

    let sched = chrono::DateTime::parse_from_rfc3339(&scheduled_at)
        .map(|d| d.with_timezone(&chrono::Utc))
        .map_err(|_| "Invalid scheduled_at format".to_string())?;

    let to_json = serde_json::to_value(&to_ids).unwrap_or(serde_json::json!([]));
    let cc_json = serde_json::to_value(&cc_ids).unwrap_or(serde_json::json!([]));
    let bcc_json = serde_json::to_value(&bcc_ids).unwrap_or(serde_json::json!([]));

    let meeting_id = governance_queries::insert_meeting(
        &title,
        description.as_deref(),
        sched,
        location.as_deref(),
        session.user_id,
        &to_json,
        &cc_json,
        &bcc_json,
    )
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    // Insert attendees from all recipient lists (to + cc + bcc) as well as explicit attendee_ids
    let all_attendee_ids: Vec<String> = attendee_ids.iter()
        .chain(to_ids.iter())
        .chain(cc_ids.iter())
        .chain(bcc_ids.iter())
        .cloned()
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    for aid_str in &all_attendee_ids {
        if let Ok(aid) = Uuid::parse_str(aid_str) {
            let _ = governance_queries::insert_meeting_attendee(meeting_id, aid).await;
        }
    }

    let _ = write_audit_log(Some(session.user_id), "CREATE_MEETING", Some("meetings"), Some(meeting_id), None, Some(serde_json::json!({ "title": title, "to": to_ids, "cc": cc_ids, "bcc": bcc_ids }))).await;
    Ok(meeting_id.to_string())
}

#[tauri::command]
pub async fn get_meetings(token: String) -> Result<Vec<serde_json::Value>, String> {
    let _session = validate_session_command(&token).await?;

    let rows = governance_queries::get_all_meetings()
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn relocate_staff(
    token: String,
    staff_id: String,
    to_location: String,
    relocation_type: String,
    start_date: Option<String>,
    end_date: Option<String>,
    reason: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    if session.role_name != "the_nomad" && session.role_name != "the_administrator" && session.role_name != "the_overseer" {
        return Err("Only nomad, overseer, or administrator can relocate staff".to_string());
    }

    let sid = Uuid::parse_str(&staff_id).map_err(|_| "Invalid staff ID".to_string())?;
    let sdate = start_date.as_deref()
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());
    let edate = end_date.as_deref()
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());

    let id = governance_queries::insert_staff_relocation(
        sid,
        &to_location,
        &relocation_type,
        sdate,
        edate,
        reason.as_deref(),
        session.user_id,
    )
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "RELOCATE_STAFF", Some("staff_relocations"), Some(id), None, Some(serde_json::json!({ "staff_id": staff_id, "to_location": to_location }))).await;
    Ok(id.to_string())
}

#[tauri::command]
pub async fn get_relocations(token: String) -> Result<Vec<serde_json::Value>, String> {
    let _session = validate_session_command(&token).await?;

    let rows = governance_queries::get_all_relocations()
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn set_archive_permission(
    token: String,
    table_name: String,
    record_id: String,
    access_level: String,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "the_librarian")?;

    let rid = Uuid::parse_str(&record_id).map_err(|_| "Invalid record ID".to_string())?;

    governance_queries::upsert_archive_permission(&table_name, rid, &access_level, session.user_id)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "SET_ARCHIVE_PERMISSION", Some("archive_permissions"), Some(rid), None, Some(serde_json::json!({ "table_name": table_name, "access_level": access_level }))).await;
    Ok(())
}

#[tauri::command]
pub async fn log_event_document(
    token: String,
    title: String,
    description: Option<String>,
    event_date: Option<String>,
    venue: Option<String>,
    venue_invoice: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    // coordinator role (excluded but still seeded)
    require_role_name(&session, "the_coordinator")?;

    let edate = event_date.as_deref()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.with_timezone(&chrono::Utc));

    let id = governance_queries::insert_event_document(
        &title,
        description.as_deref(),
        edate,
        venue.as_deref(),
        venue_invoice.as_deref(),
        session.user_id,
    )
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "LOG_EVENT_DOCUMENT", Some("event_documents"), Some(id), None, Some(serde_json::json!({ "title": title }))).await;
    Ok(id.to_string())
}

#[tauri::command]
pub async fn get_event_documents(token: String) -> Result<Vec<serde_json::Value>, String> {
    let _session = validate_session_command(&token).await?;

    let rows = governance_queries::get_all_event_documents()
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

// ── Nomad Task Assignment ─────────────────────────────────────────────────────

#[derive(sqlx::FromRow, serde::Serialize)]
struct NomadTaskRow {
    id: uuid::Uuid,
    title: String,
    description: Option<String>,
    assigned_to: Option<uuid::Uuid>,
    assigned_by: Option<uuid::Uuid>,
    target_role: Option<String>,
    status: String,
    due_date: Option<chrono::DateTime<chrono::Utc>>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

async fn ensure_nomad_tasks_table() -> Result<(), String> {
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS nomad_tasks (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            title TEXT NOT NULL,
            description TEXT,
            assigned_to UUID REFERENCES users(id),
            assigned_by UUID REFERENCES users(id),
            target_role TEXT,
            status TEXT NOT NULL DEFAULT 'pending',
            due_date TIMESTAMPTZ,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            deleted_at TIMESTAMPTZ
        )"#,
    )
    .execute(crate::db::get_db())
    .await
    .map_err(|e| format!("DB error creating table: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn nomad_assign_task(
    token: String,
    assigned_to: String,
    title: String,
    description: Option<String>,
    due_date: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "the_nomad")?;

    ensure_nomad_tasks_table().await?;

    let atid = Uuid::parse_str(&assigned_to).map_err(|_| "Invalid user ID".to_string())?;

    // Verify target is settler_commander or head_of_sanitary
    #[derive(sqlx::FromRow)]
    struct RoleCheck { role_name: String }
    let check = sqlx::query_as::<_, RoleCheck>(
        "SELECT r.name as role_name FROM users u JOIN roles r ON u.role_id = r.id WHERE u.id = $1 AND u.deleted_at IS NULL"
    )
    .bind(atid)
    .fetch_optional(crate::db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let role_name = check
        .ok_or_else(|| "User not found".to_string())?
        .role_name;

    if role_name != "settler_commander" && role_name != "head_of_sanitary" {
        return Err("Nomad can only assign tasks to settler_commander or head_of_sanitary".to_string());
    }

    let dd = due_date
        .as_deref()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.with_timezone(&chrono::Utc));

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO nomad_tasks (title, description, assigned_to, assigned_by, target_role, status, due_date) VALUES ($1,$2,$3,$4,$5,'pending',$6) RETURNING id"
    )
    .bind(&title)
    .bind(&description)
    .bind(atid)
    .bind(session.user_id)
    .bind(&role_name)
    .bind(dd)
    .fetch_one(crate::db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(
        Some(session.user_id),
        "NOMAD_ASSIGN_TASK",
        Some("nomad_tasks"),
        Some(row.0),
        None,
        Some(serde_json::json!({ "title": title, "assigned_to": assigned_to, "target_role": role_name })),
    )
    .await;

    Ok(row.0.to_string())
}

#[tauri::command]
pub async fn get_nomad_tasks(token: String) -> Result<Vec<serde_json::Value>, String> {
    let session = validate_session_command(&token).await?;

    ensure_nomad_tasks_table().await?;

    let rows = if permissions::has_permission(&session.role_name, "the_nomad") {
        sqlx::query_as::<_, NomadTaskRow>(
            "SELECT id, title, description, assigned_to, assigned_by, target_role, status, due_date, created_at FROM nomad_tasks WHERE assigned_by = $1 AND deleted_at IS NULL ORDER BY created_at DESC"
        )
        .bind(session.user_id)
        .fetch_all(crate::db::get_db())
        .await
        .map_err(|e| format!("DB error: {}", e))?
    } else {
        sqlx::query_as::<_, NomadTaskRow>(
            "SELECT id, title, description, assigned_to, assigned_by, target_role, status, due_date, created_at FROM nomad_tasks WHERE assigned_to = $1 AND deleted_at IS NULL ORDER BY created_at DESC"
        )
        .bind(session.user_id)
        .fetch_all(crate::db::get_db())
        .await
        .map_err(|e| format!("DB error: {}", e))?
    };

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn update_nomad_task_status(
    token: String,
    task_id: String,
    status: String,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;

    ensure_nomad_tasks_table().await?;

    let tid = Uuid::parse_str(&task_id).map_err(|_| "Invalid task ID".to_string())?;

    sqlx::query("UPDATE nomad_tasks SET status = $1 WHERE id = $2 AND deleted_at IS NULL")
        .bind(&status)
        .bind(tid)
        .execute(crate::db::get_db())
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(
        Some(session.user_id),
        "UPDATE_NOMAD_TASK_STATUS",
        Some("nomad_tasks"),
        Some(tid),
        None,
        Some(serde_json::json!({ "status": status })),
    )
    .await;

    Ok(())
}

// ── Librarian: Redact / Delete ────────────────────────────────────────────────

#[tauri::command]
pub async fn redact_record(
    token: String,
    table_name: String,
    record_id: String,
    redaction_reason: Option<String>,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "the_librarian")?;

    let rid = Uuid::parse_str(&record_id).map_err(|_| "Invalid record ID".to_string())?;

    // Store a redaction entry in archive_permissions with access_level = 'redacted'
    governance_queries::upsert_archive_permission(&table_name, rid, "redacted", session.user_id)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(
        Some(session.user_id),
        "REDACT_RECORD",
        Some("archive_permissions"),
        Some(rid),
        None,
        Some(serde_json::json!({ "table_name": table_name, "reason": redaction_reason })),
    )
    .await;

    Ok(())
}

#[tauri::command]
pub async fn librarian_delete_record(
    token: String,
    table_name: String,
    record_id: String,
    deletion_reason: Option<String>,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "the_librarian")?;

    let rid = Uuid::parse_str(&record_id).map_err(|_| "Invalid record ID".to_string())?;

    // Mark as deleted in archive_permissions with access_level = 'deleted'
    governance_queries::upsert_archive_permission(&table_name, rid, "deleted", session.user_id)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(
        Some(session.user_id),
        "LIBRARIAN_DELETE_RECORD",
        Some("archive_permissions"),
        Some(rid),
        None,
        Some(serde_json::json!({ "table_name": table_name, "reason": deletion_reason })),
    )
    .await;

    Ok(())
}

#[tauri::command]
pub async fn get_archive_permissions(token: String) -> Result<Vec<serde_json::Value>, String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "the_librarian")?;

    let rows = governance_queries::get_all_archive_permissions()
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

// ── Director Create Account ───────────────────────────────────────────────────

#[tauri::command]
pub async fn director_create_account(
    token: String,
    username: String,
    email: String,
    password: String,
    full_name: String,
    role_name: String,
    location: Option<String>,
) -> Result<String, String> {
    use crate::queries::users::create_user as query_create_user;

    let session = validate_session_command(&token).await?;
    // Directors (tier 3) and above can create accounts
    if session.tier < 3 {
        return Err("Only directors and above can create staff accounts".to_string());
    }

    // Directors cannot create other directors or admins — only administrator can do that
    // Check that target role tier < 3
    #[derive(sqlx::FromRow)]
    struct TierCheck { tier: i32 }
    let role_check = sqlx::query_as::<_, TierCheck>(
        "SELECT tier FROM roles WHERE name = $1"
    )
    .bind(&role_name)
    .fetch_optional(crate::db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    if let Some(rc) = role_check {
        if rc.tier >= 3 && session.role_name != "the_administrator" {
            return Err("Only the administrator can create director-tier or above accounts".to_string());
        }
    }

    let hash = bcrypt::hash(&password, bcrypt::DEFAULT_COST)
        .map_err(|e| format!("Hash error: {}", e))?;

    let user = query_create_user(&username, &email, &hash, &full_name, &role_name, location.as_deref())
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(
        Some(session.user_id),
        "DIRECTOR_CREATE_ACCOUNT",
        Some("users"),
        Some(user.id),
        None,
        Some(serde_json::json!({ "username": username, "role": role_name })),
    )
    .await;

    Ok(user.id.to_string())
}
