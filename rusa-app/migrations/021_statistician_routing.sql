-- Migration 021: Add THE_STATISTICIAN routing for DATA help requests
-- Creates the data_analyst_tasks table and backfills existing DATA category
-- help_requests to route to the_statistician.

-- ─── 1. Create data_analyst_tasks table ─────────────────────────────────────
CREATE TABLE IF NOT EXISTS data_analyst_tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title TEXT NOT NULL,
    description TEXT,
    assigned_to UUID REFERENCES users(id),
    assigned_by UUID REFERENCES users(id),
    status TEXT NOT NULL DEFAULT 'pending',
    result_notes TEXT,
    progress_notes TEXT,
    due_date TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- ─── 2. Backfill: route DATA category help_requests to the_statistician ─────
-- Any existing help request whose category is 'DATA' (case-insensitive) should
-- have been routed to the_statistician.  Re-assign them now.
UPDATE help_requests
SET assigned_proxy_director = 'the_statistician'
WHERE UPPER(category) = 'DATA'
  AND deleted_at IS NULL;
