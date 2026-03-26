-- Add experiment conclusion workflow fields
ALTER TABLE experiments
    ADD COLUMN IF NOT EXISTS conclusion_requested_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS conclusion_requested_by UUID REFERENCES users(id),
    ADD COLUMN IF NOT EXISTS conclusion_approved_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS conclusion_approved_by UUID REFERENCES users(id),
    ADD COLUMN IF NOT EXISTS final_notes TEXT,
    ADD COLUMN IF NOT EXISTS conclusion_approved BOOLEAN DEFAULT false;

-- Update status CHECK constraint to include 'conclusion_requested'
ALTER TABLE experiments DROP CONSTRAINT IF EXISTS experiments_status_check;
ALTER TABLE experiments
    ADD CONSTRAINT experiments_status_check
    CHECK (status IN ('pending', 'approved', 'rejected', 'in_progress', 'completed', 'cancelled', 'conclusion_requested'));
