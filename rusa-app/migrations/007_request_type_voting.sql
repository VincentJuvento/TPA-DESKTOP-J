-- Migration 007: Request-type-based voting system
-- Adds general_requests table and vote_type tracking

-- Add vote_type to votes table (general, budget, pressing_issue)
ALTER TABLE votes
  ADD COLUMN IF NOT EXISTS vote_type VARCHAR(50) DEFAULT 'pressing_issue'
    CHECK (vote_type IN ('general', 'budget', 'pressing_issue'));

-- Add requires_vote flag and ensure vote_id exists on budget_requests
ALTER TABLE budget_requests
  ADD COLUMN IF NOT EXISTS requires_vote BOOLEAN NOT NULL DEFAULT TRUE;

-- vote_id already exists on budget_requests from migration 001, confirm FK is in place
-- (no-op if already exists, safe to run)

-- General requests table (catch-all for requests needing director vote)
CREATE TABLE IF NOT EXISTS general_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(300) NOT NULL,
    description TEXT NOT NULL,
    request_type VARCHAR(50) NOT NULL DEFAULT 'general'
        CHECK (request_type IN ('general', 'pressing_issue')),
    status VARCHAR(50) NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'under_vote', 'approved', 'rejected')),
    requires_vote BOOLEAN NOT NULL DEFAULT TRUE,
    vote_id UUID REFERENCES votes(id),
    requested_by UUID NOT NULL REFERENCES users(id),
    reviewed_by UUID REFERENCES users(id),
    review_notes TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);
