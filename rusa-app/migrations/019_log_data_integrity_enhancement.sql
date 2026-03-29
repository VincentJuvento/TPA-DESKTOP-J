-- Migration 019: Log Data Integrity Enhancement
-- Adds structured fields to experiment_logs for test outcomes,
-- relational personnel tracking, and raw data attachments.
-- Execution date: 2026-03-29

-- ─── 1. Test Outcome ─────────────────────────────────────────────────────────
-- Required for chemistry logs; enables future filtering and reporting.
ALTER TABLE experiment_logs
    ADD COLUMN IF NOT EXISTS test_outcome VARCHAR(50);

-- ─── 2. Personnel IDs (relational) ───────────────────────────────────────────
-- Replaces free-text personnel_present with a JSON array of user UUIDs.
-- Enables accountability tracking and facility audit trails.
ALTER TABLE experiment_logs
    ADD COLUMN IF NOT EXISTS personnel_ids JSONB;

-- ─── 3. Attachments / Raw Data ───────────────────────────────────────────────
-- JSONB array of attachment metadata objects (name, size, upload_timestamp).
ALTER TABLE experiment_logs
    ADD COLUMN IF NOT EXISTS attachments JSONB;
