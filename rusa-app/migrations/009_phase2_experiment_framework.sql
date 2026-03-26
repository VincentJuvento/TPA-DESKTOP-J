-- Phase 2: Complete Experiment & Logging Framework

-- Add structured conclusion fields to experiments
ALTER TABLE experiments
    ADD COLUMN IF NOT EXISTS final_findings TEXT,
    ADD COLUMN IF NOT EXISTS methodology_summary TEXT,
    ADD COLUMN IF NOT EXISTS key_results TEXT,
    ADD COLUMN IF NOT EXISTS recommendations TEXT,
    ADD COLUMN IF NOT EXISTS limitations TEXT;

-- Add test linking and species discovery fields to experiment_logs
ALTER TABLE experiment_logs
    ADD COLUMN IF NOT EXISTS linked_test_ids TEXT,
    ADD COLUMN IF NOT EXISTS new_species_proposed UUID,
    ADD COLUMN IF NOT EXISTS new_species_description TEXT;

-- Add discovery and approval tracking to species_archive
ALTER TABLE species_archive
    ADD COLUMN IF NOT EXISTS discovery_experiment_id UUID REFERENCES experiments(id),
    ADD COLUMN IF NOT EXISTS approval_status VARCHAR(50) NOT NULL DEFAULT 'approved',
    ADD COLUMN IF NOT EXISTS approved_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS approved_by UUID REFERENCES users(id);

-- New table for Observer experiment task assignments
CREATE TABLE IF NOT EXISTS research_task_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    assigned_by UUID REFERENCES users(id),
    assigned_to UUID REFERENCES users(id),
    experiment_id UUID REFERENCES experiments(id),
    title TEXT NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    due_date TIMESTAMPTZ,
    progress_notes TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);
