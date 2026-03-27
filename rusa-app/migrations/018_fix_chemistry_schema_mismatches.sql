-- Migration 018: Fix Chemistry Schema Mismatches (Post-PR #3)
-- Resolves fatal crashes from PR #3 merge due to missing columns and constraints.
-- Execution date: 2026-03-27
-- Critical: These fixes must run before any Chemistry or Help Request operations

-- ─── 1. Add assigned_proxy_director to help_requests ─────────────────────
-- Required by aerospace_commands.rs and chemistry_commands.rs
-- This column determines which director (the_observer or the_artificer) receives the request.
-- Backend Struct: HelpRequestRow expects this field (line 49 of aerospace_commands.rs)

ALTER TABLE help_requests
    ADD COLUMN IF NOT EXISTS assigned_proxy_director VARCHAR(100) NOT NULL DEFAULT 'the_artificer';

-- Backfill existing rows with default director (no prior routing data available)
UPDATE help_requests
SET assigned_proxy_director = 'the_artificer'
WHERE assigned_proxy_director IS NULL;

-- ─── 2. Update experiments.experiment_type CHECK constraint ───────────────
-- PR #3 introduced 'new_matter' experiments for Chemistry, but the constraint
-- in migration 001 still only allows 'standard' and 'new_species'.
-- This causes INSERT/UPDATE failures when chemistry experiments are created.
-- Backend Code: chemistry_commands.rs checks for 'new_matter' (lines 105, 190, 311)

ALTER TABLE experiments
    DROP CONSTRAINT IF EXISTS experiments_experiment_type_check;

ALTER TABLE experiments
    ADD CONSTRAINT experiments_experiment_type_check
    CHECK (experiment_type IN ('standard', 'new_species', 'new_matter'));
