-- Migration 017: Chemistry Department — Matter Archive
-- Creates a dedicated matter_archive table for the Chemistry department,
-- completely decoupled from the Biology species_archive.
-- Chemists study the chemical properties of non-living objects
-- (minerals, alloys, compounds, composites, etc.).
-- New matter is added ONLY through the concluded new_matter experiment pipeline,
-- approved by The Observer.

CREATE TABLE IF NOT EXISTS matter_archive (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name                    VARCHAR(300) NOT NULL,
    classification          VARCHAR(200),
    matter_type             VARCHAR(200),          -- e.g. mineral, alloy, compound, composite
    description             TEXT,
    properties              TEXT,                  -- chemical/physical properties from the conclusion document
    discovery_experiment_id UUID REFERENCES experiments(id),
    discovered_by           UUID REFERENCES users(id),
    approved_by             UUID REFERENCES users(id),
    approved_at             TIMESTAMPTZ,
    created_at              TIMESTAMPTZ DEFAULT NOW(),
    deleted_at              TIMESTAMPTZ
);
