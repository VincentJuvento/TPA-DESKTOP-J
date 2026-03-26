-- Migration 011: Director voting protocol enhancements
-- Adds mandatory reasoning to vote ballots and links abandonment requests to votes

-- Add reasoning column to vote_ballots (filled by backend at cast time)
ALTER TABLE vote_ballots
  ADD COLUMN IF NOT EXISTS reasoning TEXT;

-- Allow abandonment requests to track the linked director vote
ALTER TABLE station_abandonment_requests
  ADD COLUMN IF NOT EXISTS vote_id UUID REFERENCES votes(id);

-- Settlement anomaly reports can also escalate to director vote
ALTER TABLE anomaly_reports
  ADD COLUMN IF NOT EXISTS vote_id UUID REFERENCES votes(id);
