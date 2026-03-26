-- Add analyst_rejection_reason column to track why an analyst rejected a request
ALTER TABLE data_requests ADD COLUMN IF NOT EXISTS analyst_rejection_reason TEXT;

-- Update status constraint to include analyst_rejected workflow state
ALTER TABLE data_requests DROP CONSTRAINT IF EXISTS data_requests_status_check;
ALTER TABLE data_requests ADD CONSTRAINT data_requests_status_check
    CHECK (status IN ('pending', 'approved', 'rejected', 'processing', 'analyst_submitted', 'analyst_rejected', 'delivered'));
