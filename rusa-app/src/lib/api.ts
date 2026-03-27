import { invoke } from '@tauri-apps/api/core';

// Auth
export const authApi = {
  login: (username: string, password: string) =>
    invoke<SessionData>('login', { username, password }),
  logout: (token: string) =>
    invoke<void>('logout', { token }),
  getCurrentUser: (token: string) =>
    invoke<SessionData>('get_current_user', { token }),
};

// Messages
export const messageApi = {
  send: (token: string, to: string[], cc: string[], bcc: string[], subject: string, body: string, scheduledAt?: string) =>
    invoke<string>('send_message', { token, to, cc, bcc, subject, body, scheduledAt }),
  getInbox: (token: string) => invoke<any[]>('get_inbox', { token }),
  getSent: (token: string) => invoke<any[]>('get_sent', { token }),
  getMessage: (token: string, messageId: string) => invoke<any>('get_message', { token, messageId }),
  recall: (token: string, messageId: string) => invoke<void>('recall_message', { token, messageId }),
  markRead: (token: string, messageId: string) => invoke<void>('mark_message_read', { token, messageId }),
};

// Users
export const userApi = {
  getAll: (token: string) => invoke<any[]>('get_all_users', { token }),
  getRoles: (token: string) => invoke<any[]>('get_roles', { token }),
  create: (token: string, username: string, email: string, password: string, fullName: string, roleName: string, location?: string) =>
    invoke<string>('create_user', { token, username, email, password, fullName, roleName, location }),
  deactivate: (token: string, userId: string) => invoke<void>('deactivate_user', { token, userId }),
  delete: (token: string, userId: string) => invoke<void>('delete_user', { token, userId }),
};

// Research
export const researchApi = {
  getExperiments: (token: string) => invoke<any[]>('get_experiments', { token }),
  proposeExperiment: (token: string, title: string, description: string, experimentType: string, startDate?: string, endDate?: string) =>
    invoke<string>('propose_experiment', { token, title, description, experimentType, startDate, endDate }),
  reviewExperiment: (token: string, experimentId: string, status: string, notes?: string) =>
    invoke<void>('review_experiment', { token, experimentId, status, notes }),
  addLog: (token: string, experimentId: string, logDate: string, personnelPresent?: string, speciesMatterTested?: string, testsPerformed?: string, linkedTestIds?: string, notes?: string) =>
    invoke<string>('add_experiment_log', { token, experimentId, logDate, personnelPresent, speciesMatterTested, testsPerformed, linkedTestIds, notes }),
  getExperimentLogs: (token: string, experimentId: string) =>
    invoke<any[]>('get_experiment_logs', { token, experimentId }),
  requestConclusion: (token: string, experimentId: string, finalNotes: string, finalFindings?: string, methodologySummary?: string, keyResults?: string, recommendations?: string, limitations?: string) =>
    invoke<void>('request_experiment_conclusion', { token, experimentId, finalNotes, finalFindings, methodologySummary, keyResults, recommendations, limitations }),
  approveConclusion: (token: string, experimentId: string, decision: string, reviewNotes?: string) =>
    invoke<void>('approve_experiment_conclusion', { token, experimentId, decision, reviewNotes }),
  getSpeciesArchive: (token: string) => invoke<any[]>('get_species_archive', { token }),
  addSpecies: (token: string, name: string, classification?: string, description?: string, habitat?: string, speciesCategory?: string) =>
    invoke<string>('add_species', { token, name, classification, description, habitat, speciesCategory }),
  getTestArchive: (token: string) => invoke<any[]>('get_test_archive', { token }),
  proposeTest: (token: string, title: string, description?: string, methodology?: string) =>
    invoke<string>('propose_test', { token, title, description, methodology }),
  reviewTest: (token: string, proposalId: string, status: string, notes?: string) =>
    invoke<void>('review_test_proposal', { token, proposalId, status, notes }),
  linkTestsToLog: (token: string, logId: string, testIds: string[]) =>
    invoke<void>('link_tests_to_log', { token, logId, testIds }),
  proposeSpeciesFromDiscovery: (token: string, experimentId: string, speciesName: string, description?: string, classification?: string, habitat?: string) =>
    invoke<string>('propose_species_from_discovery', { token, experimentId, speciesName, description, classification, habitat }),
  getObserverDashboard: (token: string) =>
    invoke<any>('get_observer_dashboard', { token }),
  assignExperimentTask: (token: string, experimentId: string, assignedTo: string, title: string, dueDate?: string) =>
    invoke<string>('assign_experiment_task', { token, experimentId, assignedTo, title, dueDate }),
  getExperimentTasks: (token: string, experimentId: string) =>
    invoke<any[]>('get_experiment_tasks', { token, experimentId }),
};

// Data Services
export const dataApi = {
  submitRequest: (token: string, payload: {
    title: string;
    requestedDataItems: string;
    reasonOfRequest: string;
    description?: string;
    dataType?: string;
    requesterLocation: string;
    requesterTelFax: string;
    requesterDepartment: string;
    requesterDepartmentEmail: string;
  }) =>
    invoke<string>('submit_data_request', {
      token,
      title: payload.title,
      requested_data_items: payload.requestedDataItems,
      reason_of_request: payload.reasonOfRequest,
      description: payload.description,
      data_type: payload.dataType,
      requester_location: payload.requesterLocation,
      requester_tel_fax: payload.requesterTelFax,
      requester_department: payload.requesterDepartment,
      requester_department_email: payload.requesterDepartmentEmail
    }),
  getRequests: (token: string) => invoke<any[]>('get_data_requests', { token }),
  review: (token: string, requestId: string, status: string, notes?: string, assignedTo?: string) =>
    invoke<void>('review_data_request', { token, requestId, status, notes, assignedTo }),
  startProcessing: (token: string, requestId: string) =>
    invoke<void>('start_processing', { token, requestId }),
  analystSubmit: (token: string, payload: {
    requestId: string;
    responseMarkdown: string;
    responseStatus: 'provided' | 'rejected';
    responseExplanation?: string;
    analystNotes?: string;
    providedBy: string[];
    attachments: Array<{ filename: string; mimeType: string | null; base64: string }>;
  }) =>
    invoke<void>('analyst_submit_response', {
      token,
      request_id: payload.requestId,
      response_markdown: payload.responseMarkdown,
      response_status: payload.responseStatus,
      response_explanation: payload.responseExplanation,
      analyst_notes: payload.analystNotes,
      provided_by: payload.providedBy,
      attachments: payload.attachments.map(a => ({ filename: a.filename, mime_type: a.mimeType, base64: a.base64 }))
    }),
  analystReject: (token: string, requestId: string, rejectionReason: string) =>
    invoke<void>('analyst_reject_request', { token, requestId, rejectionReason }),
  deliver: (token: string, requestId: string) =>
    invoke<void>('deliver_data_response', { token, requestId }),
  acknowledge: (token: string, requestId: string) =>
    invoke<void>('acknowledge_data_response', { token, request_id: requestId }),
  listAttachments: (token: string, requestId: string) =>
    invoke<any[]>('list_data_response_attachments', { token, request_id: requestId }),
  downloadAttachment: (token: string, attachmentId: string) =>
    invoke<any>('download_data_response_attachment', { token, attachment_id: attachmentId }),
};

// Security
export const securityApi = {
  createIncident: (token: string, title: string, description: string, location?: string, incidentDate?: string, severity?: string) =>
    invoke<string>('create_incident_report', { token, title, description, location, incidentDate, severity }),
  getIncidents: (token: string) => invoke<any[]>('get_incident_reports', { token }),
  updateIncidentStatus: (token: string, reportId: string, status: string) =>
    invoke<void>('update_incident_status', { token, reportId, status }),
  addLostFound: (token: string, itemName: string, description?: string, foundLocation?: string, foundDate?: string) =>
    invoke<string>('add_lost_found_item', { token, itemName, description, foundLocation, foundDate }),
  getLostFound: (token: string) => invoke<any[]>('get_lost_found', { token }),
  claimLostFound: (token: string, itemId: string) => invoke<void>('claim_lost_found', { token, itemId }),
  submitBroadcastRequest: (token: string, title: string, content: string, targetAudience?: string, targetFilters?: any) =>
    invoke<string>('submit_broadcast_request', { token, title, content, targetAudience, targetFilters }),
  getBroadcastRequests: (token: string) => invoke<any[]>('get_broadcast_requests', { token }),
  sendBroadcastDirect: (token: string, title: string, content: string, targetFilters?: any, scheduledAt?: string) =>
    invoke<string>('send_broadcast_direct', { token, title, content, targetFilters, scheduledAt }),
  reviewBroadcast: (token: string, requestId: string, status: string, notes?: string, scheduledAt?: string) =>
    invoke<void>('review_broadcast_request', { token, requestId, status, notes, scheduledAt }),
  submitFindings: (token: string, title: string, description?: string, findingsDate?: string) =>
    invoke<string>('submit_security_findings', { token, title, description, findingsDate }),
  getFindings: (token: string) => invoke<any[]>('get_security_findings', { token }),
  assignTask: (token: string, assignedTo: string, title: string, description?: string, dueDate?: string) =>
    invoke<string>('assign_security_task', { token, assignedTo, title, description, dueDate }),
  getSecurityTasks: (token: string) => invoke<any[]>('get_security_tasks', { token }),
  updateSecurityTaskStatus: (token: string, taskId: string, status: string) =>
    invoke<void>('update_security_task_status', { token, taskId, status }),
  submitExternalReport: (token: string, title: string, description: string, securityType?: string) =>
    invoke<string>('submit_external_report', { token, title, description, securityType }),
  getExternalReports: (token: string) => invoke<any[]>('get_external_reports', { token }),
};

// Astronautics
export const astronautApi = {
  getMissions: (token: string) => invoke<any[]>('get_missions', { token }),
  createMission: (token: string, title: string, description?: string, missionType?: string, shipId?: string, sectorId?: string, planetId?: string, startDate?: string) =>
    invoke<string>('create_mission', { token, title, description, missionType, shipId, sectorId, planetId, startDate }),
  updateMissionStatus: (token: string, missionId: string, status: string) =>
    invoke<void>('update_mission_status', { token, missionId, status }),
  assignCrew: (token: string, missionId: string, astronautIds: string[]) =>
    invoke<void>('assign_crew', { token, missionId, astronautIds }),
  submitReport: (token: string, missionId: string, reportType: string, content: string) =>
    invoke<string>('submit_mission_report', { token, missionId, reportType, content }),
  getReports: (token: string, missionId: string) => invoke<any[]>('get_mission_reports', { token, missionId }),
  createJournal: (token: string, missionId?: string, title?: string, content?: string, isPublic?: boolean) =>
    invoke<string>('create_exploration_journal', { token, missionId, title, content, isPublic }),
  getJournals: (token: string) => invoke<any[]>('get_journals', { token }),
  getSectors: (token: string) => invoke<any[]>('get_sectors', { token }),
  createSector: (token: string, name: string, description?: string, boundaries?: string) =>
    invoke<string>('create_sector', { token, name, description, boundaries }),
  renameSector: (token: string, sectorId: string, newName: string) =>
    invoke<void>('rename_sector', { token, sectorId, newName }),
  getPlanets: (token: string) => invoke<any[]>('get_planets', { token }),
  createPlanet: (token: string, name: string, description?: string, starSystem?: string) =>
    invoke<string>('create_planet', { token, name, description, starSystem }),
  renamePlanet: (token: string, planetId: string, newName: string) =>
    invoke<void>('rename_planet', { token, planetId, newName }),
  getShips: (token: string) => invoke<any[]>('get_ships', { token }),
  getStats: (token: string, astronautId?: string) => invoke<any>('get_astronaut_stats', { token, astronautId }),
  submitConclusionRequest: (token: string, missionId: string, reportSummary?: string) =>
    invoke<string>('submit_conclusion_request', { token, missionId, reportSummary }),
  getConclusionRequests: (token: string) => invoke<any[]>('get_conclusion_requests', { token }),
  reviewConclusionRequest: (token: string, requestId: string, decision: string, reviewNotes?: string) =>
    invoke<void>('review_conclusion_request', { token, requestId, decision, reviewNotes }),
};

// Settlement
export const settlementApi = {
  getSettlements: (token: string) => invoke<any[]>('get_settlements', { token }),
  getTasks: (token: string) => invoke<any[]>('get_settler_tasks', { token }),
  assignTask: (token: string, assignedTo: string, title: string, description?: string, dueDate?: string, settlementId?: string) =>
    invoke<string>('assign_settler_task', { token, assignedTo, title, description, dueDate, settlementId }),
  updateTaskProgress: (token: string, taskId: string, progressNotes: string, status: string) =>
    invoke<void>('update_task_progress', { token, taskId, progressNotes, status }),
  submitSupplyRequest: (token: string, settlementId?: string, title?: string, description?: string, items?: string) =>
    invoke<string>('submit_supply_request', { token, settlementId, title, description, items }),
  reviewSupplyRequest: (token: string, requestId: string, decision: string, notes?: string) =>
    invoke<void>('review_supply_request', { token, requestId, decision, notes }),
  submitAnomalyReport: (token: string, settlementId?: string, title?: string, description?: string, severity?: string) =>
    invoke<string>('submit_anomaly_report', { token, settlementId, title, description, severity }),
  reviewAnomalyReport: (token: string, reportId: string, outcome: string) =>
    invoke<void>('review_anomaly_report', { token, reportId, outcome }),
  issueHouseArrest: (token: string, settlerId: string, settlementId?: string, reason?: string, startDate?: string, endDate?: string) =>
    invoke<string>('issue_house_arrest', { token, settlerId, settlementId, reason, startDate, endDate }),
  submitSendToEarth: (token: string, settlerId: string, reason: string) =>
    invoke<string>('submit_send_to_earth', { token, settlerId, reason }),
  reviewSendToEarth: (token: string, requestId: string, decision: string, notes?: string) =>
    invoke<void>('review_send_to_earth', { token, requestId, decision, notes }),
  logInventory: (token: string, settlementId: string, itemName: string, category?: string, quantity?: number, unit?: string) =>
    invoke<string>('log_settlement_inventory', { token, settlementId, itemName, category, quantity, unit }),
  getInventory: (token: string, settlementId: string) => invoke<any[]>('get_settlement_inventory', { token, settlementId }),
  submitFarmReport: (token: string, settlementId?: string, title?: string, content?: string, cropStatus?: string, healthCheckNotes?: string) =>
    invoke<string>('submit_farm_report', { token, settlementId, title, content, cropStatus, healthCheckNotes }),
  getSupplyRequests: (token: string) => invoke<any[]>('get_supply_requests', { token }),
  getAnomalyReports: (token: string) => invoke<any[]>('get_anomaly_reports', { token }),
  getFarmReports: (token: string) => invoke<any[]>('get_farm_reports', { token }),
  submitTroublesomeReport: (token: string, reportedSettlerId: string, description: string, settlementId?: string) =>
    invoke<string>('submit_troublesome_settler_report', { token, reportedSettlerId, description, settlementId }),
  getTroublesomeReports: (token: string) => invoke<any[]>('get_troublesome_settler_reports', { token }),
  submitCivilEngineerReport: (token: string, title: string, content: string, settlementId?: string, taskId?: string, materialsUsed?: string, progressPercentage?: number, problemsEncountered?: string, plansNextSteps?: string) =>
    invoke<string>('submit_civil_engineer_report', { token, title, content, settlementId, taskId, materialsUsed, progressPercentage, problemsEncountered, plansNextSteps }),
  getCivilEngineerReports: (token: string) => invoke<any[]>('get_civil_engineer_reports', { token }),
};

// Space Station
export const stationApi = {
  getStations: (token: string) => invoke<any[]>('get_stations', { token }),
  getInventory: (token: string, stationId: string) => invoke<any[]>('get_station_inventory', { token, stationId }),
  updateInventory: (token: string, stationId: string, category: string, itemName: string, quantity: number, unit?: string) =>
    invoke<string>('update_station_inventory', { token, stationId, category, itemName, quantity, unit }),
  addAnnotation: (token: string, stationId: string, sectionName: string, description?: string, xPosition?: number, yPosition?: number) =>
    invoke<string>('add_map_annotation', { token, stationId, sectionName, description, xPosition, yPosition }),
  getAnnotations: (token: string, stationId: string) => invoke<any[]>('get_map_annotations', { token, stationId }),
  logPersonnelEvent: (token: string, stationId: string, userId: string, eventType: string, notes?: string) =>
    invoke<string>('log_personnel_event', { token, stationId, userId, eventType, notes }),
  getPersonnelLog: (token: string, stationId: string) => invoke<any[]>('get_personnel_log', { token, stationId }),
  submitFindings: (token: string, stationId: string, title: string, description?: string, isPrivate?: boolean) =>
    invoke<string>('submit_station_findings', { token, stationId, title, description, isPrivate }),
  getFindings: (token: string, stationId?: string) => invoke<any[]>('get_station_findings', { token, stationId }),
  submitSupplyRequest: (token: string, stationId: string, title: string, items?: string, totalCost?: number) =>
    invoke<string>('submit_station_supply_request', { token, stationId, title, items, totalCost }),
  getStationSupplyRequests: (token: string, stationId?: string) =>
    invoke<any[]>('get_station_supply_requests', { token, stationId }),
  reviewSupplyRequest: (token: string, requestId: string, decision: string, notes?: string) =>
    invoke<void>('review_station_supply_request', { token, requestId, decision, notes }),
  submitAbandonment: (token: string, stationId: string, reason: string) =>
    invoke<string>('submit_station_abandonment', { token, stationId, reason }),
};

// Psychiatry
export const psychiatryApi = {
  registerPatient: (token: string, patientId: string) => invoke<string>('register_patient', { token, patientId }),
  getPatients: (token: string) => invoke<any[]>('get_patients', { token }),
  scheduleAppointment: (token: string, patientId: string, scheduledAt: string) =>
    invoke<string>('schedule_appointment', { token, patientId, scheduledAt }),
  getAppointments: (token: string) => invoke<any[]>('get_appointments', { token }),
  completeAppointment: (token: string, appointmentId: string, findings?: string) =>
    invoke<void>('complete_appointment', { token, appointmentId, findings }),
  addRecoveryLog: (token: string, patientId: string, entryDate: string, status: string, notes?: string) =>
    invoke<string>('add_recovery_log', { token, patientId, entryDate, status, notes }),
  getRecoveryLog: (token: string, patientId: string) => invoke<any[]>('get_recovery_log', { token, patientId }),
  assignTask: (token: string, assignedTo: string, title: string, description?: string, dueDate?: string) =>
    invoke<string>('assign_psychiatry_task', { token, assignedTo, title, description, dueDate }),
  getTasks: (token: string) => invoke<any[]>('get_psychiatry_tasks', { token }),
  updateTaskStatus: (token: string, taskId: string, status: string, progressNotes?: string) =>
    invoke<void>('update_psychiatry_task_status', { token, taskId, status, progressNotes }),
};

// Medical
export const medicalApi = {
  allocateShift: (token: string, staffId: string, shiftStart: string, shiftEnd: string, notes?: string) =>
    invoke<string>('allocate_shift', { token, staffId, shiftStart, shiftEnd, notes }),
  getShifts: (token: string) => invoke<any[]>('get_shifts', { token }),
  getInventory: (token: string) => invoke<any[]>('get_medical_inventory', { token }),
  updateInventory: (token: string, itemName: string, category?: string, quantity?: number, unit?: string, expiryDate?: string) =>
    invoke<string>('update_medical_inventory', { token, itemName, category, quantity, unit, expiryDate }),
  createRecord: (token: string, patientId: string, diagnosis?: string, treatment?: string, medications?: string, notes?: string) =>
    invoke<string>('create_patient_record', { token, patientId, diagnosis, treatment, medications, notes }),
  getRecords: (token: string, patientId: string) => invoke<any[]>('get_patient_records', { token, patientId }),
  addSpecialization: (token: string, staffId: string, specialization: string, certifiedAt?: string) =>
    invoke<string>('add_specialization', { token, staffId, specialization, certifiedAt }),
  getSpecializations: (token: string) => invoke<any[]>('get_staff_specializations', { token }),
  assignTask: (token: string, assignedTo: string, title: string, description?: string, dueDate?: string) =>
    invoke<string>('assign_medical_task', { token, assignedTo, title, description, dueDate }),
  getTasks: (token: string) => invoke<any[]>('get_medical_tasks', { token }),
  updateTaskStatus: (token: string, taskId: string, status: string, progressNotes?: string) =>
    invoke<void>('update_medical_task_status', { token, taskId, status, progressNotes }),
};

// Sanitary
export const sanitaryApi = {
  getTasks: (token: string) => invoke<any[]>('get_sanitary_tasks', { token }),
  assignTask: (token: string, assignedTo: string, title: string, description?: string, division?: string, dueDate?: string) =>
    invoke<string>('assign_sanitary_task', { token, assignedTo, title, description, division, dueDate }),
  updateTask: (token: string, taskId: string, status: string) =>
    invoke<void>('update_sanitary_task', { token, taskId, status }),
  getInventory: (token: string) => invoke<any[]>('get_sanitary_inventory', { token }),
  updateInventory: (token: string, itemName: string, category?: string, quantity?: number, unit?: string) =>
    invoke<string>('update_sanitary_inventory', { token, itemName, category, quantity, unit }),
  addDisposalLog: (token: string, itemName: string, quantity: number, unit?: string, disposalMethod?: string, hazardLevel?: string, notes?: string) =>
    invoke<string>('add_disposal_log', { token, itemName, quantity, unit, disposalMethod, hazardLevel, notes }),
  getDisposalLogs: (token: string) => invoke<any[]>('get_disposal_logs', { token }),
  addWastewaterLog: (token: string, volumeTreated: number, unit?: string, treatmentMethod?: string, phLevel?: number, qualityNotes?: string) =>
    invoke<string>('add_wastewater_log', { token, volumeTreated, unit, treatmentMethod, phLevel, qualityNotes }),
  getWastewaterLogs: (token: string) => invoke<any[]>('get_wastewater_logs', { token }),
  submitTransfer: (token: string, fromDivision: string, toDivision: string, reason?: string) =>
    invoke<string>('submit_division_transfer', { token, fromDivision, toDivision, reason }),
  reviewTransfer: (token: string, requestId: string, decision: string, notes?: string) =>
    invoke<void>('review_division_transfer', { token, requestId, decision, notes }),
  setQuota: (token: string, division: string, quotaType: string, targetValue: number, period?: string) =>
    invoke<string>('set_division_quota', { token, division, quotaType, targetValue, period }),
  createInspection: (token: string, location: string, inspectionDate: string, findings: string, violations?: string, recommendations?: string) =>
    invoke<string>('create_inspection_report', { token, location, inspectionDate, findings, violations, recommendations }),
  getInspections: (token: string) => invoke<any[]>('get_inspection_reports', { token }),
  sendToHead: (token: string, reportId: string) => invoke<void>('send_inspection_to_head', { token, reportId }),
};

// Governance
export const governanceApi = {
  initiateVote: (token: string, title: string, description?: string) =>
    invoke<string>('initiate_vote', { token, title, description }),
  castVote: (token: string, voteId: string, decision: string, reason: string) =>
    invoke<void>('cast_vote', { token, voteId, decision, reason }),
  getVotes: (token: string) => invoke<any[]>('get_votes', { token }),
  getVoteDetails: (token: string, voteId: string) => invoke<any>('get_vote_details', { token, voteId }),
  interruptVote: (token: string, voteId: string) => invoke<void>('interrupt_vote', { token, voteId }),
  createMeeting: (token: string, title: string, description?: string, scheduledAt?: string, location?: string, attendeeIds?: string[], toIds?: string[], ccIds?: string[], bccIds?: string[]) =>
    invoke<string>('create_meeting', { token, title, description, scheduledAt, location, attendeeIds: attendeeIds ?? [], toIds: toIds ?? [], ccIds: ccIds ?? [], bccIds: bccIds ?? [] }),
  getMeetings: (token: string) => invoke<any[]>('get_meetings', { token }),
  relocateStaff: (token: string, staffId: string, toLocation: string, relocationType: string, startDate?: string, endDate?: string, reason?: string) =>
    invoke<string>('relocate_staff', { token, staffId, toLocation, relocationType, startDate, endDate, reason }),
  getRelocations: (token: string) => invoke<any[]>('get_relocations', { token }),
  setArchivePermission: (token: string, tableName: string, recordId: string, accessLevel: string) =>
    invoke<void>('set_archive_permission', { token, tableName, recordId, accessLevel }),
  getArchivePermissions: (token: string) => invoke<any[]>('get_archive_permissions', { token }),
  redactRecord: (token: string, tableName: string, recordId: string, redactionReason?: string) =>
    invoke<void>('redact_record', { token, tableName, recordId, redactionReason }),
  deleteRecord: (token: string, tableName: string, recordId: string, deletionReason?: string) =>
    invoke<void>('librarian_delete_record', { token, tableName, recordId, deletionReason }),
  logEvent: (token: string, title: string, description?: string, eventDate?: string, venue?: string, venueInvoice?: string) =>
    invoke<string>('log_event_document', { token, title, description, eventDate, venue, venueInvoice }),
  getEvents: (token: string) => invoke<any[]>('get_event_documents', { token }),
  nomadAssignTask: (token: string, assignedTo: string, title: string, description?: string, dueDate?: string) =>
    invoke<string>('nomad_assign_task', { token, assignedTo, title, description, dueDate }),
  getNomadTasks: (token: string) => invoke<any[]>('get_nomad_tasks', { token }),
  updateNomadTaskStatus: (token: string, taskId: string, status: string) =>
    invoke<void>('update_nomad_task_status', { token, taskId, status }),
  directorCreateAccount: (token: string, username: string, email: string, password: string, fullName: string, roleName: string, location?: string) =>
    invoke<string>('director_create_account', { token, username, email, password, fullName, roleName, location }),
};

// Budget
export const budgetApi = {
  getBudgetRequests: (token: string) => invoke<any[]>('get_budget_requests', { token }),
  submitBudgetRequest: (token: string, title: string, description: string, amount: number, items?: string) =>
    invoke<string>('submit_budget_request', { token, title, description, amount, items }),
  reviewBudget: (token: string, requestId: string, status: string, notes?: string) =>
    invoke<void>('review_budget_request', { token, requestId, status, notes }),
  flagBudget: (token: string, requestId: string, reason: string) =>
    invoke<void>('flag_budget_request', { token, requestId, reason }),
  getExpenditures: (token: string) => invoke<any[]>('get_expenditure_reports', { token }),
  submitExpenditureReport: (token: string, title: string, description: string, totalAmount: number, items?: string, invoiceData?: string) =>
    invoke<string>('submit_expenditure_report', { token, title, description, totalAmount, items, invoiceData }),
  flagExpenditure: (token: string, reportId: string, reason: string) =>
    invoke<void>('flag_expenditure_report', { token, reportId, reason }),
  submitInvestigation: (token: string, title: string, description: string, relatedReportId?: string) =>
    invoke<string>('submit_investigation', { token, title, description, relatedReportId }),
  initiateBudgetVote: (token: string, requestId: string) =>
    invoke<string>('initiate_budget_vote', { token, requestId }),
};

// General Requests (auto-trigger director votes)
export const generalApi = {
  submitGeneralRequest: (token: string, title: string, description: string) =>
    invoke<string>('submit_general_request', { token, title, description }),
  getGeneralRequests: (token: string) => invoke<any[]>('get_general_requests', { token }),
  reviewGeneralRequest: (token: string, requestId: string, status: string, notes?: string) =>
    invoke<void>('review_general_request', { token, requestId, status, notes }),
};

// Aerospace Engineering
export const aerospaceApi = {
  getWorkOrders: (token: string) => invoke<any[]>('get_work_orders', { token }),
  createWorkOrder: (token: string, title: string, description?: string, priority?: string, systemAffected?: string) =>
    invoke<string>('create_work_order', { token, title, description, priority, systemAffected }),
  updateWorkOrderStatus: (token: string, workOrderId: string, status: string, notes?: string) =>
    invoke<void>('update_work_order_status', { token, workOrderId, status, notes }),
  submitTechnicalReport: (token: string, title: string, content: string, findings?: string, recommendations?: string) =>
    invoke<string>('submit_technical_report', { token, title, content, findings, recommendations }),
  getTechnicalReports: (token: string) => invoke<any[]>('get_technical_reports', { token }),
  assignTask: (token: string, assignedTo: string, title: string, description?: string, dueDate?: string) =>
    invoke<string>('assign_aerospace_task', { token, assignedTo, title, description, dueDate }),
  getAssignedTasks: (token: string) => invoke<any[]>('get_aerospace_assigned_tasks', { token }),
  updateTaskStatus: (token: string, taskId: string, status: string, progressNotes?: string) =>
    invoke<void>('update_aerospace_task_status', { token, taskId, status, progressNotes }),
  submitBlueprintProposal: (token: string, shipName: string, blueprintDescription: string, designSpecs?: string, shipId?: string) =>
    invoke<string>('submit_blueprint_proposal', { token, shipName, blueprintDescription, designSpecs, shipId }),
  getBlueprintProposals: (token: string) => invoke<any[]>('get_blueprint_proposals', { token }),
  reviewBlueprintProposal: (token: string, proposalId: string, status: string, notes?: string) =>
    invoke<void>('review_blueprint_proposal', { token, proposalId, status, notes }),
  updateShipStatus: (token: string, shipId: string, status: string) =>
    invoke<void>('update_ship_status', { token, shipId, status }),
  requestTaskConclusion: (token: string, taskId: string, finalNotes: string, finalFindings?: string, methodologySummary?: string, keyResults?: string, recommendations?: string, limitations?: string) =>
    invoke<void>('request_aerospace_task_conclusion', { token, taskId, finalNotes, finalFindings, methodologySummary, keyResults, recommendations, limitations }),
  approveTaskConclusion: (token: string, taskId: string, decision: string, reviewNotes?: string) =>
    invoke<void>('approve_aerospace_task_conclusion', { token, taskId, decision, reviewNotes }),
  getAllShips: (token: string) => invoke<any[]>('get_all_ships', { token }),
  getShipDetails: (token: string, shipId: string) => invoke<any>('get_ship_details', { token, shipId }),
  submitHelpRequest: (token: string, title: string, description?: string, category?: string) =>
    invoke<string>('submit_help_request', { token, title, description, category }),
  getHelpRequests: (token: string) => invoke<any[]>('get_help_requests', { token }),
  resolveHelpRequest: (token: string, requestId: string, status: string, response?: string) =>
    invoke<void>('resolve_help_request', { token, requestId, status, response }),
  rejectHelpRequest: (token: string, requestId: string, rejectionReason: string) =>
    invoke<void>('reject_help_request', { token, requestId, rejectionReason }),
  approveHelpRequest: (token: string, requestId: string, assignedToId: string) =>
    invoke<string>('approve_help_request', { token, requestId, assignedToId }),
  proxyDeliverTaskResponse: (token: string, requestId: string, response: string) =>
    invoke<void>('proxy_deliver_task_response', { token, requestId, response }),
};

// Admin
export const adminApi = {
  createDirector: (token: string, username: string, email: string, password: string, fullName: string, roleName: string) =>
    invoke<string>('admin_create_director', { token, username, email, password, fullName, roleName }),
  terminateDirector: (token: string, directorId: string) =>
    invoke<void>('admin_terminate_director', { token, directorId }),
  terminatePersonnel: (token: string, userId: string, reason?: string) =>
    invoke<void>('terminate_personnel', { token, userId, reason }),
  getAuditLog: (token: string, limit?: number, offset?: number) =>
    invoke<any[]>('get_audit_log', { token, limit, offset }),
  overrideVote: (token: string, voteId: string, outcome: string) =>
    invoke<void>('override_vote', { token, voteId, outcome }),
};

// Research Tasks (Observer/Artificer proxy chain)
export const researchTaskApi = {
  assign: (token: string, title: string, description?: string, assignedTo?: string, sourceMessageId?: string, dueDate?: string) =>
    invoke<string>('assign_research_task', { token, title, description, assignedTo, sourceMessageId, dueDate }),
  getTasks: (token: string) => invoke<any[]>('get_research_tasks', { token }),
  submitResult: (token: string, taskId: string, resultNotes: string) =>
    invoke<void>('submit_research_task_result', { token, taskId, resultNotes }),
  complete: (token: string, taskId: string) =>
    invoke<void>('complete_research_task', { token, taskId }),
};

// Types
export interface SessionData {
  user_id: string;
  username: string;
  full_name: string;
  email: string;
  role_name: string;
  role_display_name: string;
  tier: number;
  subsystem: string | null;
  location: string | null;
  tel_fax: string | null;
  department: string | null;
  department_email: string | null;
  token: string;
  /** Role names that this user's role inherits permissions from. */
  inherited_permissions: string[];
}
