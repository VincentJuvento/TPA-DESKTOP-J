-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Roles table
CREATE TABLE IF NOT EXISTS roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) UNIQUE NOT NULL,
    display_name VARCHAR(150) NOT NULL,
    tier INTEGER NOT NULL DEFAULT 1, -- 1=staff, 2=staff_lead, 3=director, 4=admin
    subsystem VARCHAR(50),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Users table
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(100) UNIQUE NOT NULL,
    email VARCHAR(200) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    full_name VARCHAR(200) NOT NULL,
    role_id UUID NOT NULL REFERENCES roles(id),
    location VARCHAR(200),
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- Sessions table
CREATE TABLE IF NOT EXISTS sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    token VARCHAR(255) UNIQUE NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Audit log table
CREATE TABLE IF NOT EXISTS audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    action VARCHAR(100) NOT NULL,
    table_name VARCHAR(100),
    record_id UUID,
    old_data JSONB,
    new_data JSONB,
    ip_address VARCHAR(50),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Messages table (internal email-esque messaging)
CREATE TABLE IF NOT EXISTS messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    from_user_id UUID NOT NULL REFERENCES users(id),
    subject VARCHAR(500) NOT NULL,
    body TEXT NOT NULL,
    scheduled_at TIMESTAMPTZ,
    sent_at TIMESTAMPTZ DEFAULT NOW(),
    recalled_at TIMESTAMPTZ,
    is_draft BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- Message recipients (To, CC, BCC)
CREATE TABLE IF NOT EXISTS message_recipients (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    message_id UUID NOT NULL REFERENCES messages(id),
    recipient_id UUID NOT NULL REFERENCES users(id),
    recipient_type VARCHAR(10) NOT NULL CHECK (recipient_type IN ('to', 'cc', 'bcc')),
    read_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ
);

-- ============= SS-02: Research & Lab Ops =============
CREATE TABLE IF NOT EXISTS experiments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(300) NOT NULL,
    description TEXT,
    proposed_by UUID NOT NULL REFERENCES users(id),
    assigned_by UUID REFERENCES users(id),
    status VARCHAR(50) DEFAULT 'pending' CHECK (status IN ('pending', 'approved', 'rejected', 'in_progress', 'completed', 'cancelled')),
    experiment_type VARCHAR(50) DEFAULT 'standard' CHECK (experiment_type IN ('standard', 'new_species')),
    start_date DATE,
    end_date DATE,
    director_vote_id UUID,
    reviewed_by UUID REFERENCES users(id),
    review_notes TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS experiment_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    experiment_id UUID NOT NULL REFERENCES experiments(id),
    log_date DATE NOT NULL,
    personnel_present TEXT,
    species_matter_tested TEXT,
    tests_performed TEXT,
    notes TEXT,
    is_completed BOOLEAN DEFAULT false,
    logged_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS species_archive (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(300) NOT NULL,
    classification VARCHAR(200),
    description TEXT,
    discovery_date DATE,
    discovered_by UUID REFERENCES users(id),
    habitat VARCHAR(200),
    status VARCHAR(50) DEFAULT 'known',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS test_archive (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(300) NOT NULL,
    description TEXT,
    methodology TEXT,
    category VARCHAR(100),
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS test_proposals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(300) NOT NULL,
    description TEXT,
    methodology TEXT,
    proposed_by UUID NOT NULL REFERENCES users(id),
    status VARCHAR(50) DEFAULT 'pending' CHECK (status IN ('pending', 'approved', 'rejected')),
    reviewed_by UUID REFERENCES users(id),
    review_notes TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- ============= SS-03: Theoretical Sciences =============
CREATE TABLE IF NOT EXISTS math_tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(300) NOT NULL,
    description TEXT,
    assigned_to UUID NOT NULL REFERENCES users(id),
    assigned_by UUID NOT NULL REFERENCES users(id),
    status VARCHAR(50) DEFAULT 'pending' CHECK (status IN ('pending', 'in_progress', 'submitted', 'approved', 'rejected')),
    result_latex TEXT,
    result_notes TEXT,
    due_date DATE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- ============= SS-04: Data Services =============
CREATE TABLE IF NOT EXISTS data_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    requested_by UUID NOT NULL REFERENCES users(id),
    title VARCHAR(300) NOT NULL,
    description TEXT NOT NULL,
    data_type VARCHAR(200),
    status VARCHAR(50) DEFAULT 'pending' CHECK (status IN ('pending', 'approved', 'rejected', 'processing', 'delivered')),
    reviewed_by UUID REFERENCES users(id),
    review_notes TEXT,
    response_data TEXT,
    responded_by UUID REFERENCES users(id),
    delivered_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- ============= SS-05: Global Security =============
CREATE TABLE IF NOT EXISTS incident_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(300) NOT NULL,
    description TEXT NOT NULL,
    location VARCHAR(200),
    incident_date TIMESTAMPTZ,
    severity VARCHAR(50) DEFAULT 'low' CHECK (severity IN ('low', 'medium', 'high', 'critical')),
    status VARCHAR(50) DEFAULT 'open' CHECK (status IN ('open', 'under_investigation', 'resolved', 'closed')),
    reported_by UUID NOT NULL REFERENCES users(id),
    security_type VARCHAR(20) CHECK (security_type IN ('earth', 'galactic')),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS lost_and_found (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    item_name VARCHAR(300) NOT NULL,
    description TEXT,
    found_location VARCHAR(200),
    found_date DATE,
    status VARCHAR(50) DEFAULT 'unclaimed' CHECK (status IN ('unclaimed', 'claimed', 'disposed')),
    logged_by UUID NOT NULL REFERENCES users(id),
    claimed_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS broadcast_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    requested_by UUID NOT NULL REFERENCES users(id),
    title VARCHAR(300) NOT NULL,
    content TEXT NOT NULL,
    target_audience VARCHAR(200),
    status VARCHAR(50) DEFAULT 'pending' CHECK (status IN ('pending', 'approved', 'rejected', 'broadcast')),
    reviewed_by UUID REFERENCES users(id),
    review_notes TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS security_findings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(300) NOT NULL,
    description TEXT,
    findings_date DATE,
    reported_by UUID NOT NULL REFERENCES users(id),
    security_type VARCHAR(20) CHECK (security_type IN ('earth', 'galactic')),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- ============= SS-07: Astronautics =============
CREATE TABLE IF NOT EXISTS sectors (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(300) NOT NULL,
    description TEXT,
    boundaries TEXT,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS planets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(300) NOT NULL,
    sector_id UUID REFERENCES sectors(id),
    description TEXT,
    coordinates VARCHAR(200),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS ships (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(300) NOT NULL,
    model VARCHAR(200),
    capacity INTEGER,
    ship_type VARCHAR(100),
    status VARCHAR(50) DEFAULT 'available' CHECK (status IN ('available', 'in_mission', 'maintenance', 'decommissioned')),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS missions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(300) NOT NULL,
    description TEXT,
    mission_type VARCHAR(20) NOT NULL CHECK (mission_type IN ('interstellar', 'terrain')),
    status VARCHAR(50) DEFAULT 'planned' CHECK (status IN ('planned', 'in_transit', 'on_location', 'returning', 'completed', 'cancelled')),
    created_by UUID NOT NULL REFERENCES users(id),
    ship_id UUID REFERENCES ships(id),
    sector_id UUID REFERENCES sectors(id),
    planet_id UUID REFERENCES planets(id),
    start_date DATE,
    end_date DATE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS mission_crew (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    mission_id UUID NOT NULL REFERENCES missions(id),
    user_id UUID NOT NULL REFERENCES users(id),
    role VARCHAR(100),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS mission_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    mission_id UUID NOT NULL REFERENCES missions(id),
    submitted_by UUID NOT NULL REFERENCES users(id),
    report_type VARCHAR(50) CHECK (report_type IN ('status', 'completion_request')),
    content TEXT NOT NULL,
    status VARCHAR(50) DEFAULT 'submitted' CHECK (status IN ('submitted', 'acknowledged', 'approved', 'rejected')),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS exploration_journals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    mission_id UUID REFERENCES missions(id),
    author_id UUID NOT NULL REFERENCES users(id),
    title VARCHAR(300) NOT NULL,
    content TEXT NOT NULL,
    is_public BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- Astronaut mission counters
CREATE TABLE IF NOT EXISTS astronaut_stats (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    astronaut_id UUID UNIQUE NOT NULL REFERENCES users(id),
    interstellar_count INTEGER DEFAULT 0,
    terrain_count INTEGER DEFAULT 0,
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- ============= SS-08: Settlement Ops =============
CREATE TABLE IF NOT EXISTS settlements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(300) NOT NULL,
    planet_id UUID REFERENCES planets(id),
    location VARCHAR(300),
    status VARCHAR(50) DEFAULT 'active' CHECK (status IN ('active', 'abandoned', 'pending_abandonment')),
    commander_id UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS settler_tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    settlement_id UUID REFERENCES settlements(id),
    assigned_to UUID NOT NULL REFERENCES users(id),
    assigned_by UUID NOT NULL REFERENCES users(id),
    title VARCHAR(300) NOT NULL,
    description TEXT,
    status VARCHAR(50) DEFAULT 'pending' CHECK (status IN ('pending', 'in_progress', 'completed', 'rejected')),
    due_date DATE,
    progress_notes TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS supply_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    requested_by UUID NOT NULL REFERENCES users(id),
    settlement_id UUID REFERENCES settlements(id),
    title VARCHAR(300) NOT NULL,
    description TEXT,
    items JSONB,
    total_cost DECIMAL(15,2),
    status VARCHAR(50) DEFAULT 'pending' CHECK (status IN ('pending', 'commander_approved', 'commander_rejected', 'director_approved', 'director_rejected')),
    commander_notes TEXT,
    director_notes TEXT,
    reviewed_by UUID REFERENCES users(id),
    notes TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS anomaly_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    settlement_id UUID REFERENCES settlements(id),
    reported_by UUID NOT NULL REFERENCES users(id),
    title VARCHAR(300) NOT NULL,
    description TEXT NOT NULL,
    severity VARCHAR(50) DEFAULT 'low' CHECK (severity IN ('low', 'medium', 'high', 'critical')),
    outcome VARCHAR(50) CHECK (outcome IN ('rejected', 'forwarded', 'abandonment_escalated')),
    status VARCHAR(50) DEFAULT 'open' CHECK (status IN ('open', 'under_review', 'resolved', 'escalated')),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS house_arrests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    settler_id UUID NOT NULL REFERENCES users(id),
    settlement_id UUID REFERENCES settlements(id),
    reason TEXT NOT NULL,
    ordered_by UUID NOT NULL REFERENCES users(id),
    start_date DATE,
    end_date DATE,
    status VARCHAR(50) DEFAULT 'active' CHECK (status IN ('active', 'lifted')),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS send_to_earth_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    settler_id UUID NOT NULL REFERENCES users(id),
    requested_by UUID NOT NULL REFERENCES users(id),
    reason TEXT NOT NULL,
    status VARCHAR(50) DEFAULT 'pending' CHECK (status IN ('pending', 'approved', 'rejected')),
    director_notes TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS settlement_inventory (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    settlement_id UUID NOT NULL REFERENCES settlements(id),
    item_name VARCHAR(300) NOT NULL,
    category VARCHAR(100),
    quantity INTEGER DEFAULT 0,
    unit VARCHAR(50),
    logged_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS farm_progress_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    settlement_id UUID REFERENCES settlements(id),
    farmer_id UUID NOT NULL REFERENCES users(id),
    title VARCHAR(300) NOT NULL,
    content TEXT NOT NULL,
    crop_status TEXT,
    health_check_notes TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- ============= SS-09: Space Station Management =============
CREATE TABLE IF NOT EXISTS stations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(300) NOT NULL,
    location VARCHAR(300),
    station_type VARCHAR(100),
    status VARCHAR(50) DEFAULT 'active' CHECK (status IN ('active', 'maintenance', 'abandoned')),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS station_inventory (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    station_id UUID REFERENCES stations(id),
    category VARCHAR(100) NOT NULL CHECK (category IN ('food', 'clothing', 'bedsheets', 'equipment', 'medical', 'other')),
    item_name VARCHAR(300) NOT NULL,
    quantity INTEGER DEFAULT 0,
    unit VARCHAR(50),
    logged_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS station_map_annotations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    station_id UUID REFERENCES stations(id),
    section_name VARCHAR(200) NOT NULL,
    description TEXT,
    x_position DECIMAL(8,2),
    y_position DECIMAL(8,2),
    annotated_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS station_personnel_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    station_id UUID REFERENCES stations(id),
    user_id UUID NOT NULL REFERENCES users(id),
    event_type VARCHAR(20) CHECK (event_type IN ('boarding', 'departing')),
    event_date TIMESTAMPTZ DEFAULT NOW(),
    notes TEXT,
    logged_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS station_findings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    station_id UUID REFERENCES stations(id),
    reported_by UUID NOT NULL REFERENCES users(id),
    title VARCHAR(300) NOT NULL,
    description TEXT,
    is_private BOOLEAN DEFAULT false,
    reported_to_security BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS station_supply_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    station_id UUID REFERENCES stations(id),
    requested_by UUID NOT NULL REFERENCES users(id),
    title VARCHAR(300) NOT NULL,
    items JSONB,
    total_cost DECIMAL(15,2),
    status VARCHAR(50) DEFAULT 'pending' CHECK (status IN ('pending', 'approved', 'rejected')),
    reviewed_by UUID REFERENCES users(id),
    notes TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS station_abandonment_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    station_id UUID REFERENCES stations(id),
    requested_by UUID NOT NULL REFERENCES users(id),
    reason TEXT NOT NULL,
    status VARCHAR(50) DEFAULT 'pending' CHECK (status IN ('pending', 'approved', 'rejected')),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- ============= SS-10: Clinical Psychiatry =============
CREATE TABLE IF NOT EXISTS psychiatric_patients (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    patient_id UUID UNIQUE NOT NULL REFERENCES users(id),
    notes TEXT,
    registered_at TIMESTAMPTZ DEFAULT NOW(),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS appointments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    psychiatrist_id UUID NOT NULL REFERENCES users(id),
    patient_id UUID NOT NULL REFERENCES users(id),
    scheduled_at TIMESTAMPTZ NOT NULL,
    status VARCHAR(50) DEFAULT 'scheduled' CHECK (status IN ('scheduled', 'completed', 'cancelled', 'no_show')),
    findings TEXT,
    is_findings_hidden BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS patient_recovery_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    patient_id UUID NOT NULL REFERENCES users(id),
    psychiatrist_id UUID NOT NULL REFERENCES users(id),
    entry_date DATE NOT NULL,
    status VARCHAR(100),
    notes TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- ============= SS-11: Medical Services =============
CREATE TABLE IF NOT EXISTS medical_shifts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    staff_id UUID NOT NULL REFERENCES users(id),
    allocated_by UUID NOT NULL REFERENCES users(id),
    shift_start TIMESTAMPTZ NOT NULL,
    shift_end TIMESTAMPTZ NOT NULL,
    notes TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS medical_inventory (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    item_name VARCHAR(300) NOT NULL,
    category VARCHAR(100),
    quantity INTEGER DEFAULT 0,
    unit VARCHAR(50),
    expiry_date DATE,
    location VARCHAR(200),
    logged_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS patient_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    patient_id UUID NOT NULL REFERENCES users(id),
    treated_by UUID NOT NULL REFERENCES users(id),
    diagnosis TEXT,
    treatment TEXT,
    medications TEXT,
    visit_date TIMESTAMPTZ DEFAULT NOW(),
    notes TEXT,
    is_confidential BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS staff_specializations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    staff_id UUID NOT NULL REFERENCES users(id),
    specialization VARCHAR(200) NOT NULL,
    certified_at DATE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- ============= SS-12: Sanitary & Waste =============
CREATE TABLE IF NOT EXISTS sanitary_staff_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    staff_id UUID NOT NULL REFERENCES users(id),
    division VARCHAR(50) NOT NULL CHECK (division IN ('cleanup', 'disposal', 'wastewater', 'transport', 'inspector')),
    assigned_by UUID REFERENCES users(id),
    assigned_at TIMESTAMPTZ DEFAULT NOW(),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS sanitary_tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    assigned_to UUID NOT NULL REFERENCES users(id),
    assigned_by UUID NOT NULL REFERENCES users(id),
    title VARCHAR(300) NOT NULL,
    description TEXT,
    division VARCHAR(50),
    status VARCHAR(50) DEFAULT 'pending' CHECK (status IN ('pending', 'in_progress', 'completed')),
    due_date DATE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS sanitary_inventory (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    item_name VARCHAR(300) NOT NULL,
    category VARCHAR(100),
    quantity INTEGER DEFAULT 0,
    unit VARCHAR(50),
    logged_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS disposal_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    logged_by UUID NOT NULL REFERENCES users(id),
    item_name VARCHAR(300) NOT NULL,
    quantity DECIMAL(10,3),
    unit VARCHAR(50),
    disposal_method VARCHAR(200),
    hazard_level VARCHAR(50),
    notes TEXT,
    disposal_date DATE DEFAULT CURRENT_DATE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS wastewater_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    logged_by UUID NOT NULL REFERENCES users(id),
    volume_treated DECIMAL(12,3),
    unit VARCHAR(50) DEFAULT 'liters',
    treatment_method VARCHAR(200),
    ph_level DECIMAL(5,2),
    quality_notes TEXT,
    treatment_date DATE DEFAULT CURRENT_DATE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS division_transfer_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    requested_by UUID NOT NULL REFERENCES users(id),
    from_division VARCHAR(50) NOT NULL,
    to_division VARCHAR(50) NOT NULL,
    reason TEXT,
    status VARCHAR(50) DEFAULT 'pending' CHECK (status IN ('pending', 'approved', 'rejected')),
    reviewed_by UUID REFERENCES users(id),
    review_notes TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS sanitary_quotas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    division VARCHAR(50) NOT NULL,
    quota_type VARCHAR(200) NOT NULL,
    target_value INTEGER,
    period VARCHAR(50),
    set_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- ============= SS-13: Sanitary Inspection =============
CREATE TABLE IF NOT EXISTS inspection_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    inspector_id UUID NOT NULL REFERENCES users(id),
    location VARCHAR(300) NOT NULL,
    inspection_date DATE NOT NULL,
    findings TEXT NOT NULL,
    violations TEXT,
    recommendations TEXT,
    status VARCHAR(50) DEFAULT 'submitted' CHECK (status IN ('submitted', 'acknowledged', 'action_taken')),
    sent_to_head BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- ============= SS-14: Budget & Finance =============
CREATE TABLE IF NOT EXISTS budget_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    requested_by UUID NOT NULL REFERENCES users(id),
    title VARCHAR(300) NOT NULL,
    description TEXT NOT NULL,
    amount DECIMAL(15,2) NOT NULL,
    items JSONB,
    status VARCHAR(50) DEFAULT 'pending' CHECK (status IN ('pending', 'under_review', 'approved', 'rejected', 'flagged')),
    accountant_notes TEXT,
    vote_id UUID,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS expenditure_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    reported_by UUID NOT NULL REFERENCES users(id),
    title VARCHAR(300) NOT NULL,
    description TEXT NOT NULL,
    total_amount DECIMAL(15,2) NOT NULL,
    items JSONB,
    invoice_data TEXT,
    is_flagged BOOLEAN DEFAULT false,
    flag_reason TEXT,
    status VARCHAR(50) DEFAULT 'submitted' CHECK (status IN ('submitted', 'reviewed', 'approved', 'flagged_for_investigation')),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- ============= SS-15: Governance =============
CREATE TABLE IF NOT EXISTS votes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(300) NOT NULL,
    description TEXT,
    initiated_by UUID NOT NULL REFERENCES users(id),
    status VARCHAR(50) DEFAULT 'open' CHECK (status IN ('open', 'snoozed', 'passed', 'failed', 'interrupted')),
    session_time VARCHAR(20),
    quorum_required INTEGER DEFAULT 8,
    yay_count INTEGER DEFAULT 0,
    nay_count INTEGER DEFAULT 0,
    abstain_count INTEGER DEFAULT 0,
    interrupted_by UUID REFERENCES users(id),
    deadline TIMESTAMPTZ,
    snooze_count INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS vote_ballots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vote_id UUID NOT NULL REFERENCES votes(id),
    voter_id UUID NOT NULL REFERENCES users(id),
    decision VARCHAR(10) CHECK (decision IN ('yes', 'no', 'abstain')),
    voted_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(vote_id, voter_id)
);

CREATE TABLE IF NOT EXISTS meetings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(300) NOT NULL,
    description TEXT,
    scheduled_at TIMESTAMPTZ NOT NULL,
    location VARCHAR(300),
    created_by UUID NOT NULL REFERENCES users(id),
    status VARCHAR(50) DEFAULT 'scheduled' CHECK (status IN ('scheduled', 'ongoing', 'completed', 'cancelled')),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS meeting_attendees (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    meeting_id UUID NOT NULL REFERENCES meetings(id),
    user_id UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS staff_relocations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    staff_id UUID NOT NULL REFERENCES users(id),
    from_location VARCHAR(300),
    to_location VARCHAR(300) NOT NULL,
    relocation_type VARCHAR(20) DEFAULT 'permanent' CHECK (relocation_type IN ('permanent', 'temporary')),
    requested_by UUID NOT NULL REFERENCES users(id),
    start_date DATE,
    end_date DATE,
    reason TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS archive_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    table_name VARCHAR(100) NOT NULL,
    record_id UUID NOT NULL,
    access_level VARCHAR(50) DEFAULT 'public',
    set_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS event_documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(300) NOT NULL,
    description TEXT,
    event_date TIMESTAMPTZ,
    venue VARCHAR(300),
    venue_invoice TEXT,
    logged_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- ============= Help Requests (General) =============
CREATE TABLE IF NOT EXISTS help_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    requested_by UUID NOT NULL REFERENCES users(id),
    title VARCHAR(300) NOT NULL,
    description TEXT NOT NULL,
    category VARCHAR(100),
    status VARCHAR(50) DEFAULT 'open' CHECK (status IN ('open', 'in_progress', 'resolved', 'closed')),
    response TEXT,
    resolved_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- ============= Seed Roles =============
INSERT INTO roles (id, name, display_name, tier, subsystem) VALUES
-- Tier 1: Staff
(gen_random_uuid(), 'biologist', 'Biologist', 1, 'research'),
(gen_random_uuid(), 'chemist', 'Chemist', 1, 'research'),
(gen_random_uuid(), 'physicist', 'Physicist', 1, 'research'),
(gen_random_uuid(), 'biological_engineer', 'Biological Engineer', 1, 'research'),
(gen_random_uuid(), 'agricultural_engineer', 'Agricultural Engineer', 1, 'research'),
(gen_random_uuid(), 'mathematician', 'Mathematician', 1, 'theoretical_sciences'),
(gen_random_uuid(), 'data_analyst', 'Data Analyst', 1, 'data_services'),
(gen_random_uuid(), 'earth_security_head', 'Earth Security Head', 2, 'security'),
(gen_random_uuid(), 'earth_security_staff', 'Earth Security Staff', 1, 'security'),
(gen_random_uuid(), 'galactic_security_head', 'Galactic Security Head', 2, 'security'),
(gen_random_uuid(), 'galactic_security_staff', 'Galactic Security Staff', 1, 'security'),
(gen_random_uuid(), 'astronaut', 'Astronaut', 1, 'astronautics'),
(gen_random_uuid(), 'aerospace_engineer', 'Aerospace Engineer', 1, 'astronautics'),
(gen_random_uuid(), 'settler_commander', 'Settler Commander', 2, 'settlement'),
(gen_random_uuid(), 'civil_engineer', 'Civil Engineer (Settler)', 1, 'settlement'),
(gen_random_uuid(), 'farmer', 'Farmer', 1, 'settlement'),
(gen_random_uuid(), 'space_station_settler', 'Space Station Settler', 1, 'space_station'),
(gen_random_uuid(), 'psychiatrist', 'Psychiatrist', 2, 'psychiatry'),
(gen_random_uuid(), 'psychiatrist_assistant', 'Psychiatrist Assistant', 1, 'psychiatry'),
(gen_random_uuid(), 'head_of_medicine', 'Head of Medicine', 2, 'medical'),
(gen_random_uuid(), 'medical_staff', 'Medical Staff', 1, 'medical'),
(gen_random_uuid(), 'head_of_sanitary', 'Head of Sanitary', 2, 'sanitary'),
(gen_random_uuid(), 'cleanup_crew', 'Cleanup Crew', 1, 'sanitary'),
(gen_random_uuid(), 'disposal_crew', 'Disposal Crew', 1, 'sanitary'),
(gen_random_uuid(), 'wastewater_crew', 'Wastewater Crew', 1, 'sanitary'),
(gen_random_uuid(), 'transport_crew', 'Transport Crew', 1, 'sanitary'),
(gen_random_uuid(), 'sanitary_inspector', 'Sanitary Inspector', 1, 'sanitary_inspection'),
-- Tier 3: Directors
(gen_random_uuid(), 'the_observer', 'The Observer (Director)', 3, 'research'),
(gen_random_uuid(), 'the_artificer', 'The Artificer (Director)', 3, 'theoretical_sciences'),
(gen_random_uuid(), 'the_statistician', 'The Statistician (Director)', 3, 'data_services'),
(gen_random_uuid(), 'the_guardian', 'The Guardian (Director)', 3, 'security'),
(gen_random_uuid(), 'the_anchorman', 'The Anchorman (Director)', 3, 'communications'),
(gen_random_uuid(), 'the_wanderer', 'The Wanderer (Director)', 3, 'astronautics'),
(gen_random_uuid(), 'the_taskmaster', 'The Taskmaster (Director)', 3, 'astronautics'),
(gen_random_uuid(), 'the_coordinator', 'The Coordinator (Director)', 3, 'governance'),
(gen_random_uuid(), 'the_accountant', 'The Accountant (Director)', 3, 'finance'),
(gen_random_uuid(), 'the_librarian', 'The Librarian (Director)', 3, 'governance'),
(gen_random_uuid(), 'the_nomad', 'The Nomad (Director)', 3, 'governance'),
(gen_random_uuid(), 'the_overseer', 'The Overseer (Director)', 3, 'security'),
(gen_random_uuid(), 'the_director', 'The Director (Director)', 3, 'governance'),
-- Tier 4: Admin
(gen_random_uuid(), 'the_administrator', 'The Administrator (Admin)', 4, 'administration')
ON CONFLICT (name) DO NOTHING;

-- ============= SS-06: Aerospace Engineering =============
CREATE TABLE IF NOT EXISTS aerospace_work_orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(300) NOT NULL,
    description TEXT,
    priority VARCHAR(50) DEFAULT 'medium' CHECK (priority IN ('low', 'medium', 'high', 'critical')),
    system_affected VARCHAR(200),
    status VARCHAR(50) DEFAULT 'open' CHECK (status IN ('open', 'in_progress', 'completed', 'cancelled')),
    assigned_to UUID REFERENCES users(id),
    notes TEXT,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS aerospace_technical_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(300) NOT NULL,
    content TEXT NOT NULL,
    findings TEXT,
    recommendations TEXT,
    submitted_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);
