-- Async AI generation jobs (paste course -> Gemma -> flashcards/blocks/...).
CREATE TYPE job_status AS ENUM ('pending', 'running', 'done', 'failed');
CREATE TYPE job_kind   AS ENUM ('blocks', 'flashcards', 'exam', 'feynman', 'concept_map');

CREATE TABLE generation_jobs (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subject_id  UUID NOT NULL REFERENCES subjects(id) ON DELETE CASCADE,
    source_id   UUID REFERENCES source_documents(id) ON DELETE SET NULL,
    kind        job_kind   NOT NULL,
    status      job_status NOT NULL DEFAULT 'pending',
    model       TEXT NOT NULL,            -- snapshot of AI_MODEL used
    result      JSONB,                    -- {created, skipped, ...}
    error       TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    finished_at TIMESTAMPTZ
);
CREATE INDEX idx_jobs_subject ON generation_jobs(subject_id, created_at DESC);
