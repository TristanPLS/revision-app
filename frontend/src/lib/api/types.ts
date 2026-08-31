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

// ---- Concept maps + schemas + FSRS insights (Milestone 4) ----
export const conceptMapListItemSchema = z.object({
  id: z.string(),
  title: z.string(),
  source: z.string(),
  created_at: z.string(),
  node_count: z.number(),
});
export type ConceptMapListItem = z.infer<typeof conceptMapListItemSchema>;

export const conceptMapNodeSchema = z.object({
  id: z.string(),
  label: z.string(),
  parent_id: z.string().nullable(),
});
export type ConceptMapNode = z.infer<typeof conceptMapNodeSchema>;

export const conceptMapEdgeSchema = z.object({
  id: z.string(),
  from_node: z.string(),
  to_node: z.string(),
  label: z.string().nullable(),
});
export type ConceptMapEdge = z.infer<typeof conceptMapEdgeSchema>;

export const conceptMapDetailSchema = z.object({
  id: z.string(),
  subject_id: z.string(),
  title: z.string(),
  nodes: z.array(conceptMapNodeSchema),
  edges: z.array(conceptMapEdgeSchema),
});
export type ConceptMapDetail = z.infer<typeof conceptMapDetailSchema>;

export const schemaListItemSchema = z.object({
  id: z.string(),
  title: z.string(),
  created_at: z.string(),
  has_drawing: z.boolean(),
});
export type SchemaListItem = z.infer<typeof schemaListItemSchema>;

export const schemaAssetSchema = z.object({
  id: z.string(),
  subject_id: z.string(),
  block_id: z.string().nullable(),
  title: z.string(),
  reference: z.string().nullable(),
  drawing: z.unknown().nullable(),
  created_at: z.string(),
  updated_at: z.string(),
});
export type SchemaAsset = z.infer<typeof schemaAssetSchema>;

// ---- Study plan ("Tout générer d'un coup") ----
export const planBlockSchema = z.object({
  title: z.string(),
  code: z.string().nullable(),
  summary: z.string().nullable(),
});
export type PlanBlock = z.infer<typeof planBlockSchema>;

export const studyPlanSchema = z.object({
  blocks: z.array(planBlockSchema),
  flashcards: z.number(),
  exam_questions: z.number(),
  feynman_concepts: z.number(),
  map_nodes: z.number(),
  cornell_cues: z.number(),
  schemas: z.number(),
});
export type StudyPlan = z.infer<typeof studyPlanSchema>;

export const fsrsInsightsSchema = z.object({
  reviews_total: z.number(),
  cards_reviewed: z.number(),
  measured_retention: z.number().nullable(),
  predicted_retention: z.number().nullable(),
  rating_counts: z.array(z.number()).length(4),
  median_interval_days: z.number().nullable(),
  target_retention: z.number(),
  recommendation: z.string(),
});
export type FsrsInsights = z.infer<typeof fsrsInsightsSchema>;

// ---- Geography: flags & capitals (transversal, outside subjects) ----
export const geoKind = z.enum(["flag", "capital"]);
export type GeoKind = z.infer<typeof geoKind>;

// No capital here: a queue item carries iso2, so the referential must not hand
// out the expected answer before the card is played.
export const geoCountrySchema = z.object({
  iso2: z.string(),
  name_fr: z.string(),
  continent: z.string(),
});
export type GeoCountry = z.infer<typeof geoCountrySchema>;

// Neither branch carries the expected answer: for a flag card the right name
// hides among the 4 shuffled options, for a capital card only the country is
// sent. The answer comes back from `api.geo.answer`, once the card is consumed.
export const geoQueueItemSchema = z.discriminatedUnion("kind", [
  z.object({
    kind: z.literal("flag"),
    card_id: z.string(),
    iso2: z.string(),
    options: z.array(z.string()).length(4),
    continent: z.string(),
    state: cardState,
    due: z.string(),
    reps: z.number(),
  }),
  z.object({
    kind: z.literal("capital"),
    card_id: z.string(),
    iso2: z.string(),
    country_name: z.string(),
    continent: z.string(),
    state: cardState,
    due: z.string(),
    reps: z.number(),
  }),
]);
export type GeoQueueItem = z.infer<typeof geoQueueItemSchema>;
export type GeoFlagItem = Extract<GeoQueueItem, { kind: "flag" }>;
export type GeoCapitalItem = Extract<GeoQueueItem, { kind: "capital" }>;

export const geoAnswerResponseSchema = z.object({
  correct: z.boolean(),
  expected: z.string(),
  accepted_alternatives: z.array(z.string()),
  next_due: z.string(),
  scheduled_days: z.number(),
  leitner_box: z.number(),
  state: cardState,
});
export type GeoAnswerResponse = z.infer<typeof geoAnswerResponseSchema>;

export const geoStatsSchema = z.object({
  total_cards: z.number(),
  new_cards: z.number(),
  in_progress: z.number(),
  mastered: z.number(),
  due_now: z.number(),
  // null before any answer has been logged
  success_rate: z.number().nullable(),
});
export type GeoStats = z.infer<typeof geoStatsSchema>;
