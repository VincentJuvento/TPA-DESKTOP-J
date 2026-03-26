-- Migration 012: Blueprint proposals pipeline and mission counter
-- Adds blueprint_proposals table, extends vote_type constraint, adds mission_counter to users,
-- and extends ships status to include 'design' and 'building' phases.

-- 1. Extend vote_type on votes to allow 'blueprint'
ALTER TABLE votes
  DROP CONSTRAINT IF EXISTS votes_vote_type_check;

ALTER TABLE votes
  ADD CONSTRAINT votes_vote_type_check
    CHECK (vote_type IN ('general', 'budget', 'pressing_issue', 'blueprint'));

-- 2. Create blueprint_proposals table
CREATE TABLE IF NOT EXISTS blueprint_proposals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ship_name TEXT NOT NULL,
    blueprint_description TEXT NOT NULL,
    design_specs TEXT,
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'under_vote', 'approved', 'rejected')),
    vote_id UUID REFERENCES votes(id),
    ship_id UUID REFERENCES ships(id),
    submitted_by UUID REFERENCES users(id),
    reviewed_by UUID REFERENCES users(id),
    review_notes TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- 3. Add mission_counter to users table (tracks officially completed missions for astronauts)
ALTER TABLE users
  ADD COLUMN IF NOT EXISTS mission_counter INTEGER NOT NULL DEFAULT 0;

-- 4. Extend ships status to include 'design' and 'building' phases
ALTER TABLE ships
  DROP CONSTRAINT IF EXISTS ships_status_check;

ALTER TABLE ships
  ADD CONSTRAINT ships_status_check
    CHECK (status IN ('design', 'building', 'available', 'in_mission', 'maintenance', 'decommissioned'));
