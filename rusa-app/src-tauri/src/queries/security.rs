use crate::db::get_db;
use std::collections::HashSet;
use uuid::Uuid;

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct IncidentReportRow {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub incident_date: Option<chrono::DateTime<chrono::Utc>>,
    pub severity: Option<String>,
    pub status: Option<String>,
    pub reported_by: Option<Uuid>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct LostFoundRow {
    pub id: Uuid,
    pub item_name: String,
    pub description: Option<String>,
    pub found_location: Option<String>,
    pub found_date: Option<chrono::NaiveDate>,
    pub status: Option<String>,
    pub logged_by: Option<Uuid>,
    pub claimed_by: Option<Uuid>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct BroadcastRequestRow {
    pub id: Uuid,
    pub title: String,
    pub content: Option<String>,
    pub target_audience: Option<String>,
    pub target_filters: Option<serde_json::Value>,
    pub routed_to: Option<String>,
    pub status: Option<String>,
    pub requested_by: Option<Uuid>,
    pub reviewed_by: Option<Uuid>,
    pub review_notes: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct SecurityFindingRow {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub findings_date: Option<chrono::NaiveDate>,
    pub reported_by: Option<Uuid>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub async fn insert_incident_report(
    title: &str,
    description: &str,
    location: Option<&str>,
    incident_date: Option<chrono::DateTime<chrono::Utc>>,
    severity: &str,
    reported_by: Uuid,
) -> Result<Uuid, sqlx::Error> {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO incident_reports (title, description, location, incident_date, severity, status, reported_by) VALUES ($1,$2,$3,$4,$5,'open',$6) RETURNING id",
    )
    .bind(title)
    .bind(description)
    .bind(location)
    .bind(incident_date)
    .bind(severity)
    .bind(reported_by)
    .fetch_one(get_db())
    .await?;
    Ok(row.0)
}

pub async fn get_all_incident_reports() -> Result<Vec<IncidentReportRow>, sqlx::Error> {
    sqlx::query_as::<_, IncidentReportRow>(
        "SELECT id, title, description, location, incident_date, severity, status, reported_by, created_at FROM incident_reports WHERE deleted_at IS NULL ORDER BY created_at DESC",
    )
    .fetch_all(get_db())
    .await
}

pub async fn update_incident_status(
    report_id: Uuid,
    status: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE incident_reports SET status = $1 WHERE id = $2 AND deleted_at IS NULL",
    )
    .bind(status)
    .bind(report_id)
    .execute(get_db())
    .await?;
    Ok(())
}

pub async fn insert_lost_found_item(
    item_name: &str,
    description: Option<&str>,
    found_location: Option<&str>,
    found_date: Option<chrono::NaiveDate>,
    logged_by: Uuid,
) -> Result<Uuid, sqlx::Error> {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO lost_and_found (item_name, description, found_location, found_date, status, logged_by) VALUES ($1,$2,$3,$4,'unclaimed',$5) RETURNING id",
    )
    .bind(item_name)
    .bind(description)
    .bind(found_location)
    .bind(found_date)
    .bind(logged_by)
    .fetch_one(get_db())
    .await?;
    Ok(row.0)
}

pub async fn get_all_lost_found() -> Result<Vec<LostFoundRow>, sqlx::Error> {
    sqlx::query_as::<_, LostFoundRow>(
        "SELECT id, item_name, description, found_location, found_date, status, logged_by, claimed_by, created_at FROM lost_and_found WHERE deleted_at IS NULL ORDER BY created_at DESC",
    )
    .fetch_all(get_db())
    .await
}

pub async fn claim_lost_found_item(
    item_id: Uuid,
    claimed_by: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE lost_and_found SET status = 'claimed', claimed_by = $1 WHERE id = $2 AND deleted_at IS NULL AND status = 'unclaimed'",
    )
    .bind(claimed_by)
    .bind(item_id)
    .execute(get_db())
    .await?;
    Ok(())
}

pub async fn insert_broadcast_request(
    title: &str,
    content: &str,
    target_audience: Option<&str>,
    target_filters: Option<serde_json::Value>,
    routed_to: &str,
    requested_by: Uuid,
) -> Result<Uuid, sqlx::Error> {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO broadcast_requests (title, content, target_audience, target_filters, routed_to, status, requested_by) VALUES ($1,$2,$3,$4,$5,'pending',$6) RETURNING id",
    )
    .bind(title)
    .bind(content)
    .bind(target_audience)
    .bind(target_filters)
    .bind(routed_to)
    .bind(requested_by)
    .fetch_one(get_db())
    .await?;
    Ok(row.0)
}

pub async fn get_broadcast_requests(routed_to_role: Option<&str>) -> Result<Vec<BroadcastRequestRow>, sqlx::Error> {
    let mut query = "SELECT id, title, content, target_audience, target_filters, routed_to, status, requested_by, reviewed_by, review_notes, created_at FROM broadcast_requests WHERE deleted_at IS NULL".to_string();
    
    if routed_to_role.is_some() {
        query.push_str(" AND routed_to = $1");
    }
    
    query.push_str(" ORDER BY created_at DESC");

    let mut q = sqlx::query_as::<_, BroadcastRequestRow>(&query);
    if let Some(role) = routed_to_role {
        q = q.bind(role);
    }
    
    q.fetch_all(get_db()).await
}

pub async fn get_broadcast_request_by_id(request_id: Uuid) -> Result<Option<BroadcastRequestRow>, sqlx::Error> {
    sqlx::query_as::<_, BroadcastRequestRow>(
        "SELECT id, title, content, target_audience, target_filters, routed_to, status, requested_by, reviewed_by, review_notes, created_at FROM broadcast_requests WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(request_id)
    .fetch_optional(get_db())
    .await
}

pub async fn resolve_broadcast_targets(
    target_filters: Option<&serde_json::Value>,
) -> Result<Vec<Uuid>, sqlx::Error> {
    #[derive(sqlx::FromRow)]
    struct UserIdRow {
        id: Uuid,
    }

    let mut set: HashSet<Uuid> = HashSet::new();

    let mut personnel_ids: Vec<Uuid> = Vec::new();
    let mut departments: Vec<String> = Vec::new();
    let mut locations: Vec<String> = Vec::new();

    if let Some(filters) = target_filters {
        if let Some(arr) = filters.get("personnel").and_then(|v| v.as_array()) {
            for v in arr {
                if let Some(s) = v.as_str() {
                    if let Ok(id) = Uuid::parse_str(s) {
                        personnel_ids.push(id);
                    }
                }
            }
        }
        if let Some(arr) = filters.get("departments").and_then(|v| v.as_array()) {
            for v in arr {
                if let Some(s) = v.as_str() {
                    let t = s.trim();
                    if !t.is_empty() {
                        departments.push(t.to_string());
                    }
                }
            }
        }
        if let Some(arr) = filters.get("locations").and_then(|v| v.as_array()) {
            for v in arr {
                if let Some(s) = v.as_str() {
                    let t = s.trim();
                    if !t.is_empty() {
                        locations.push(t.to_string());
                    }
                }
            }
        }
    }

    let has_any_filter = !personnel_ids.is_empty() || !departments.is_empty() || !locations.is_empty();

    if !has_any_filter {
        let rows = sqlx::query_as::<_, UserIdRow>(
            "SELECT id FROM users WHERE deleted_at IS NULL AND is_active = true",
        )
        .fetch_all(get_db())
        .await?;
        return Ok(rows.into_iter().map(|r| r.id).collect());
    }

    for id in personnel_ids {
        set.insert(id);
    }

    if !departments.is_empty() {
        let rows = sqlx::query_as::<_, UserIdRow>(
            "SELECT id FROM users WHERE deleted_at IS NULL AND is_active = true AND department = ANY($1)",
        )
        .bind(&departments)
        .fetch_all(get_db())
        .await?;
        for r in rows {
            set.insert(r.id);
        }
    }

    if !locations.is_empty() {
        let rows = sqlx::query_as::<_, UserIdRow>(
            "SELECT id FROM users WHERE deleted_at IS NULL AND is_active = true AND location = ANY($1)",
        )
        .bind(&locations)
        .fetch_all(get_db())
        .await?;
        for r in rows {
            set.insert(r.id);
        }
    }

    Ok(set.into_iter().collect())
}

pub async fn update_broadcast_request_review(
    request_id: Uuid,
    status: &str,
    reviewed_by: Uuid,
    notes: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE broadcast_requests SET status = $1, reviewed_by = $2, review_notes = $3 WHERE id = $4 AND deleted_at IS NULL",
    )
    .bind(status)
    .bind(reviewed_by)
    .bind(notes)
    .bind(request_id)
    .execute(get_db())
    .await?;
    Ok(())
}

pub async fn insert_security_finding(
    title: &str,
    description: Option<&str>,
    findings_date: Option<chrono::NaiveDate>,
    reported_by: Uuid,
) -> Result<Uuid, sqlx::Error> {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO security_findings (title, description, findings_date, reported_by) VALUES ($1,$2,$3,$4) RETURNING id",
    )
    .bind(title)
    .bind(description)
    .bind(findings_date)
    .bind(reported_by)
    .fetch_one(get_db())
    .await?;
    Ok(row.0)
}

pub async fn get_all_security_findings() -> Result<Vec<SecurityFindingRow>, sqlx::Error> {
    sqlx::query_as::<_, SecurityFindingRow>(
        "SELECT id, title, description, findings_date, reported_by, created_at FROM security_findings WHERE deleted_at IS NULL ORDER BY created_at DESC",
    )
    .fetch_all(get_db())
    .await
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct SecurityTaskRow {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub assigned_to: Option<Uuid>,
    pub assigned_by: Option<Uuid>,
    pub status: String,
    pub activity_logs: Option<serde_json::Value>,
    pub due_date: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub conclusion_requested_at: Option<chrono::DateTime<chrono::Utc>>,
    pub conclusion_requested_by: Option<Uuid>,
    pub conclusion_approved_at: Option<chrono::DateTime<chrono::Utc>>,
    pub conclusion_approved_by: Option<Uuid>,
    pub final_notes: Option<String>,
}

pub async fn ensure_security_tasks_table() -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS security_tasks (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            title TEXT NOT NULL,
            description TEXT,
            assigned_to UUID REFERENCES users(id),
            assigned_by UUID REFERENCES users(id),
            status TEXT NOT NULL DEFAULT 'pending',
            due_date TIMESTAMPTZ,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            deleted_at TIMESTAMPTZ
        )"#,
    )
    .execute(get_db())
    .await?;

    let alter_stmts = [
        "ALTER TABLE security_tasks ADD COLUMN IF NOT EXISTS activity_logs JSONB DEFAULT '[]'",
        "ALTER TABLE security_tasks ADD COLUMN IF NOT EXISTS conclusion_requested_at TIMESTAMPTZ",
        "ALTER TABLE security_tasks ADD COLUMN IF NOT EXISTS conclusion_requested_by UUID REFERENCES users(id)",
        "ALTER TABLE security_tasks ADD COLUMN IF NOT EXISTS conclusion_approved_at TIMESTAMPTZ",
        "ALTER TABLE security_tasks ADD COLUMN IF NOT EXISTS conclusion_approved_by UUID REFERENCES users(id)",
        "ALTER TABLE security_tasks ADD COLUMN IF NOT EXISTS final_notes TEXT",
    ];
    for stmt in &alter_stmts {
        sqlx::query(stmt).execute(get_db()).await?;
    }
    Ok(())
}

const SECURITY_TASK_SELECT: &str =
    "SELECT id, title, description, assigned_to, assigned_by, status, activity_logs, due_date, created_at, \
     conclusion_requested_at, conclusion_requested_by, conclusion_approved_at, conclusion_approved_by, final_notes \
     FROM security_tasks";

pub async fn insert_security_task(
    title: &str,
    description: Option<&str>,
    assigned_to: Uuid,
    assigned_by: Uuid,
    due_date: Option<chrono::DateTime<chrono::Utc>>,
) -> Result<Uuid, sqlx::Error> {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO security_tasks (title, description, assigned_to, assigned_by, status, due_date) VALUES ($1,$2,$3,$4,'pending',$5) RETURNING id",
    )
    .bind(title)
    .bind(description)
    .bind(assigned_to)
    .bind(assigned_by)
    .bind(due_date)
    .fetch_one(get_db())
    .await?;
    Ok(row.0)
}

pub async fn get_security_task_by_id(task_id: Uuid, user_id: Uuid) -> Result<Option<SecurityTaskRow>, sqlx::Error> {
    sqlx::query_as::<_, SecurityTaskRow>(
        &format!("{} WHERE id = $1 AND (assigned_to = $2 OR assigned_by = $2) AND deleted_at IS NULL", SECURITY_TASK_SELECT),
    )
    .bind(task_id)
    .bind(user_id)
    .fetch_optional(get_db())
    .await
}

pub async fn get_security_tasks_for_assigner(assigner_id: Uuid) -> Result<Vec<SecurityTaskRow>, sqlx::Error> {
    sqlx::query_as::<_, SecurityTaskRow>(
        &format!("{} WHERE assigned_by = $1 AND deleted_at IS NULL ORDER BY created_at DESC", SECURITY_TASK_SELECT),
    )
    .bind(assigner_id)
    .fetch_all(get_db())
    .await
}

pub async fn get_security_tasks_for_user(user_id: Uuid) -> Result<Vec<SecurityTaskRow>, sqlx::Error> {
    sqlx::query_as::<_, SecurityTaskRow>(
        &format!("{} WHERE assigned_to = $1 AND deleted_at IS NULL ORDER BY created_at DESC", SECURITY_TASK_SELECT),
    )
    .bind(user_id)
    .fetch_all(get_db())
    .await
}

pub async fn update_security_task_status(task_id: Uuid, status: &str, user_id: Uuid, full_name: &str) -> Result<(), sqlx::Error> {
    let log_entry = serde_json::json!([{
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "author_id": user_id.to_string(),
        "author_name": full_name,
        "content": format!("Status changed to '{}'", status),
        "log_type": "status_change"
    }]);
    sqlx::query(
        "UPDATE security_tasks SET status = $1, \
         activity_logs = COALESCE(activity_logs, '[]'::jsonb) || $2 \
         WHERE id = $3 AND deleted_at IS NULL",
    )
    .bind(status)
    .bind(&log_entry)
    .bind(task_id)
    .execute(get_db())
    .await?;
    Ok(())
}

pub async fn append_security_task_log(task_id: Uuid, user_id: Uuid, full_name: &str, content: &str) -> Result<(), sqlx::Error> {
    let entry = serde_json::json!([{
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "author_id": user_id.to_string(),
        "author_name": full_name,
        "content": content,
        "log_type": "progress_update"
    }]);
    sqlx::query(
        "UPDATE security_tasks SET activity_logs = COALESCE(activity_logs, '[]'::jsonb) || $1 WHERE id = $2 AND deleted_at IS NULL",
    )
    .bind(&entry)
    .bind(task_id)
    .execute(get_db())
    .await?;
    Ok(())
}

pub async fn request_security_task_conclusion(task_id: Uuid, user_id: Uuid, full_name: &str, notes: &str) -> Result<(), sqlx::Error> {
    let log_entry = serde_json::json!([{
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "author_id": user_id.to_string(),
        "author_name": full_name,
        "content": notes,
        "log_type": "conclusion_requested"
    }]);
    sqlx::query(
        "UPDATE security_tasks SET status = 'conclusion_requested', \
         conclusion_requested_at = NOW(), conclusion_requested_by = $1, \
         final_notes = $2, \
         activity_logs = COALESCE(activity_logs, '[]'::jsonb) || $3 \
         WHERE id = $4 AND deleted_at IS NULL",
    )
    .bind(user_id)
    .bind(notes)
    .bind(&log_entry)
    .bind(task_id)
    .execute(get_db())
    .await?;
    Ok(())
}

pub async fn review_security_task_conclusion(task_id: Uuid, user_id: Uuid, full_name: &str, approve: bool, review_notes: Option<&str>) -> Result<(), sqlx::Error> {
    let log_type = if approve { "conclusion_approved" } else { "conclusion_rejected" };
    let log_entry = serde_json::json!([{
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "author_id": user_id.to_string(),
        "author_name": full_name,
        "content": review_notes.unwrap_or(""),
        "log_type": log_type
    }]);

    if approve {
        sqlx::query(
            "UPDATE security_tasks SET status = 'completed', \
             conclusion_approved_at = NOW(), conclusion_approved_by = $1, \
             activity_logs = COALESCE(activity_logs, '[]'::jsonb) || $2 \
             WHERE id = $3 AND deleted_at IS NULL",
        )
        .bind(user_id)
        .bind(&log_entry)
        .bind(task_id)
        .execute(get_db())
        .await?;
    } else {
        sqlx::query(
            "UPDATE security_tasks SET status = 'in_progress', \
             activity_logs = COALESCE(activity_logs, '[]'::jsonb) || $1 \
             WHERE id = $2 AND deleted_at IS NULL",
        )
        .bind(&log_entry)
        .bind(task_id)
        .execute(get_db())
        .await?;
    }
    Ok(())
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct ExternalReportRow {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub submitted_by: Option<Uuid>,
    pub security_type: Option<String>,
    pub status: Option<String>,
    pub incident_report_id: Option<Uuid>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub async fn insert_external_report(
    title: &str,
    description: &str,
    submitted_by: Uuid,
    security_type: Option<&str>,
) -> Result<Uuid, sqlx::Error> {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO external_reports (title, description, submitted_by, security_type, status) VALUES ($1,$2,$3,$4,'submitted') RETURNING id",
    )
    .bind(title)
    .bind(description)
    .bind(submitted_by)
    .bind(security_type)
    .fetch_one(get_db())
    .await?;
    Ok(row.0)
}

pub async fn get_all_external_reports() -> Result<Vec<ExternalReportRow>, sqlx::Error> {
    sqlx::query_as::<_, ExternalReportRow>(
        "SELECT id, title, description, submitted_by, security_type, status, incident_report_id, created_at FROM external_reports WHERE deleted_at IS NULL ORDER BY created_at DESC",
    )
    .fetch_all(get_db())
    .await
}

pub async fn get_external_reports_by_submitter(submitted_by: Uuid) -> Result<Vec<ExternalReportRow>, sqlx::Error> {
    sqlx::query_as::<_, ExternalReportRow>(
        "SELECT id, title, description, submitted_by, security_type, status, incident_report_id, created_at FROM external_reports WHERE submitted_by = $1 AND deleted_at IS NULL ORDER BY created_at DESC",
    )
    .bind(submitted_by)
    .fetch_all(get_db())
    .await
}

// ─── Security Reports (Research Department) ──────────────────────────────────

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct SecurityReportRow {
    pub id: Uuid,
    pub submitted_by: Uuid,
    pub submitted_at: chrono::DateTime<chrono::Utc>,
    pub title: String,
    pub category: String,
    pub description: String,
    pub severity: String,
    pub related_experiment_id: Option<Uuid>,
    pub related_task_id: Option<Uuid>,
    pub status: String,
    pub security_staff_notes: Option<String>,
    pub resolved_at: Option<chrono::DateTime<chrono::Utc>>,
    pub attachments: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub async fn insert_security_report(
    submitted_by: Uuid,
    title: &str,
    category: &str,
    description: &str,
    severity: &str,
    related_experiment_id: Option<Uuid>,
    related_task_id: Option<Uuid>,
    attachments: Option<serde_json::Value>,
) -> Result<Uuid, sqlx::Error> {
    let row: (Uuid,) = sqlx::query_as(
        r#"INSERT INTO security_reports
            (submitted_by, title, category, description, severity, related_experiment_id, related_task_id, attachments)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
           RETURNING id"#,
    )
    .bind(submitted_by)
    .bind(title)
    .bind(category)
    .bind(description)
    .bind(severity)
    .bind(related_experiment_id)
    .bind(related_task_id)
    .bind(attachments)
    .fetch_one(get_db())
    .await?;
    Ok(row.0)
}

pub async fn get_security_reports_by_submitter(submitted_by: Uuid) -> Result<Vec<SecurityReportRow>, sqlx::Error> {
    sqlx::query_as::<_, SecurityReportRow>(
        r#"SELECT id, submitted_by, submitted_at, title, category, description, severity,
                  related_experiment_id, related_task_id, status, security_staff_notes,
                  resolved_at, attachments, created_at
           FROM security_reports
           WHERE submitted_by = $1 AND deleted_at IS NULL
           ORDER BY created_at DESC"#,
    )
    .bind(submitted_by)
    .fetch_all(get_db())
    .await
}

pub async fn get_all_security_reports() -> Result<Vec<SecurityReportRow>, sqlx::Error> {
    sqlx::query_as::<_, SecurityReportRow>(
        r#"SELECT id, submitted_by, submitted_at, title, category, description, severity,
                  related_experiment_id, related_task_id, status, security_staff_notes,
                  resolved_at, attachments, created_at
           FROM security_reports
           WHERE deleted_at IS NULL
           ORDER BY created_at DESC"#,
    )
    .fetch_all(get_db())
    .await
}

pub async fn acknowledge_security_report(
    report_id: Uuid,
    notes: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE security_reports SET status = 'acknowledged', security_staff_notes = COALESCE($1, security_staff_notes) WHERE id = $2 AND deleted_at IS NULL",
    )
    .bind(notes)
    .bind(report_id)
    .execute(get_db())
    .await?;
    Ok(())
}

pub async fn update_security_report_status(
    report_id: Uuid,
    status: &str,
    notes: Option<&str>,
) -> Result<(), sqlx::Error> {
    let resolved_at: Option<chrono::DateTime<chrono::Utc>> = if status == "resolved" || status == "closed" {
        Some(chrono::Utc::now())
    } else {
        None
    };
    sqlx::query(
        r#"UPDATE security_reports
           SET status = $1,
               security_staff_notes = COALESCE($2, security_staff_notes),
               resolved_at = CASE WHEN $3 THEN NOW() ELSE resolved_at END
           WHERE id = $4 AND deleted_at IS NULL"#,
    )
    .bind(status)
    .bind(notes)
    .bind(resolved_at.is_some())
    .bind(report_id)
    .execute(get_db())
    .await?;
    Ok(())
}
