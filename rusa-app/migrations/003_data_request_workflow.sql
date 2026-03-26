-- Fix the status constraint to include all workflow states
ALTER TABLE data_requests DROP CONSTRAINT IF EXISTS data_requests_status_check;
ALTER TABLE data_requests ADD CONSTRAINT data_requests_status_check 
    CHECK (status IN ('pending', 'approved', 'rejected', 'processing', 'analyst_submitted', 'delivered'));

-- Add assigned_to column to track which analyst is handling it
ALTER TABLE data_requests ADD COLUMN IF NOT EXISTS assigned_to UUID REFERENCES users(id);

-- Add analyst_notes column
ALTER TABLE data_requests ADD COLUMN IF NOT EXISTS analyst_notes TEXT;
