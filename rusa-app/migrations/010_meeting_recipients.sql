-- Add To/CC/BCC recipient columns to meetings table
ALTER TABLE meetings
    ADD COLUMN IF NOT EXISTS to_recipients  JSONB NOT NULL DEFAULT '[]',
    ADD COLUMN IF NOT EXISTS cc_recipients  JSONB NOT NULL DEFAULT '[]',
    ADD COLUMN IF NOT EXISTS bcc_recipients JSONB NOT NULL DEFAULT '[]';
