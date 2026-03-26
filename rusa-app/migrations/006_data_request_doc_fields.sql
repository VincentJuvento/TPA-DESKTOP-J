-- SS-04: Data Services (Data Request / Data Response documentation-aligned fields)

-- Extend users with optional header fields used to auto-fill request forms
ALTER TABLE users ADD COLUMN IF NOT EXISTS tel_fax TEXT;
ALTER TABLE users ADD COLUMN IF NOT EXISTS department TEXT;
ALTER TABLE users ADD COLUMN IF NOT EXISTS department_email TEXT;

-- Backfill department/department_email defaults for existing users
UPDATE users u
SET
  department = COALESCE(u.department, r.display_name),
  department_email = COALESCE(u.department_email, u.email)
FROM roles r
WHERE u.role_id = r.id AND u.deleted_at IS NULL;

-- Data request: mandatory doc fields + response/signature plumbing
ALTER TABLE data_requests
  ADD COLUMN IF NOT EXISTS requester_location TEXT NOT NULL DEFAULT '',
  ADD COLUMN IF NOT EXISTS requester_tel_fax TEXT NOT NULL DEFAULT 'N/A',
  ADD COLUMN IF NOT EXISTS requester_department TEXT NOT NULL DEFAULT '',
  ADD COLUMN IF NOT EXISTS requester_department_email TEXT NOT NULL DEFAULT '',
  ADD COLUMN IF NOT EXISTS submitted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  ADD COLUMN IF NOT EXISTS requested_data_items TEXT NOT NULL DEFAULT '',
  ADD COLUMN IF NOT EXISTS reason_of_request TEXT NOT NULL DEFAULT '',
  ADD COLUMN IF NOT EXISTS requested_by_name TEXT NOT NULL DEFAULT '',
  ADD COLUMN IF NOT EXISTS requested_by_signature TEXT NOT NULL DEFAULT '',
  ADD COLUMN IF NOT EXISTS requested_by_signed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  ADD COLUMN IF NOT EXISTS response_status VARCHAR(20) NOT NULL DEFAULT 'provided'
    CHECK (response_status IN ('provided', 'rejected')),
  ADD COLUMN IF NOT EXISTS response_explanation TEXT,
  ADD COLUMN IF NOT EXISTS response_markdown TEXT,
  ADD COLUMN IF NOT EXISTS response_submitted_at TIMESTAMPTZ,
  ADD COLUMN IF NOT EXISTS provided_by JSONB NOT NULL DEFAULT '[]'::jsonb,
  ADD COLUMN IF NOT EXISTS provided_by_names JSONB NOT NULL DEFAULT '[]'::jsonb,
  ADD COLUMN IF NOT EXISTS final_reviewed_by UUID REFERENCES users(id),
  ADD COLUMN IF NOT EXISTS final_reviewed_at TIMESTAMPTZ,
  ADD COLUMN IF NOT EXISTS delivered_by UUID REFERENCES users(id),
  ADD COLUMN IF NOT EXISTS delivered_message_id UUID REFERENCES messages(id),
  ADD COLUMN IF NOT EXISTS requester_acknowledged_by UUID REFERENCES users(id),
  ADD COLUMN IF NOT EXISTS requester_acknowledged_at TIMESTAMPTZ;

-- Backfill request header/signature fields for existing requests
UPDATE data_requests dr
SET
  requester_location = COALESCE(dr.requester_location, COALESCE(u.location, '')),
  requester_department = COALESCE(dr.requester_department, r.display_name),
  requester_department_email = COALESCE(dr.requester_department_email, u.email),
  requested_by_name = COALESCE(dr.requested_by_name, u.full_name),
  requested_by_signature = COALESCE(dr.requested_by_signature, u.full_name),
  submitted_at = COALESCE(dr.submitted_at, dr.created_at, NOW()),
  requested_by_signed_at = COALESCE(dr.requested_by_signed_at, dr.created_at, NOW()),
  requested_data_items = COALESCE(NULLIF(dr.requested_data_items, ''), dr.description),
  reason_of_request = COALESCE(NULLIF(dr.reason_of_request, ''), dr.description),
  response_markdown = COALESCE(dr.response_markdown, dr.response_data)
FROM users u
JOIN roles r ON u.role_id = r.id
WHERE dr.requested_by = u.id AND dr.deleted_at IS NULL;

-- Attachments for the data response (stored in DB for offline availability)
CREATE TABLE IF NOT EXISTS data_response_attachments (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  request_id UUID NOT NULL REFERENCES data_requests(id),
  uploaded_by UUID NOT NULL REFERENCES users(id),
  filename TEXT NOT NULL,
  mime_type TEXT,
  bytes BYTEA NOT NULL,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  deleted_at TIMESTAMPTZ
);

