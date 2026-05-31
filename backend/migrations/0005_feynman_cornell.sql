-- Feynman menu (self-explanation) and Cornell notes (Milestone 3).

CREATE TABLE feynman_concepts (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subject_id UUID NOT NULL REFERENCES subjects(id) ON DELETE CASCADE,
    block_id   UUID REFERENCES blocks(id) ON DELETE SET NULL,
    title      TEXT NOT NULL,           -- "Pourquoi la 6ème extinction est particulière ?"
    hint       TEXT,                    -- what a good explanation should cover
    source     TEXT NOT NULL DEFAULT 'manual',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_feynman_subject ON feynman_concepts(subject_id);

CREATE TABLE feynman_attempts (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    concept_id  UUID NOT NULL REFERENCES feynman_concepts(id) ON DELETE CASCADE,
    session_id  UUID REFERENCES study_sessions(id) ON DELETE SET NULL,
    self_rating SMALLINT CHECK (self_rating BETWEEN 1 AND 5),
    hesitations INT NOT NULL DEFAULT 0,   -- > 3 => revise (methodology)
    duration_s  INT,                      -- > 120 => revise
    explanation TEXT,
    ai_feedback TEXT,
    ai_score    SMALLINT,                 -- 0-100, optional
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_feynman_attempts ON feynman_attempts(concept_id, created_at DESC);

CREATE TABLE cornell_notes (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subject_id UUID NOT NULL REFERENCES subjects(id) ON DELETE CASCADE,
    block_id   UUID REFERENCES blocks(id) ON DELETE SET NULL,
    title      TEXT NOT NULL,
    body       TEXT NOT NULL,            -- main note column
    summary    TEXT,                     -- bottom Cornell summary
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_cornell_subject ON cornell_notes(subject_id, created_at DESC);

CREATE TABLE cornell_cues (              -- left-margin questions
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    note_id      UUID NOT NULL REFERENCES cornell_notes(id) ON DELETE CASCADE,
    question     TEXT NOT NULL,
    answer       TEXT,
    flashcard_id UUID REFERENCES flashcards(id) ON DELETE SET NULL  -- set when converted
);
CREATE INDEX idx_cornell_cues ON cornell_cues(note_id);

-- Wire the deferred back-reference from flashcards (declared nullable in 0002).
ALTER TABLE flashcards
    ADD CONSTRAINT fk_flashcard_cornell
    FOREIGN KEY (cornell_note_id) REFERENCES cornell_notes(id) ON DELETE SET NULL;
