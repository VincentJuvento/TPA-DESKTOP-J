-- 015_message_scheduling.sql
-- Enable scheduled message notifications

ALTER TABLE messages
  ADD COLUMN IF NOT EXISTS notified_at TIMESTAMPTZ;

UPDATE messages
SET notified_at = COALESCE(notified_at, sent_at)
WHERE deleted_at IS NULL
  AND recalled_at IS NULL
  AND is_draft = false
  AND sent_at IS NOT NULL
  AND sent_at <= NOW()
  AND notified_at IS NULL;
