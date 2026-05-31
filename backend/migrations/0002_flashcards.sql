-- Flashcards carry FSRS memory state; reviews is an append-only log.
CREATE TYPE card_state AS ENUM ('new', 'learning', 'review', 'relearning');

CREATE TABLE flashcards (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subject_id      UUID NOT NULL REFERENCES subjects(id) ON DELETE CASCADE,
    block_id        UUID REFERENCES blocks(id) ON DELETE SET NULL,
    front           TEXT NOT NULL,
    back            TEXT NOT NULL,
    hint            TEXT,                              -- optional mental-image hint
    source          TEXT NOT NULL DEFAULT 'manual',    -- manual | ai | cornell
    cornell_note_id UUID,                              -- FK added in a later milestone (Cornell)

    -- ---- FSRS memory state ----
    stability     REAL,                                -- NULL until first review
    difficulty    REAL,
    state         card_state NOT NULL DEFAULT 'new',
    due           TIMESTAMPTZ NOT NULL DEFAULT now(),  -- drives the review queue
    last_reviewed TIMESTAMPTZ,
    reps          INT NOT NULL DEFAULT 0,
    lapses        INT NOT NULL DEFAULT 0,

    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);
-- The hot index for the review queue:
CREATE INDEX idx_flashcards_due     ON flashcards(due);
CREATE INDEX idx_flashcards_subject ON flashcards(subject_id, due);
CREATE INDEX idx_flashcards_block   ON flashcards(block_id);

CREATE TABLE reviews (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    flashcard_id     UUID NOT NULL REFERENCES flashcards(id) ON DELETE CASCADE,
    session_id       UUID REFERENCES study_sessions(id) ON DELETE SET NULL,
    rating           SMALLINT NOT NULL CHECK (rating BETWEEN 1 AND 4),  -- 1 Again .. 4 Easy
    reviewed_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    elapsed_days     INT NOT NULL,
    stability_after  REAL,
    difficulty_after REAL,
    scheduled_days   INT NOT NULL
);
CREATE INDEX idx_reviews_card ON reviews(flashcard_id, reviewed_at);
