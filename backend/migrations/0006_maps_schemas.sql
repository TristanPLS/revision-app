-- Concept maps (cartographie) and hand-drawn schemas (dual coding) — Milestone 4.

CREATE TABLE concept_maps (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subject_id UUID NOT NULL REFERENCES subjects(id) ON DELETE CASCADE,
    block_id   UUID REFERENCES blocks(id) ON DELETE SET NULL,
    title      TEXT NOT NULL,
    source     TEXT NOT NULL DEFAULT 'manual',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_maps_subject ON concept_maps(subject_id, created_at DESC);

CREATE TABLE concept_map_nodes (
    id        UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    map_id    UUID NOT NULL REFERENCES concept_maps(id) ON DELETE CASCADE,
    label     TEXT NOT NULL,
    parent_id UUID REFERENCES concept_map_nodes(id) ON DELETE CASCADE  -- hierarchy
);
CREATE INDEX idx_map_nodes ON concept_map_nodes(map_id);

CREATE TABLE concept_map_edges (
    id        UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    map_id    UUID NOT NULL REFERENCES concept_maps(id) ON DELETE CASCADE,
    from_node UUID NOT NULL REFERENCES concept_map_nodes(id) ON DELETE CASCADE,
    to_node   UUID NOT NULL REFERENCES concept_map_nodes(id) ON DELETE CASCADE,
    label     TEXT                                  -- relationship ("cause", "entraîne"…)
);
CREATE INDEX idx_map_edges ON concept_map_edges(map_id);

-- Dual coding: "redo from memory then compare". The drawing is a tldraw snapshot.
CREATE TABLE schema_assets (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subject_id UUID NOT NULL REFERENCES subjects(id) ON DELETE CASCADE,
    block_id   UUID REFERENCES blocks(id) ON DELETE SET NULL,
    title      TEXT NOT NULL,
    reference  TEXT,                                 -- what the schema should contain (compare target)
    drawing    JSONB,                                -- tldraw snapshot of the from-memory attempt
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_schema_subject ON schema_assets(subject_id, created_at DESC);
