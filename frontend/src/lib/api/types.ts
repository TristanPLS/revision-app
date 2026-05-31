import { z } from "zod";

export const cardState = z.enum(["new", "learning", "review", "relearning"]);
export type CardState = z.infer<typeof cardState>;

export const subjectSchema = z.object({
  id: z.string(),
  name: z.string(),
  description: z.string().nullable(),
  exam_date: z.string().nullable(),
  created_at: z.string(),
  updated_at: z.string(),
});
export type Subject = z.infer<typeof subjectSchema>;

export const subjectListItemSchema = subjectSchema.extend({
  card_count: z.number(),
  due_count: z.number(),
});
export type SubjectListItem = z.infer<typeof subjectListItemSchema>;

export const blockSchema = z.object({
  id: z.string(),
  subject_id: z.string(),
  code: z.string().nullable(),
  title: z.string(),
  summary: z.string().nullable(),
  position: z.number(),
  created_at: z.string(),
});
export type Block = z.infer<typeof blockSchema>;

export const flashcardSchema = z.object({
  id: z.string(),
  subject_id: z.string(),
  block_id: z.string().nullable(),
  front: z.string(),
  back: z.string(),
  hint: z.string().nullable(),
  source: z.string(),
  stability: z.number().nullable(),
  difficulty: z.number().nullable(),
  state: cardState,
  due: z.string(),
  last_reviewed: z.string().nullable(),
  reps: z.number(),
  lapses: z.number(),
  created_at: z.string(),
});
export type Flashcard = z.infer<typeof flashcardSchema>;

export const reviewResponseSchema = z.object({
  id: z.string(),
  state: cardState,
  due: z.string(),
  stability: z.number().nullable(),
  difficulty: z.number().nullable(),
  scheduled_days: z.number(),
  reps: z.number(),
  lapses: z.number(),
  leitner_box: z.number(),
});
export type ReviewResponse = z.infer<typeof reviewResponseSchema>;

export const jobStatus = z.enum(["pending", "running", "done", "failed"]);
export const generationJobSchema = z.object({
  id: z.string(),
  subject_id: z.string(),
  source_id: z.string().nullable(),
  kind: z.string(),
  status: jobStatus,
  model: z.string(),
  result: z.unknown().nullable(),
  error: z.string().nullable(),
  created_at: z.string(),
  finished_at: z.string().nullable(),
});
export type GenerationJob = z.infer<typeof generationJobSchema>;

export const blockMasterySchema = z.object({
  block_id: z.string().nullable(),
  title: z.string(),
  total: z.number(),
  due: z.number(),
  mastery: z.number(),
});
export type BlockMastery = z.infer<typeof blockMasterySchema>;

export const subjectStatsSchema = z.object({
  total_cards: z.number(),
  due_now: z.number(),
  by_box: z.array(z.number()).length(5),
  reviews_total: z.number(),
  weakest_blocks: z.array(blockMasterySchema),
});
export type SubjectStats = z.infer<typeof subjectStatsSchema>;

export const guardrailsSchema = z.object({
  today_minutes: z.number(),
  streak_days: z.number(),
  rest_day_today: z.boolean(),
  after_22h: z.boolean(),
  sessions_today: z.number(),
  nudges: z.array(z.string()),
});
export type Guardrails = z.infer<typeof guardrailsSchema>;

// ---- Exams (Milestone 2) ----
export const questionType = z.enum(["mcq", "true_false", "short_answer", "open_ended"]);
export type QuestionType = z.infer<typeof questionType>;

export const attemptStatus = z.enum(["in_progress", "submitted", "graded"]);

const optionSchema = z.object({ key: z.string(), text: z.string() });

export const questionPublicSchema = z.object({
  id: z.string(),
  block_id: z.string().nullable(),
  position: z.number(),
  qtype: questionType,
  prompt: z.string(),
  options: z.array(optionSchema).nullable(),
  points: z.number(),
});
export type QuestionPublic = z.infer<typeof questionPublicSchema>;

export const examDetailSchema = z.object({
  id: z.string(),
  subject_id: z.string(),
  title: z.string(),
  time_limit_s: z.number().nullable(),
  questions: z.array(questionPublicSchema),
});
export type ExamDetail = z.infer<typeof examDetailSchema>;

export const examListItemSchema = z.object({
  id: z.string(),
  title: z.string(),
  time_limit_s: z.number().nullable(),
  created_at: z.string(),
  question_count: z.number(),
  attempt_count: z.number(),
  best_score: z.number().nullable(),
  max_score: z.number().nullable(),
});
export type ExamListItem = z.infer<typeof examListItemSchema>;

export const attemptStartSchema = z.object({
  attempt_id: z.string(),
  started_at: z.string(),
  time_limit_s: z.number().nullable(),
});
export type AttemptStart = z.infer<typeof attemptStartSchema>;

export const resultItemSchema = z.object({
  question_id: z.string(),
  block_id: z.string().nullable(),
  block_title: z.string().nullable(),
  qtype: questionType,
  prompt: z.string(),
  options: z.array(optionSchema).nullable(),
  answer_key: z.string().nullable(),
  explanation: z.string().nullable(),
  points: z.number(),
  response: z.string().nullable(),
  is_correct: z.boolean().nullable(),
  awarded: z.number().nullable(),
  ai_feedback: z.string().nullable(),
});
export type ResultItem = z.infer<typeof resultItemSchema>;

export const blockScoreSchema = z.object({
  block_id: z.string().nullable(),
  title: z.string(),
  awarded: z.number(),
  max: z.number(),
});

export const attemptResultSchema = z.object({
  attempt_id: z.string(),
  status: attemptStatus,
  score: z.number().nullable(),
  max_score: z.number().nullable(),
  items: z.array(resultItemSchema),
  by_block: z.array(blockScoreSchema),
});
export type AttemptResult = z.infer<typeof attemptResultSchema>;

export const studySessionSchema = z.object({
  id: z.string(),
  subject_id: z.string().nullable(),
  started_at: z.string(),
  ended_at: z.string().nullable(),
  duration_s: z.number().nullable(),
  mode: z.string().nullable(),
  notes: z.string().nullable(),
});
export type StudySession = z.infer<typeof studySessionSchema>;

// ---- Feynman (Milestone 3) ----
export const feynmanConceptItemSchema = z.object({
  id: z.string(),
  block_id: z.string().nullable(),
  title: z.string(),
  hint: z.string().nullable(),
  source: z.string(),
  created_at: z.string(),
  attempts: z.number(),
  last_rating: z.number().nullable(),
});
export type FeynmanConceptItem = z.infer<typeof feynmanConceptItemSchema>;

export const feynmanConceptSchema = z.object({
  id: z.string(),
  subject_id: z.string(),
  block_id: z.string().nullable(),
  title: z.string(),
  hint: z.string().nullable(),
  source: z.string(),
  created_at: z.string(),
});
export type FeynmanConcept = z.infer<typeof feynmanConceptSchema>;

export const feynmanAttemptSchema = z.object({
  id: z.string(),
  concept_id: z.string(),
  self_rating: z.number().nullable(),
  hesitations: z.number(),
  duration_s: z.number().nullable(),
  explanation: z.string().nullable(),
  ai_feedback: z.string().nullable(),
  ai_score: z.number().nullable(),
  created_at: z.string(),
});
export type FeynmanAttempt = z.infer<typeof feynmanAttemptSchema>;

// ---- Cornell (Milestone 3) ----
export const cornellNoteItemSchema = z.object({
  id: z.string(),
  title: z.string(),
  created_at: z.string(),
  cue_count: z.number(),
});
export type CornellNoteItem = z.infer<typeof cornellNoteItemSchema>;

export const cornellCueSchema = z.object({
  id: z.string(),
  note_id: z.string(),
  question: z.string(),
  answer: z.string().nullable(),
  flashcard_id: z.string().nullable(),
});
export type CornellCue = z.infer<typeof cornellCueSchema>;

export const cornellNoteDetailSchema = z.object({
  id: z.string(),
  subject_id: z.string(),
  block_id: z.string().nullable(),
  title: z.string(),
  body: z.string(),
  summary: z.string().nullable(),
  created_at: z.string(),
  cues: z.array(cornellCueSchema),
});
export type CornellNoteDetail = z.infer<typeof cornellNoteDetailSchema>;
