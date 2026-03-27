pub mod db;
pub mod models;
pub mod queries;
pub mod auth;
pub mod commands;

use tauri::Emitter;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_handle = app.handle().clone();
            tauri::async_runtime::block_on(async move {
                let database_url = std::env::var("DATABASE_URL")
                    .unwrap_or_else(|_| "postgres://rusa:rusa@localhost:5432/rusa_ims".to_string());
                let redis_url = std::env::var("REDIS_URL")
                    .unwrap_or_else(|_| "redis://127.0.0.1/".to_string());

                db::init_db(&database_url).await.expect("Failed to init DB");
                db::run_migrations()
                    .await
                    .expect("Failed to run DB migrations");
                db::init_redis(&redis_url).expect("Failed to init Redis");
            });

            let scheduler_app = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                #[derive(sqlx::FromRow)]
                struct DueMessageRow {
                    id: uuid::Uuid,
                    subject: String,
                    is_broadcast: Option<bool>,
                    broadcast_sender: Option<String>,
                }

                loop {
                    let due: Result<Vec<DueMessageRow>, sqlx::Error> = sqlx::query_as::<_, DueMessageRow>(
                        r#"
                        SELECT id, subject, is_broadcast, broadcast_sender
                        FROM messages
                        WHERE deleted_at IS NULL
                          AND recalled_at IS NULL
                          AND is_draft = false
                          AND sent_at IS NOT NULL
                          AND sent_at <= NOW()
                          AND notified_at IS NULL
                        ORDER BY sent_at ASC
                        LIMIT 50
                        "#,
                    )
                    .fetch_all(db::get_db())
                    .await;

                    if let Ok(rows) = due {
                        for row in rows {
                            let _ = sqlx::query("UPDATE messages SET notified_at = NOW() WHERE id = $1 AND notified_at IS NULL")
                                .bind(row.id)
                                .execute(db::get_db())
                                .await;

                            if row.is_broadcast.unwrap_or(false) {
                                let recipients: Vec<(uuid::Uuid,)> = sqlx::query_as(
                                    "SELECT recipient_id FROM message_recipients WHERE message_id = $1 AND deleted_at IS NULL",
                                )
                                .bind(row.id)
                                .fetch_all(db::get_db())
                                .await
                                .unwrap_or_default();
                                let target_users: Vec<uuid::Uuid> = recipients.into_iter().map(|r| r.0).collect();

                                let payload = serde_json::json!({
                                    "message_id": row.id,
                                    "from": row.broadcast_sender.clone().unwrap_or_else(|| "SYSTEM".to_string()),
                                    "subject": row.subject,
                                    "is_broadcast": true,
                                    "target_users": target_users
                                });
                                let _ = scheduler_app.emit("new_broadcast", payload);
                            } else {
                                let payload = serde_json::json!({
                                    "message_id": row.id,
                                    "subject": row.subject
                                });
                                let _ = scheduler_app.emit("new_message", payload);
                            }
                        }
                    }

                    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // auth
            commands::auth_commands::login,
            commands::auth_commands::logout,
            commands::auth_commands::get_current_user,
            // messages
            commands::message_commands::send_message,
            commands::message_commands::get_inbox,
            commands::message_commands::get_sent,
            commands::message_commands::get_message,
            commands::message_commands::recall_message,
            commands::message_commands::mark_message_read,
            // users
            commands::user_commands::get_all_users,
            commands::user_commands::get_roles,
            commands::user_commands::create_user,
            commands::user_commands::deactivate_user,
            commands::user_commands::delete_user,
            // research
            commands::research_commands::get_experiments,
            commands::research_commands::propose_experiment,
            commands::research_commands::review_experiment,
            commands::research_commands::add_experiment_log,
            commands::research_commands::get_experiment_logs,
            commands::research_commands::request_experiment_conclusion,
            commands::research_commands::approve_experiment_conclusion,
            commands::research_commands::get_species_archive,
            commands::research_commands::add_species,
            commands::research_commands::get_test_archive,
            commands::research_commands::propose_test,
            commands::research_commands::review_test_proposal,
            commands::research_commands::link_tests_to_log,
            commands::research_commands::propose_species_from_discovery,
            commands::research_commands::get_observer_dashboard,
            commands::research_commands::assign_experiment_task,
            commands::research_commands::get_experiment_tasks,
            // data
            commands::data_commands::submit_data_request,
            commands::data_commands::get_data_requests,
            commands::data_commands::review_data_request,
            commands::data_commands::start_processing,
            commands::data_commands::analyst_submit_response,
            commands::data_commands::analyst_reject_request,
            commands::data_commands::deliver_data_response,
            commands::data_commands::acknowledge_data_response,
            commands::data_commands::list_data_response_attachments,
            commands::data_commands::download_data_response_attachment,
            // security
            commands::security_commands::create_incident_report,
            commands::security_commands::get_incident_reports,
            commands::security_commands::update_incident_status,
            commands::security_commands::add_lost_found_item,
            commands::security_commands::get_lost_found,
            commands::security_commands::claim_lost_found,
            commands::security_commands::submit_broadcast_request,
            commands::security_commands::get_broadcast_requests,
            commands::security_commands::send_broadcast_direct,
            commands::security_commands::review_broadcast_request,
            commands::security_commands::submit_security_findings,
            commands::security_commands::get_security_findings,
            commands::security_commands::assign_security_task,
            commands::security_commands::get_security_tasks,
            commands::security_commands::update_security_task_status,
            commands::security_commands::submit_external_report,
            commands::security_commands::get_external_reports,
            // astronaut
            commands::astronaut_commands::get_missions,
            commands::astronaut_commands::create_mission,
            commands::astronaut_commands::update_mission_status,
            commands::astronaut_commands::assign_crew,
            commands::astronaut_commands::submit_mission_report,
            commands::astronaut_commands::get_mission_reports,
            commands::astronaut_commands::create_exploration_journal,
            commands::astronaut_commands::get_journals,
            commands::astronaut_commands::get_sectors,
            commands::astronaut_commands::create_sector,
            commands::astronaut_commands::rename_sector,
            commands::astronaut_commands::get_ships,
            commands::astronaut_commands::get_astronaut_stats,
            commands::astronaut_commands::submit_conclusion_request,
            commands::astronaut_commands::get_conclusion_requests,
            commands::astronaut_commands::review_conclusion_request,
            commands::astronaut_commands::get_planets,
            commands::astronaut_commands::create_planet,
            commands::astronaut_commands::rename_planet,
            // settlement
            commands::settlement_commands::get_settlements,
            commands::settlement_commands::get_settler_tasks,
            commands::settlement_commands::assign_settler_task,
            commands::settlement_commands::update_task_progress,
            commands::settlement_commands::submit_supply_request,
            commands::settlement_commands::review_supply_request,
            commands::settlement_commands::submit_anomaly_report,
            commands::settlement_commands::review_anomaly_report,
            commands::settlement_commands::issue_house_arrest,
            commands::settlement_commands::submit_send_to_earth,
            commands::settlement_commands::review_send_to_earth,
            commands::settlement_commands::log_settlement_inventory,
            commands::settlement_commands::get_settlement_inventory,
            commands::settlement_commands::submit_farm_report,
            commands::settlement_commands::get_supply_requests,
            commands::settlement_commands::get_anomaly_reports,
            commands::settlement_commands::get_farm_reports,
            commands::settlement_commands::submit_troublesome_settler_report,
            commands::settlement_commands::get_troublesome_settler_reports,
            commands::settlement_commands::submit_civil_engineer_report,
            commands::settlement_commands::get_civil_engineer_reports,
            // aerospace
            commands::aerospace_commands::get_work_orders,
            commands::aerospace_commands::create_work_order,
            commands::aerospace_commands::update_work_order_status,
            commands::aerospace_commands::submit_technical_report,
            commands::aerospace_commands::get_technical_reports,
            commands::aerospace_commands::assign_aerospace_task,
            commands::aerospace_commands::get_aerospace_assigned_tasks,
            commands::aerospace_commands::update_aerospace_task_status,
            commands::aerospace_commands::submit_blueprint_proposal,
            commands::aerospace_commands::get_blueprint_proposals,
            commands::aerospace_commands::review_blueprint_proposal,
            commands::aerospace_commands::update_ship_status,
            commands::aerospace_commands::request_aerospace_task_conclusion,
            commands::aerospace_commands::approve_aerospace_task_conclusion,
            commands::aerospace_commands::get_all_ships,
            commands::aerospace_commands::get_ship_details,
            commands::aerospace_commands::submit_help_request,
            commands::aerospace_commands::get_help_requests,
            commands::aerospace_commands::resolve_help_request,
            commands::aerospace_commands::reject_help_request,
            commands::aerospace_commands::approve_help_request,
            commands::aerospace_commands::proxy_deliver_task_response,
            // station
            commands::station_commands::get_stations,
            commands::station_commands::get_station_inventory,
            commands::station_commands::update_station_inventory,
            commands::station_commands::add_map_annotation,
            commands::station_commands::get_map_annotations,
            commands::station_commands::log_personnel_event,
            commands::station_commands::get_personnel_log,
            commands::station_commands::submit_station_findings,
            commands::station_commands::get_station_findings,
            commands::station_commands::get_station_supply_requests,
            commands::station_commands::submit_station_supply_request,
            commands::station_commands::review_station_supply_request,
            commands::station_commands::submit_station_abandonment,
            // psychiatry
            commands::psychiatry_commands::register_patient,
            commands::psychiatry_commands::get_patients,
            commands::psychiatry_commands::schedule_appointment,
            commands::psychiatry_commands::get_appointments,
            commands::psychiatry_commands::complete_appointment,
            commands::psychiatry_commands::add_recovery_log,
            commands::psychiatry_commands::get_recovery_log,
            commands::psychiatry_commands::assign_psychiatry_task,
            commands::psychiatry_commands::get_psychiatry_tasks,
            commands::psychiatry_commands::update_psychiatry_task_status,
            // medical
            commands::medical_commands::allocate_shift,
            commands::medical_commands::get_shifts,
            commands::medical_commands::get_medical_inventory,
            commands::medical_commands::update_medical_inventory,
            commands::medical_commands::create_patient_record,
            commands::medical_commands::get_patient_records,
            commands::medical_commands::add_specialization,
            commands::medical_commands::get_staff_specializations,
            commands::medical_commands::submit_budget_request,
            commands::medical_commands::submit_expenditure_report,
            commands::medical_commands::assign_medical_task,
            commands::medical_commands::get_medical_tasks,
            commands::medical_commands::update_medical_task_status,
            // sanitary
            commands::sanitary_commands::get_sanitary_tasks,
            commands::sanitary_commands::assign_sanitary_task,
            commands::sanitary_commands::update_sanitary_task,
            commands::sanitary_commands::get_sanitary_inventory,
            commands::sanitary_commands::update_sanitary_inventory,
            commands::sanitary_commands::add_disposal_log,
            commands::sanitary_commands::get_disposal_logs,
            commands::sanitary_commands::add_wastewater_log,
            commands::sanitary_commands::get_wastewater_logs,
            commands::sanitary_commands::submit_division_transfer,
            commands::sanitary_commands::review_division_transfer,
            commands::sanitary_commands::set_division_quota,
            commands::sanitary_commands::create_inspection_report,
            commands::sanitary_commands::get_inspection_reports,
            commands::sanitary_commands::send_inspection_to_head,
            // governance
            commands::governance_commands::initiate_vote,
            commands::governance_commands::cast_vote,
            commands::governance_commands::get_votes,
            commands::governance_commands::get_vote_details,
            commands::governance_commands::interrupt_vote,
            commands::governance_commands::create_meeting,
            commands::governance_commands::get_meetings,
            commands::governance_commands::relocate_staff,
            commands::governance_commands::get_relocations,
            commands::governance_commands::set_archive_permission,
            commands::governance_commands::log_event_document,
            commands::governance_commands::get_event_documents,
            commands::governance_commands::nomad_assign_task,
            commands::governance_commands::get_nomad_tasks,
            commands::governance_commands::update_nomad_task_status,
            commands::governance_commands::redact_record,
            commands::governance_commands::librarian_delete_record,
            commands::governance_commands::get_archive_permissions,
            commands::governance_commands::director_create_account,
            // budget
            commands::budget_commands::get_budget_requests,
            commands::budget_commands::review_budget_request,
            commands::budget_commands::flag_budget_request,
            commands::budget_commands::get_expenditure_reports,
            commands::budget_commands::flag_expenditure_report,
            commands::budget_commands::submit_investigation,
            commands::budget_commands::initiate_budget_vote,
            // general requests
            commands::general_commands::submit_general_request,
            commands::general_commands::get_general_requests,
            commands::general_commands::review_general_request,
            // admin
            commands::admin_commands::admin_create_director,
            commands::admin_commands::admin_terminate_director,
            commands::admin_commands::terminate_personnel,
            commands::admin_commands::get_audit_log,
            commands::admin_commands::override_vote,
            // chemistry
            commands::chemistry_commands::get_matter_archive,
            commands::chemistry_commands::add_chemistry_log,
            commands::chemistry_commands::approve_chemistry_conclusion,
            commands::chemistry_commands::get_chemistry_observer_dashboard,
            // research tasks
            commands::research_task_commands::assign_research_task,
            commands::research_task_commands::get_research_tasks,
            commands::research_task_commands::submit_research_task_result,
            commands::research_task_commands::complete_research_task,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

