-- Core content model: subjects -> blocks -> source documents, plus study sessions.
CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE subjects (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name        TEXT NOT NULL,
    description TEXT,
    exam_date   DATE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- "thématique" (e.g. Bloc A..E pour la biodiversité). Subject-agnostic.
CREATE TABLE blocks (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subject_id  UUID NOT NULL REFERENCES subjects(id) ON DELETE CASCADE,
    code        TEXT,
    title       TEXT NOT NULL,
    summary     TEXT,
    position    INT  NOT NULL DEFAULT 0,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_blocks_subject ON blocks(subject_id, position);

-- Raw pasted/uploaded course material (markdown/plain), input to AI generation.
CREATE TABLE source_documents (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subject_id  UUID NOT NULL REFERENCES subjects(id) ON DELETE CASCADE,
    block_id    UUID REFERENCES blocks(id) ON DELETE SET NULL,
    title       TEXT NOT NULL,
    content     TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_source_subject ON source_documents(subject_id);

-- Health guardrails: duration tracking, streak, rest-day awareness.
CREATE TABLE study_sessions (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subject_id  UUID REFERENCES subjects(id) ON DELETE SET NULL,
    started_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    ended_at    TIMESTAMPTZ,
    duration_s  INT,
    mode        TEXT,
    notes       TEXT
);
CREATE INDEX idx_sessions_started ON study_sessions(started_at DESC);
