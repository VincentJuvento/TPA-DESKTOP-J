-- External Reports: submitted by non-security personnel to the security team
CREATE TABLE IF NOT EXISTS external_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(300) NOT NULL,
    description TEXT NOT NULL,
    submitted_by UUID NOT NULL REFERENCES users(id),
    security_type VARCHAR(20) CHECK (security_type IN ('earth', 'galactic')),
    status VARCHAR(50) DEFAULT 'submitted' CHECK (status IN ('submitted', 'under_review', 'converted_to_incident', 'archived')),
    incident_report_id UUID REFERENCES incident_reports(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_external_reports_submitted_by ON external_reports(submitted_by);
CREATE INDEX IF NOT EXISTS idx_external_reports_status ON external_reports(status);

-- Troublesome Settler Reports: formal disciplinary complaint record
CREATE TABLE IF NOT EXISTS troublesome_settler_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    reported_settler_id UUID NOT NULL REFERENCES users(id),
    reported_by UUID NOT NULL REFERENCES users(id),
    settlement_id UUID REFERENCES settlements(id),
    description TEXT NOT NULL,
    status VARCHAR(50) DEFAULT 'pending' CHECK (status IN ('pending', 'house_arrest', 'deportation_requested', 'resolved')),
    house_arrest_id UUID REFERENCES house_arrests(id),
    send_to_earth_id UUID REFERENCES send_to_earth_requests(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_troublesome_reports_settler ON troublesome_settler_reports(reported_settler_id);
CREATE INDEX IF NOT EXISTS idx_troublesome_reports_settlement ON troublesome_settler_reports(settlement_id);

-- Civil Engineer Progress Reports: construction-specific progress tracking
CREATE TABLE IF NOT EXISTS civil_engineer_progress_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id UUID REFERENCES settler_tasks(id),
    settlement_id UUID REFERENCES settlements(id),
    title VARCHAR(300) NOT NULL,
    content TEXT NOT NULL,
    materials_used JSONB,
    progress_percentage INTEGER CHECK (progress_percentage >= 0 AND progress_percentage <= 100),
    problems_encountered TEXT,
    plans_next_steps TEXT,
    submitted_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_civil_reports_submitted_by ON civil_engineer_progress_reports(submitted_by);
CREATE INDEX IF NOT EXISTS idx_civil_reports_settlement ON civil_engineer_progress_reports(settlement_id);
CREATE INDEX IF NOT EXISTS idx_civil_reports_task ON civil_engineer_progress_reports(task_id);
