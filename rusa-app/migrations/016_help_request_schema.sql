-- Migration 016: Enforce Help Request Proxy Director Pipeline
-- Adds rejection tracking fields, task linking, and updates status constraint.

-- ─── Add rejection and task-link fields ─────────────────────────────────────
ALTER TABLE help_requests
    ADD COLUMN IF NOT EXISTS rejection_reason TEXT,        -- reason provided by the director when rejecting
    ADD COLUMN IF NOT EXISTS rejected_at TIMESTAMPTZ,      -- timestamp when the request was rejected
    ADD COLUMN IF NOT EXISTS created_task_id UUID;         -- UUID of the task created when request is approved

-- ─── Update status constraint to include 'rejected' ─────────────────────────
ALTER TABLE help_requests DROP CONSTRAINT IF EXISTS help_requests_status_check;
ALTER TABLE help_requests
    ADD CONSTRAINT help_requests_status_check
    CHECK (status IN ('open', 'in_review', 'rejected', 'converted', 'resolved', 'closed'));
