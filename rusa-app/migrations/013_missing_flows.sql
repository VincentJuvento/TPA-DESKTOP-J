-- Migration 013: Missing flows for Aerospace, Biological, and Agricultural Engineers
-- 1. Add species_category to species_archive for plant-only filtering
-- 2. Add conclusion fields to aerospace_assigned_tasks
-- 3. Create help_requests table with role-based routing
-- 4. Enhance ships table with build lifecycle fields

-- ─── 1. Species Category ────────────────────────────────────────────────────
ALTER TABLE species_archive
    ADD COLUMN IF NOT EXISTS species_category VARCHAR(100) DEFAULT 'unknown';

-- Backfill any existing rows as 'unknown' (already handled by default)
-- The frontend/backend will require category on new inserts.

-- ─── 2. Aerospace Task Conclusion Fields ────────────────────────────────────
ALTER TABLE aerospace_assigned_tasks
    ADD COLUMN IF NOT EXISTS conclusion_requested_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS conclusion_requested_by UUID REFERENCES users(id),
    ADD COLUMN IF NOT EXISTS conclusion_approved_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS conclusion_approved_by UUID REFERENCES users(id),
    ADD COLUMN IF NOT EXISTS final_notes TEXT,
    ADD COLUMN IF NOT EXISTS final_findings TEXT,
    ADD COLUMN IF NOT EXISTS methodology_summary TEXT,
    ADD COLUMN IF NOT EXISTS key_results TEXT,
    ADD COLUMN IF NOT EXISTS recommendations TEXT,
    ADD COLUMN IF NOT EXISTS limitations TEXT;

-- Extend status check to allow new states (drop old constraint if it exists)
ALTER TABLE aerospace_assigned_tasks
    DROP CONSTRAINT IF EXISTS aerospace_assigned_tasks_status_check;

-- ─── 3. Help Requests Table ─────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS help_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    requested_by UUID NOT NULL REFERENCES users(id),
    title TEXT NOT NULL,
    description TEXT,
    category VARCHAR(100),
    assigned_proxy_director VARCHAR(100) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'open'
        CHECK (status IN ('open', 'in_review', 'converted', 'resolved', 'closed')),
    response TEXT,
    resolved_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- ─── 4. Ships Table Enhancements ────────────────────────────────────────────
-- Extend ships table with build-lifecycle columns
ALTER TABLE ships
    ADD COLUMN IF NOT EXISTS ship_name TEXT,
    ADD COLUMN IF NOT EXISTS build_status VARCHAR(50),
    ADD COLUMN IF NOT EXISTS blueprint_approver UUID REFERENCES users(id),
    ADD COLUMN IF NOT EXISTS launch_date TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS materials_used TEXT,
    ADD COLUMN IF NOT EXISTS last_updated TIMESTAMPTZ DEFAULT NOW();

-- Backfill ship_name from existing name column where present
UPDATE ships SET ship_name = name WHERE ship_name IS NULL AND name IS NOT NULL;

-- Extend ships status to include design→building→completed lifecycle
ALTER TABLE ships DROP CONSTRAINT IF EXISTS ships_status_check;
ALTER TABLE ships
    ADD CONSTRAINT ships_status_check
    CHECK (status IN ('design', 'building', 'available', 'completed', 'in_mission', 'maintenance', 'decommissioned'));
