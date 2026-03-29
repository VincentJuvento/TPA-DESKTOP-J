-- Migration 020: Security Reports for Research Department
-- Creates a dedicated security_reports table for researchers (Biologist, Chemist, Mathematician)
-- to submit security-related reports from their department modules.
-- Execution date: 2026-03-29

CREATE TABLE IF NOT EXISTS security_reports (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    submitted_by        UUID NOT NULL REFERENCES users(id),
    submitted_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    title               TEXT NOT NULL,
    category            TEXT NOT NULL,
    description         TEXT NOT NULL,
    severity            TEXT NOT NULL DEFAULT 'low',
    related_experiment_id UUID,
    related_task_id     UUID,
    status              TEXT NOT NULL DEFAULT 'new',
    security_staff_notes TEXT,
    resolved_at         TIMESTAMPTZ,
    attachments         JSONB,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at          TIMESTAMPTZ,
    CONSTRAINT security_reports_severity_check
        CHECK (severity IN ('low', 'medium', 'high', 'critical')),
    CONSTRAINT security_reports_status_check
        CHECK (status IN ('new', 'acknowledged', 'investigating', 'resolved', 'closed'))
);
