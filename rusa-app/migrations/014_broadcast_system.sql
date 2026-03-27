-- 014_broadcast_system.sql
-- Add target filters and strictly route to guardian or anchorman

ALTER TABLE broadcast_requests
  ADD COLUMN IF NOT EXISTS target_filters JSONB,
  ADD COLUMN IF NOT EXISTS routed_to VARCHAR(50);
  
UPDATE broadcast_requests br
SET routed_to = CASE
  WHEN r.name IN ('head_of_earth_security', 'head_of_galactic_security') THEN 'the_guardian'
  ELSE 'the_anchorman'
END
FROM users u
JOIN roles r ON u.role_id = r.id
WHERE br.requested_by = u.id AND br.routed_to IS NULL;

ALTER TABLE messages
  ADD COLUMN IF NOT EXISTS is_broadcast BOOLEAN DEFAULT false,
  ADD COLUMN IF NOT EXISTS broadcast_sender VARCHAR(200);