-- Migration 018: Fix Chemistry Schema Mismatches (Post-PR #3)
-- Resolves fatal crashes from PR #3 merge due to missing columns and constraints.
-- Execution date: 2026-03-27
-- Critical: These fixes must run before any Chemistry or Help Request operations

-- ─── 1. Add assigned_proxy_director to help_requests ─────────────────────
-- Required by aerospace_commands.rs and chemistry_commands.rs
-- This column determines which director (the_observer or the_artificer) receives the request.
-- Backend Struct: HelpRequestRow expects this field (line 49 of aerospace_commands.rs)

-- Add column as nullable first so the backfill below can target all existing rows.
-- If the column already exists (e.g. from migration 013) this is a safe no-op.
ALTER TABLE help_requests
    ADD COLUMN IF NOT EXISTS assigned_proxy_director VARCHAR(100);

-- Backfill ALL existing rows using the same routing logic as submit_help_request:
--   biological_engineer / agricultural_engineer / chemist → the_observer
--   all other roles (aerospace_engineer, etc.)           → the_artificer
-- This preserves departmental boundaries established in PR #3 and ensures
-- The Observer's dashboard is not polluted with aerospace requests and vice versa.
UPDATE help_requests hr
SET assigned_proxy_director = CASE
    WHEN r.name IN ('biological_engineer', 'agricultural_engineer', 'chemist') THEN 'the_observer'
    ELSE 'the_artificer'
END
FROM users u
JOIN roles r ON r.id = u.role_id
WHERE hr.requested_by = u.id;

-- Enforce NOT NULL now that all rows have been assigned a value.
ALTER TABLE help_requests
    ALTER COLUMN assigned_proxy_director SET NOT NULL;

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
