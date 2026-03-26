use crate::db::get_db;
use uuid::Uuid;

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct VoteRow {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: Option<String>,
    pub vote_type: Option<String>,
    pub initiated_by: Option<Uuid>,
    pub interrupted_by: Option<Uuid>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub deadline: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct VoteDetailRow {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: Option<String>,
    pub initiated_by: Option<Uuid>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub deadline: Option<chrono::DateTime<chrono::Utc>>,
    pub yes_count: Option<i64>,
    pub no_count: Option<i64>,
    pub abstain_count: Option<i64>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct MeetingRow {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub scheduled_at: Option<chrono::DateTime<chrono::Utc>>,
    pub location: Option<String>,
    pub created_by: Option<Uuid>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub to_recipients: Option<serde_json::Value>,
    pub cc_recipients: Option<serde_json::Value>,
    pub bcc_recipients: Option<serde_json::Value>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct RelocationRow {
    pub id: Uuid,
    pub staff_id: Option<Uuid>,
    pub to_location: String,
    pub relocation_type: Option<String>,
    pub start_date: Option<chrono::NaiveDate>,
    pub end_date: Option<chrono::NaiveDate>,
    pub reason: Option<String>,
    pub requested_by: Option<Uuid>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct EventDocumentRow {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub event_date: Option<chrono::DateTime<chrono::Utc>>,
    pub venue: Option<String>,
    pub venue_invoice: Option<String>,
    pub logged_by: Option<Uuid>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub async fn insert_vote(
    title: &str,
    description: Option<&str>,
    initiated_by: Uuid,
) -> Result<Uuid, sqlx::Error> {
    insert_vote_typed(title, description, initiated_by, "pressing_issue").await
}

/// Insert a vote with an explicit vote_type.
/// vote_type must be one of: 'general', 'budget', 'pressing_issue'
pub async fn insert_vote_typed(
    title: &str,
    description: Option<&str>,
    initiated_by: Uuid,
    vote_type: &str,
) -> Result<Uuid, sqlx::Error> {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO votes (title, description, status, initiated_by, deadline, vote_type) \
         VALUES ($1,$2,'open',$3, NOW() + INTERVAL '15 minutes',$4) RETURNING id",
    )
    .bind(title)
    .bind(description)
    .bind(initiated_by)
    .bind(vote_type)
    .fetch_one(get_db())
    .await?;
    Ok(row.0)
}

pub async fn check_vote_status(
    vote_id: Uuid,
) -> Result<Option<(String, Option<chrono::DateTime<chrono::Utc>>)>, sqlx::Error> {
    let row: Option<(String, Option<chrono::DateTime<chrono::Utc>>)> = sqlx::query_as(
        "SELECT status, deadline FROM votes WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(vote_id)
    .fetch_optional(get_db())
    .await?;
    Ok(row)
}

pub async fn upsert_vote_ballot(
    vote_id: Uuid,
    voter_id: Uuid,
    decision: &str,
    reasoning: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"INSERT INTO vote_ballots (vote_id, voter_id, decision, reasoning, voted_at)
           VALUES ($1, $2, $3, $4, NOW())
           ON CONFLICT (vote_id, voter_id) DO UPDATE SET decision = $3, reasoning = $4, voted_at = NOW()"#,
    )
    .bind(vote_id)
    .bind(voter_id)
    .bind(decision)
    .bind(reasoning)
    .execute(get_db())
    .await?;
    Ok(())
}

/// Count ballots cast for a given vote.
pub async fn count_vote_ballots(vote_id: Uuid) -> Result<i64, sqlx::Error> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM vote_ballots WHERE vote_id = $1",
    )
    .bind(vote_id)
    .fetch_one(get_db())
    .await?;
    Ok(row.0)
}

/// Try to resolve a vote immediately if quorum (8 directors) has been reached.
/// Returns true if the vote was resolved.
pub async fn try_resolve_vote_on_quorum(vote_id: Uuid) -> Result<bool, sqlx::Error> {
    const QUORUM: i64 = 8;

    // Count all ballots
    let total = count_vote_ballots(vote_id).await?;
    if total < QUORUM {
        return Ok(false);
    }

    // Count yes/no/abstain
    let counts: (i64, i64, i64) = sqlx::query_as(
        r#"SELECT
            (SELECT COUNT(*) FROM vote_ballots WHERE vote_id = $1 AND decision = 'yes'),
            (SELECT COUNT(*) FROM vote_ballots WHERE vote_id = $1 AND decision = 'no'),
            (SELECT COUNT(*) FROM vote_ballots WHERE vote_id = $1 AND decision = 'abstain')"#,
    )
    .bind(vote_id)
    .fetch_one(get_db())
    .await?;

    let (yes_count, no_count, abstain_count) = counts;

    // All-abstain → auto-deny
    let new_status = if abstain_count == total {
        "failed"
    } else if yes_count > no_count {
        "passed"
    } else {
        "failed"
    };

    sqlx::query("UPDATE votes SET status = $1 WHERE id = $2 AND status = 'open'")
        .bind(new_status)
        .bind(vote_id)
        .execute(get_db())
        .await?;

    // Notify initiator and update linked general_requests
    notify_vote_resolution(vote_id, new_status).await.ok();

    Ok(true)
}

pub async fn resolve_expired_votes() -> Result<(), sqlx::Error> {
    // Fetch open expired votes
    let expired_ids: Vec<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM votes WHERE status = 'open' AND deadline IS NOT NULL AND deadline < NOW()",
    )
    .fetch_all(get_db())
    .await?;

    for (vote_id,) in expired_ids {
        let counts: (i64, i64, i64, i64) = sqlx::query_as(
            r#"SELECT
                (SELECT COUNT(*) FROM vote_ballots WHERE vote_id = $1),
                (SELECT COUNT(*) FROM vote_ballots WHERE vote_id = $1 AND decision = 'yes'),
                (SELECT COUNT(*) FROM vote_ballots WHERE vote_id = $1 AND decision = 'no'),
                (SELECT COUNT(*) FROM vote_ballots WHERE vote_id = $1 AND decision = 'abstain')"#,
        )
        .bind(vote_id)
        .fetch_one(get_db())
        .await?;

        let (total, yes_count, _no_count, abstain_count) = counts;
        const QUORUM: i64 = 8;

        if total < QUORUM {
            // Quorum not met — snooze by extending deadline another 15 minutes
            sqlx::query(
                "UPDATE votes SET deadline = NOW() + INTERVAL '15 minutes' WHERE id = $1 AND status = 'open'",
            )
            .bind(vote_id)
            .execute(get_db())
            .await?;
        } else {
            // Quorum met — resolve
            let new_status = if abstain_count == total {
                "failed" // all abstain = auto-deny
            } else if yes_count > _no_count {
                "passed"
            } else {
                "failed"
            };
            sqlx::query("UPDATE votes SET status = $1 WHERE id = $2 AND status = 'open'")
                .bind(new_status)
                .bind(vote_id)
                .execute(get_db())
                .await?;

            // Notify initiator and update linked general_requests
            notify_vote_resolution(vote_id, new_status).await.ok();
        }
    }

    Ok(())
}

/// After a vote resolves, send an in-app notification message to the initiator
/// and auto-update any linked general_request to match the vote outcome.
async fn notify_vote_resolution(vote_id: Uuid, outcome: &str) -> Result<(), sqlx::Error> {
    // Fetch vote title and initiator
    let row: Option<(Option<String>, Option<Uuid>)> = sqlx::query_as(
        "SELECT title, initiated_by FROM votes WHERE id = $1",
    )
    .bind(vote_id)
    .fetch_optional(get_db())
    .await?;

    let (title, initiated_by) = match row {
        Some((t, i)) => (t.unwrap_or_default(), i),
        None => return Ok(()),
    };

    // Auto-update linked general_requests
    let gr_outcome = if outcome == "passed" { "approved" } else { "rejected" };
    sqlx::query(
        "UPDATE general_requests SET status = $1 WHERE vote_id = $2 AND status = 'under_vote'",
    )
    .bind(gr_outcome)
    .bind(vote_id)
    .execute(get_db())
    .await
    .ok();

    // Auto-update linked blueprint_proposals
    sqlx::query(
        "UPDATE blueprint_proposals SET status = $1 WHERE vote_id = $2 AND status = 'under_vote'",
    )
    .bind(gr_outcome)
    .bind(vote_id)
    .execute(get_db())
    .await
    .ok();

    // Send a self-notification message to the initiator
    if let Some(initiator_id) = initiated_by {
        let outcome_label = if outcome == "passed" { "PASSED ✅" } else { "FAILED ❌" };
        let subject = format!("Vote Result: {} — {}", outcome_label, title);
        let body = format!(
            "The director vote for \"{}\" has concluded.\nOutcome: {}\n\nYou may review the full vote details in the Governance section.",
            title, outcome_label
        );
        // Insert a notification message (self-addressed: from=to=initiator)
        let msg_row: Result<(Uuid,), _> = sqlx::query_as(
            "INSERT INTO messages (from_user_id, subject, body, is_draft) VALUES ($1,$2,$3,false) RETURNING id",
        )
        .bind(initiator_id)
        .bind(&subject)
        .bind(&body)
        .fetch_one(get_db())
        .await;

        if let Ok((msg_id,)) = msg_row {
            sqlx::query(
                "INSERT INTO message_recipients (message_id, recipient_id, recipient_type) VALUES ($1,$2,'to')",
            )
            .bind(msg_id)
            .bind(initiator_id)
            .execute(get_db())
            .await
            .ok();
        }

        // Also notify the original requester if this came from a general_request
        let gr_requester: Option<(Uuid,)> = sqlx::query_as(
            "SELECT requested_by FROM general_requests WHERE vote_id = $1 AND requested_by != $2 LIMIT 1",
        )
        .bind(vote_id)
        .bind(initiator_id)
        .fetch_optional(get_db())
        .await
        .unwrap_or(None);

        if let Some((requester_id,)) = gr_requester {
            let req_subject = format!("Your request vote has {}", if outcome == "passed" { "passed ✅" } else { "failed ❌" });
            let req_body = format!(
                "The directors' vote linked to your request \"{}\" has concluded with outcome: {}.\n\nYou may view the result in the Governance section.",
                title, outcome_label
            );
            let req_msg: Result<(Uuid,), _> = sqlx::query_as(
                "INSERT INTO messages (from_user_id, subject, body, is_draft) VALUES ($1,$2,$3,false) RETURNING id",
            )
            .bind(initiator_id) // sent by initiator (Director who ran the vote)
            .bind(&req_subject)
            .bind(&req_body)
            .fetch_one(get_db())
            .await;

            if let Ok((req_msg_id,)) = req_msg {
                sqlx::query(
                    "INSERT INTO message_recipients (message_id, recipient_id, recipient_type) VALUES ($1,$2,'to')",
                )
                .bind(req_msg_id)
                .bind(requester_id)
                .execute(get_db())
                .await
                .ok();
            }
        }
    }

    Ok(())
}

pub async fn get_all_votes() -> Result<Vec<VoteRow>, sqlx::Error> {
    sqlx::query_as::<_, VoteRow>(
        "SELECT id, title, description, status, \
         COALESCE(vote_type, 'pressing_issue') as vote_type, \
         initiated_by, interrupted_by, created_at, deadline \
         FROM votes WHERE deleted_at IS NULL ORDER BY created_at DESC",
    )
    .fetch_all(get_db())
    .await
}

pub async fn get_vote_details(vote_id: Uuid) -> Result<Option<VoteDetailRow>, sqlx::Error> {
    sqlx::query_as::<_, VoteDetailRow>(
        r#"SELECT v.id, v.title, v.description, v.status, v.initiated_by, v.created_at, v.deadline,
                  (SELECT COUNT(*) FROM vote_ballots WHERE vote_id = v.id AND decision = 'yes') as yes_count,
                  (SELECT COUNT(*) FROM vote_ballots WHERE vote_id = v.id AND decision = 'no') as no_count,
                  (SELECT COUNT(*) FROM vote_ballots WHERE vote_id = v.id AND decision = 'abstain') as abstain_count
           FROM votes v WHERE v.id = $1"#,
    )
    .bind(vote_id)
    .fetch_optional(get_db())
    .await
}

pub async fn interrupt_vote(vote_id: Uuid, interrupted_by: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE votes SET status = 'interrupted', interrupted_by = $1 WHERE id = $2 AND status = 'open'",
    )
    .bind(interrupted_by)
    .bind(vote_id)
    .execute(get_db())
    .await?;
    Ok(())
}

pub async fn insert_meeting(
    title: &str,
    description: Option<&str>,
    scheduled_at: chrono::DateTime<chrono::Utc>,
    location: Option<&str>,
    created_by: Uuid,
    to_recipients: &serde_json::Value,
    cc_recipients: &serde_json::Value,
    bcc_recipients: &serde_json::Value,
) -> Result<Uuid, sqlx::Error> {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO meetings (title, description, scheduled_at, location, created_by, to_recipients, cc_recipients, bcc_recipients) VALUES ($1,$2,$3,$4,$5,$6,$7,$8) RETURNING id",
    )
    .bind(title)
    .bind(description)
    .bind(scheduled_at)
    .bind(location)
    .bind(created_by)
    .bind(to_recipients)
    .bind(cc_recipients)
    .bind(bcc_recipients)
    .fetch_one(get_db())
    .await?;
    Ok(row.0)
}

pub async fn insert_meeting_attendee(
    meeting_id: Uuid,
    user_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO meeting_attendees (meeting_id, user_id) VALUES ($1,$2) ON CONFLICT DO NOTHING",
    )
    .bind(meeting_id)
    .bind(user_id)
    .execute(get_db())
    .await?;
    Ok(())
}

pub async fn get_all_meetings() -> Result<Vec<MeetingRow>, sqlx::Error> {
    sqlx::query_as::<_, MeetingRow>(
        "SELECT id, title, description, scheduled_at, location, created_by, created_at, \
         to_recipients, cc_recipients, bcc_recipients \
         FROM meetings WHERE deleted_at IS NULL ORDER BY scheduled_at DESC",
    )
    .fetch_all(get_db())
    .await
}

pub async fn insert_staff_relocation(
    staff_id: Uuid,
    to_location: &str,
    relocation_type: &str,
    start_date: Option<chrono::NaiveDate>,
    end_date: Option<chrono::NaiveDate>,
    reason: Option<&str>,
    requested_by: Uuid,
) -> Result<Uuid, sqlx::Error> {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO staff_relocations (staff_id, to_location, relocation_type, start_date, end_date, reason, requested_by) VALUES ($1,$2,$3,$4,$5,$6,$7) RETURNING id",
    )
    .bind(staff_id)
    .bind(to_location)
    .bind(relocation_type)
    .bind(start_date)
    .bind(end_date)
    .bind(reason)
    .bind(requested_by)
    .fetch_one(get_db())
    .await?;
    Ok(row.0)
}

pub async fn get_all_relocations() -> Result<Vec<RelocationRow>, sqlx::Error> {
    sqlx::query_as::<_, RelocationRow>(
        "SELECT id, staff_id, to_location, relocation_type, start_date, end_date, reason, requested_by, created_at FROM staff_relocations ORDER BY created_at DESC",
    )
    .fetch_all(get_db())
    .await
}

pub async fn upsert_archive_permission(
    table_name: &str,
    record_id: Uuid,
    access_level: &str,
    set_by: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO archive_permissions (table_name, record_id, access_level, set_by) VALUES ($1,$2,$3,$4) ON CONFLICT (table_name, record_id) DO UPDATE SET access_level = $3, set_by = $4",
    )
    .bind(table_name)
    .bind(record_id)
    .bind(access_level)
    .bind(set_by)
    .execute(get_db())
    .await?;
    Ok(())
}

pub async fn insert_event_document(
    title: &str,
    description: Option<&str>,
    event_date: Option<chrono::DateTime<chrono::Utc>>,
    venue: Option<&str>,
    venue_invoice: Option<&str>,
    logged_by: Uuid,
) -> Result<Uuid, sqlx::Error> {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO event_documents (title, description, event_date, venue, venue_invoice, logged_by) VALUES ($1,$2,$3,$4,$5,$6) RETURNING id",
    )
    .bind(title)
    .bind(description)
    .bind(event_date)
    .bind(venue)
    .bind(venue_invoice)
    .bind(logged_by)
    .fetch_one(get_db())
    .await?;
    Ok(row.0)
}

pub async fn get_all_event_documents() -> Result<Vec<EventDocumentRow>, sqlx::Error> {
    sqlx::query_as::<_, EventDocumentRow>(
        "SELECT id, title, description, event_date, venue, venue_invoice, logged_by, created_at FROM event_documents WHERE deleted_at IS NULL ORDER BY event_date DESC",
    )
    .fetch_all(get_db())
    .await
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct ArchivePermissionRow {
    pub id: Uuid,
    pub table_name: String,
    pub record_id: Uuid,
    pub access_level: String,
    pub set_by: Uuid,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub async fn get_all_archive_permissions() -> Result<Vec<ArchivePermissionRow>, sqlx::Error> {
    sqlx::query_as::<_, ArchivePermissionRow>(
        "SELECT id, table_name, record_id, access_level, set_by, created_at FROM archive_permissions ORDER BY created_at DESC",
    )
    .fetch_all(get_db())
    .await
}
