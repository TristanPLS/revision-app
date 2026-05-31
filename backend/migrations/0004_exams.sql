-- Mock exams (évaluation): timed, mixed question types, AI-gradable.
CREATE TYPE question_type  AS ENUM ('mcq', 'true_false', 'short_answer', 'open_ended');
CREATE TYPE attempt_status AS ENUM ('in_progress', 'submitted', 'graded');

CREATE TABLE exams (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subject_id   UUID NOT NULL REFERENCES subjects(id) ON DELETE CASCADE,
    title        TEXT NOT NULL,
    time_limit_s INT,                              -- NULL = untimed
    source       TEXT NOT NULL DEFAULT 'manual',    -- manual | ai
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_exams_subject ON exams(subject_id, created_at DESC);

CREATE TABLE questions (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    exam_id     UUID NOT NULL REFERENCES exams(id) ON DELETE CASCADE,
    block_id    UUID REFERENCES blocks(id) ON DELETE SET NULL,   -- enables per-block scoring
    position    INT NOT NULL DEFAULT 0,
    qtype       question_type NOT NULL,
    prompt      TEXT NOT NULL,
    options     JSONB,                              -- [{"key":"a","text":"…"}] for MCQ
    answer_key  TEXT,                               -- MCQ key / "true"|"false"; NULL when AI-graded
    explanation TEXT,                               -- shown after grading / rubric for AI grading
    points      INT NOT NULL DEFAULT 1,
    ai_graded   BOOLEAN NOT NULL DEFAULT false      -- true for short_answer/open_ended
);
CREATE INDEX idx_questions_exam ON questions(exam_id, position);

CREATE TABLE exam_attempts (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    exam_id      UUID NOT NULL REFERENCES exams(id) ON DELETE CASCADE,
    session_id   UUID REFERENCES study_sessions(id) ON DELETE SET NULL,
    status       attempt_status NOT NULL DEFAULT 'in_progress',
    started_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    submitted_at TIMESTAMPTZ,
    score        REAL,
    max_score    REAL
);
CREATE INDEX idx_attempts_exam ON exam_attempts(exam_id, started_at DESC);

CREATE TABLE exam_answers (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    attempt_id  UUID NOT NULL REFERENCES exam_attempts(id) ON DELETE CASCADE,
    question_id UUID NOT NULL REFERENCES questions(id) ON DELETE CASCADE,
    response    TEXT,
    is_correct  BOOLEAN,
    awarded     REAL,
    ai_feedback TEXT,
    UNIQUE (attempt_id, question_id)
);
