//! DTOs: database rows (FromRow), request payloads, and response shapes.
#![allow(dead_code)]

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Enums (mapped to Postgres ENUM types)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "card_state", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum CardState {
    New,
    Learning,
    Review,
    Relearning,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "job_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Pending,
    Running,
    Done,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "job_kind", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum JobKind {
    Blocks,
    Flashcards,
    Exam,
    Feynman,
    ConceptMap,
}

// ---------------------------------------------------------------------------
// Database rows
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Subject {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub exam_date: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Block {
    pub id: Uuid,
    pub subject_id: Uuid,
    pub code: Option<String>,
    pub title: String,
    pub summary: Option<String>,
    pub position: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct SourceDocument {
    pub id: Uuid,
    pub subject_id: Uuid,
    pub block_id: Option<Uuid>,
    pub title: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Flashcard {
    pub id: Uuid,
    pub subject_id: Uuid,
    pub block_id: Option<Uuid>,
    pub front: String,
    pub back: String,
    pub hint: Option<String>,
    pub source: String,
    pub stability: Option<f32>,
    pub difficulty: Option<f32>,
    pub state: CardState,
    pub due: DateTime<Utc>,
    pub last_reviewed: Option<DateTime<Utc>>,
    pub reps: i32,
    pub lapses: i32,
    pub created_at: DateTime<Utc>,
}

/// Explicit column list for SELECTing a full `Flashcard` (matches the struct).
pub const FLASHCARD_COLS: &str = "id, subject_id, block_id, front, back, hint, source, \
    stability, difficulty, state, due, last_reviewed, reps, lapses, created_at";

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct StudySession {
    pub id: Uuid,
    pub subject_id: Option<Uuid>,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub duration_s: Option<i32>,
    pub mode: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct GenerationJob {
    pub id: Uuid,
    pub subject_id: Uuid,
    pub source_id: Option<Uuid>,
    pub kind: JobKind,
    pub status: JobStatus,
    pub model: String,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
}

// ---------------------------------------------------------------------------
// Request payloads
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateSubject {
    pub name: String,
    pub description: Option<String>,
    pub exam_date: Option<NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSubject {
    pub name: Option<String>,
    pub description: Option<String>,
    pub exam_date: Option<NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct CreateBlock {
    pub code: Option<String>,
    pub title: String,
    pub summary: Option<String>,
    pub position: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSource {
    pub title: String,
    pub content: String,
    pub block_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct GenerateRequest {
    pub kind: JobKind,
    pub count: Option<i32>,
    pub block_id: Option<Uuid>,
    pub title: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateFlashcard {
    pub front: String,
    pub back: String,
    pub hint: Option<String>,
    pub block_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateFlashcard {
    pub front: Option<String>,
    pub back: Option<String>,
    pub hint: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ReviewRequest {
    pub rating: i16, // 1 Again .. 4 Easy
    pub session_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct StartSession {
    pub subject_id: Option<Uuid>,
    pub mode: Option<String>,
}

// ---------------------------------------------------------------------------
// Response shapes
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct ReviewResponse {
    pub id: Uuid,
    pub state: CardState,
    pub due: DateTime<Utc>,
    pub stability: Option<f32>,
    pub difficulty: Option<f32>,
    pub scheduled_days: i32,
    pub reps: i32,
    pub lapses: i32,
    pub leitner_box: u8,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct BlockMastery {
    pub block_id: Option<Uuid>,
    pub title: String,
    pub total: i64,
    pub due: i64,
    /// 0.0–1.0: share of cards in Leitner boxes 4–5 (well retained).
    pub mastery: f32,
}

#[derive(Debug, Serialize)]
pub struct SubjectStats {
    pub total_cards: i64,
    pub due_now: i64,
    /// counts per Leitner box, index 0 = box 1 .. index 4 = box 5
    pub by_box: [i64; 5],
    pub reviews_total: i64,
    pub weakest_blocks: Vec<BlockMastery>,
}

#[derive(Debug, Serialize)]
pub struct Guardrails {
    pub today_minutes: i64,
    pub streak_days: i64,
    pub rest_day_today: bool,
    pub after_22h: bool,
    pub sessions_today: i64,
    /// gentle nudge messages the UI can surface
    pub nudges: Vec<String>,
}

// ---------------------------------------------------------------------------
// Exams (Milestone 2)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "question_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum QuestionType {
    Mcq,
    TrueFalse,
    ShortAnswer,
    OpenEnded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "attempt_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AttemptStatus {
    InProgress,
    Submitted,
    Graded,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Exam {
    pub id: Uuid,
    pub subject_id: Uuid,
    pub title: String,
    pub time_limit_s: Option<i32>,
    pub source: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct ExamAttempt {
    pub id: Uuid,
    pub exam_id: Uuid,
    pub session_id: Option<Uuid>,
    pub status: AttemptStatus,
    pub started_at: DateTime<Utc>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub score: Option<f32>,
    pub max_score: Option<f32>,
}

/// Question as sent to the client while *taking* the exam — no answer key.
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct QuestionPublic {
    pub id: Uuid,
    pub block_id: Option<Uuid>,
    pub position: i32,
    pub qtype: QuestionType,
    pub prompt: String,
    pub options: Option<serde_json::Value>,
    pub points: i32,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ExamListItem {
    pub id: Uuid,
    pub title: String,
    pub time_limit_s: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub question_count: i64,
    pub attempt_count: i64,
    pub best_score: Option<f32>,
    pub max_score: Option<f32>,
}

#[derive(Debug, Serialize)]
pub struct ExamDetail {
    pub id: Uuid,
    pub subject_id: Uuid,
    pub title: String,
    pub time_limit_s: Option<i32>,
    pub questions: Vec<QuestionPublic>,
}

#[derive(Debug, Serialize)]
pub struct AttemptStart {
    pub attempt_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub time_limit_s: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct SubmitAnswer {
    pub question_id: Uuid,
    pub response: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SubmitAttempt {
    pub answers: Vec<SubmitAnswer>,
}

/// One graded question for the results screen (joined answer + question + block).
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct ResultItem {
    pub question_id: Uuid,
    pub block_id: Option<Uuid>,
    pub block_title: Option<String>,
    pub qtype: QuestionType,
    pub prompt: String,
    pub options: Option<serde_json::Value>,
    pub answer_key: Option<String>,
    pub explanation: Option<String>,
    pub points: i32,
    pub response: Option<String>,
    pub is_correct: Option<bool>,
    pub awarded: Option<f32>,
    pub ai_feedback: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BlockScore {
    pub block_id: Option<Uuid>,
    pub title: String,
    pub awarded: f32,
    pub max: f32,
}

#[derive(Debug, Serialize)]
pub struct AttemptResult {
    pub attempt_id: Uuid,
    pub status: AttemptStatus,
    pub score: Option<f32>,
    pub max_score: Option<f32>,
    pub items: Vec<ResultItem>,
    pub by_block: Vec<BlockScore>,
}

// ---------------------------------------------------------------------------
// Feynman + Cornell (Milestone 3)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct FeynmanConcept {
    pub id: Uuid,
    pub subject_id: Uuid,
    pub block_id: Option<Uuid>,
    pub title: String,
    pub hint: Option<String>,
    pub source: String,
    pub created_at: DateTime<Utc>,
}

/// List item enriched with practice history.
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct FeynmanConceptItem {
    pub id: Uuid,
    pub block_id: Option<Uuid>,
    pub title: String,
    pub hint: Option<String>,
    pub source: String,
    pub created_at: DateTime<Utc>,
    pub attempts: i64,
    pub last_rating: Option<i16>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct FeynmanAttempt {
    pub id: Uuid,
    pub concept_id: Uuid,
    pub self_rating: Option<i16>,
    pub hesitations: i32,
    pub duration_s: Option<i32>,
    pub explanation: Option<String>,
    pub ai_feedback: Option<String>,
    pub ai_score: Option<i16>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateFeynmanConcept {
    pub title: String,
    pub hint: Option<String>,
    pub block_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct CreateFeynmanAttempt {
    pub self_rating: Option<i16>,
    pub hesitations: Option<i32>,
    pub duration_s: Option<i32>,
    pub explanation: Option<String>,
    pub session_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct CornellNote {
    pub id: Uuid,
    pub subject_id: Uuid,
    pub block_id: Option<Uuid>,
    pub title: String,
    pub body: String,
    pub summary: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct CornellNoteItem {
    pub id: Uuid,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub cue_count: i64,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct CornellCue {
    pub id: Uuid,
    pub note_id: Uuid,
    pub question: String,
    pub answer: Option<String>,
    pub flashcard_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct CornellCueInput {
    pub question: String,
    pub answer: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCornellNote {
    pub title: String,
    pub body: String,
    pub summary: Option<String>,
    pub block_id: Option<Uuid>,
    #[serde(default)]
    pub cues: Vec<CornellCueInput>,
}

#[derive(Debug, Serialize)]
pub struct CornellNoteDetail {
    pub id: Uuid,
    pub subject_id: Uuid,
    pub block_id: Option<Uuid>,
    pub title: String,
    pub body: String,
    pub summary: Option<String>,
    pub created_at: DateTime<Utc>,
    pub cues: Vec<CornellCue>,
}

// ---------------------------------------------------------------------------
// Concept maps, schemas, FSRS insights (Milestone 4)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct ConceptMap {
    pub id: Uuid,
    pub subject_id: Uuid,
    pub block_id: Option<Uuid>,
    pub title: String,
    pub source: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ConceptMapListItem {
    pub id: Uuid,
    pub title: String,
    pub source: String,
    pub created_at: DateTime<Utc>,
    pub node_count: i64,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct ConceptMapNode {
    pub id: Uuid,
    pub label: String,
    pub parent_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct ConceptMapEdge {
    pub id: Uuid,
    pub from_node: Uuid,
    pub to_node: Uuid,
    pub label: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ConceptMapDetail {
    pub id: Uuid,
    pub subject_id: Uuid,
    pub title: String,
    pub nodes: Vec<ConceptMapNode>,
    pub edges: Vec<ConceptMapEdge>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct SchemaAsset {
    pub id: Uuid,
    pub subject_id: Uuid,
    pub block_id: Option<Uuid>,
    pub title: String,
    pub reference: Option<String>,
    pub drawing: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct SchemaListItem {
    pub id: Uuid,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub has_drawing: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateSchema {
    pub title: String,
    pub reference: Option<String>,
    pub block_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSchema {
    pub title: Option<String>,
    pub reference: Option<String>,
    pub drawing: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct FsrsInsights {
    pub reviews_total: i64,
    pub cards_reviewed: i64,
    /// share of mature-review ratings that were NOT "Again" (1) — measured retention
    pub measured_retention: Option<f32>,
    /// model-predicted average retrievability at review time
    pub predicted_retention: Option<f32>,
    pub rating_counts: [i64; 4], // again, hard, good, easy
    pub median_interval_days: Option<i32>,
    pub target_retention: f32,
    pub recommendation: String,
}
