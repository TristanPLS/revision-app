// Generation handlers thread pool/ai/ids/source/count/block/title through many
// inner functions — a wide signature is clearer here than a params struct.
#![allow(clippy::too_many_arguments)]

use std::collections::HashMap;

use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

use super::{prompts, schemas, AiClient};
use crate::error::{AppError, AppResult};
use crate::models::StudyPlan;

/// A block created for a bundle, used to auto-classify generated items by
/// matching the model's `block_hint` against the block title.
#[derive(Clone)]
struct BlockRef {
    id: Uuid,
    title: String,
}

/// Resolve a model-provided `block_hint` to one of the created blocks: exact
/// (case-insensitive) title match first, then a loose contains match; falls
/// back to `default` (used as-is for single-scope generation, where `blocks`
/// is empty and `block_hint` is ignored).
fn resolve_block(hint: Option<&str>, blocks: &[BlockRef], default: Option<Uuid>) -> Option<Uuid> {
    if blocks.is_empty() {
        return default;
    }
    let Some(h) = hint.map(str::trim).filter(|h| !h.is_empty()) else {
        return default;
    };
    let hl = h.to_lowercase();
    blocks
        .iter()
        .find(|b| b.title.to_lowercase() == hl)
        .or_else(|| {
            blocks.iter().find(|b| {
                let bl = b.title.to_lowercase();
                bl.contains(&hl) || hl.contains(&bl)
            })
        })
        .map(|b| b.id)
        .or(default)
}

fn block_titles(blocks: &[BlockRef]) -> Vec<String> {
    blocks.iter().map(|b| b.title.clone()).collect()
}

#[derive(Deserialize)]
struct GenFlashcards {
    flashcards: Vec<GenCard>,
}

#[derive(Deserialize)]
struct GenCard {
    front: String,
    back: String,
    block_hint: Option<String>,
}

/// Background task: generate flashcards from a source document, insert them, and
/// move the `generation_jobs` row through running -> done/failed.
pub async fn run_flashcards(
    pool: PgPool,
    ai: AiClient,
    job_id: Uuid,
    subject_id: Uuid,
    source: String,
    count: i32,
    block_id: Option<Uuid>,
    block_title: Option<String>,
) {
    let _ = sqlx::query("UPDATE generation_jobs SET status = 'running' WHERE id = $1")
        .bind(job_id)
        .execute(&pool)
        .await;

    let outcome = generate_and_insert(
        &pool,
        &ai,
        subject_id,
        &source,
        count,
        block_id,
        block_title.as_deref(),
        &[],
    )
    .await;

    match outcome {
        Ok(result) => {
            tracing::info!(%job_id, %result, "flashcard generation done");
            let _ = sqlx::query(
                "UPDATE generation_jobs SET status='done', result=$2, finished_at=now() WHERE id=$1",
            )
            .bind(job_id)
            .bind(result)
            .execute(&pool)
            .await;
        }
        Err(e) => {
            tracing::error!(%job_id, error=%e, "flashcard generation failed");
            let _ = sqlx::query(
                "UPDATE generation_jobs SET status='failed', error=$2, finished_at=now() WHERE id=$1",
            )
            .bind(job_id)
            .bind(e.to_string())
            .execute(&pool)
            .await;
        }
    }
}

async fn generate_and_insert(
    pool: &PgPool,
    ai: &AiClient,
    subject_id: Uuid,
    source: &str,
    count: i32,
    block_id: Option<Uuid>,
    block_title: Option<&str>,
    blocks: &[BlockRef],
) -> AppResult<Value> {
    let prompt = prompts::flashcards_prompt(source, count, block_title, &block_titles(blocks));
    let raw = ai.generate_json(&prompt, schemas::flashcards_schema()).await?;
    let parsed: GenFlashcards =
        serde_json::from_str(&raw).map_err(|e| AppError::AiSchema(e.to_string()))?;

    let mut created = 0i64;
    let mut skipped = 0i64;
    let mut tx = pool.begin().await?;
    for c in parsed.flashcards {
        let front = c.front.trim();
        let back = c.back.trim();
        if front.is_empty() || back.is_empty() {
            skipped += 1;
            continue;
        }
        let bid = resolve_block(c.block_hint.as_deref(), blocks, block_id);
        sqlx::query(
            "INSERT INTO flashcards (subject_id, block_id, front, back, source) \
             VALUES ($1, $2, $3, $4, 'ai')",
        )
        .bind(subject_id)
        .bind(bid)
        .bind(front)
        .bind(back)
        .execute(&mut *tx)
        .await?;
        created += 1;
    }
    tx.commit().await?;

    Ok(json!({ "created": created, "skipped": skipped }))
}

// ---------------------------------------------------------------------------
// Concept map generation (Milestone 4)
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct GenMap {
    title: Option<String>,
    nodes: Vec<GenNode>,
    #[serde(default)]
    edges: Vec<GenEdge>,
}

#[derive(Deserialize)]
struct GenNode {
    id: String,
    label: String,
    parent: Option<String>,
}

#[derive(Deserialize)]
struct GenEdge {
    from: String,
    to: String,
    label: Option<String>,
}

pub async fn run_concept_map(
    pool: PgPool,
    ai: AiClient,
    job_id: Uuid,
    subject_id: Uuid,
    source: String,
    block_id: Option<Uuid>,
    block_title: Option<String>,
    map_title: String,
) {
    let _ = sqlx::query("UPDATE generation_jobs SET status = 'running' WHERE id = $1")
        .bind(job_id)
        .execute(&pool)
        .await;

    let outcome = generate_map_inner(
        &pool,
        &ai,
        subject_id,
        &source,
        block_id,
        block_title.as_deref(),
        &map_title,
        None,
    )
    .await;

    match outcome {
        Ok(result) => {
            tracing::info!(%job_id, %result, "concept map generation done");
            let _ = sqlx::query(
                "UPDATE generation_jobs SET status='done', result=$2, finished_at=now() WHERE id=$1",
            )
            .bind(job_id)
            .bind(result)
            .execute(&pool)
            .await;
        }
        Err(e) => {
            tracing::error!(%job_id, error=%e, "concept map generation failed");
            let _ = sqlx::query(
                "UPDATE generation_jobs SET status='failed', error=$2, finished_at=now() WHERE id=$1",
            )
            .bind(job_id)
            .bind(e.to_string())
            .execute(&pool)
            .await;
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn generate_map_inner(
    pool: &PgPool,
    ai: &AiClient,
    subject_id: Uuid,
    source: &str,
    block_id: Option<Uuid>,
    block_title: Option<&str>,
    map_title: &str,
    target_nodes: Option<i32>,
) -> AppResult<Value> {
    let prompt = prompts::concept_map_prompt(source, block_title, target_nodes);
    let raw = ai.generate_json(&prompt, schemas::concept_map_schema()).await?;
    let parsed: GenMap = serde_json::from_str(&raw).map_err(|e| AppError::AiSchema(e.to_string()))?;

    if parsed.nodes.is_empty() {
        return Err(AppError::AiSchema("aucun nœud généré".into()));
    }
    let title = parsed
        .title
        .as_deref()
        .filter(|t| !t.trim().is_empty())
        .unwrap_or(map_title);

    let mut tx = pool.begin().await?;
    let map_id: Uuid = sqlx::query_scalar(
        "INSERT INTO concept_maps (subject_id, block_id, title, source) \
         VALUES ($1, $2, $3, 'ai') RETURNING id",
    )
    .bind(subject_id)
    .bind(block_id)
    .bind(title)
    .fetch_one(&mut *tx)
    .await?;

    // Pass 1: insert nodes, remember ai-id -> uuid.
    let mut idmap: HashMap<String, Uuid> = HashMap::new();
    for n in &parsed.nodes {
        if n.label.trim().is_empty() {
            continue;
        }
        let nid: Uuid =
            sqlx::query_scalar("INSERT INTO concept_map_nodes (map_id, label) VALUES ($1, $2) RETURNING id")
                .bind(map_id)
                .bind(n.label.trim())
                .fetch_one(&mut *tx)
                .await?;
        idmap.insert(n.id.clone(), nid);
    }
    // Pass 2: wire parents.
    for n in &parsed.nodes {
        if let Some(p) = n.parent.as_deref().filter(|p| !p.trim().is_empty()) {
            if let (Some(&child), Some(&parent)) = (idmap.get(&n.id), idmap.get(p)) {
                if child != parent {
                    sqlx::query("UPDATE concept_map_nodes SET parent_id = $1 WHERE id = $2")
                        .bind(parent)
                        .bind(child)
                        .execute(&mut *tx)
                        .await?;
                }
            }
        }
    }
    // Edges.
    let mut edges = 0i64;
    for e in &parsed.edges {
        if let (Some(&a), Some(&b)) = (idmap.get(&e.from), idmap.get(&e.to)) {
            if a != b {
                sqlx::query(
                    "INSERT INTO concept_map_edges (map_id, from_node, to_node, label) VALUES ($1, $2, $3, $4)",
                )
                .bind(map_id)
                .bind(a)
                .bind(b)
                .bind(e.label.as_deref())
                .execute(&mut *tx)
                .await?;
                edges += 1;
            }
        }
    }
    tx.commit().await?;

    Ok(json!({ "map_id": map_id, "nodes": idmap.len(), "edges": edges }))
}

// ---------------------------------------------------------------------------
// Exam generation (Milestone 2)
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct GenExam {
    questions: Vec<GenQuestion>,
}

#[derive(Deserialize)]
struct GenQuestion {
    qtype: String,
    prompt: String,
    options: Option<Vec<GenOption>>,
    answer_key: Option<String>,
    explanation: Option<String>,
    points: Option<i32>,
    block_hint: Option<String>,
}

#[derive(Deserialize)]
struct GenOption {
    key: String,
    text: String,
}

fn parse_qtype(s: &str) -> Option<crate::models::QuestionType> {
    use crate::models::QuestionType::*;
    match s.trim().to_lowercase().as_str() {
        "mcq" | "qcm" => Some(Mcq),
        "true_false" | "truefalse" | "vrai_faux" => Some(TrueFalse),
        "short_answer" | "short" | "court" => Some(ShortAnswer),
        "open_ended" | "open" | "ouvert" => Some(OpenEnded),
        _ => None,
    }
}

/// Background task: generate an exam (+ questions) from a source document.
#[allow(clippy::too_many_arguments)]
pub async fn run_exam(
    pool: PgPool,
    ai: AiClient,
    job_id: Uuid,
    subject_id: Uuid,
    source: String,
    count: i32,
    block_id: Option<Uuid>,
    block_title: Option<String>,
    exam_title: String,
) {
    let _ = sqlx::query("UPDATE generation_jobs SET status = 'running' WHERE id = $1")
        .bind(job_id)
        .execute(&pool)
        .await;

    let outcome = generate_exam_inner(
        &pool,
        &ai,
        subject_id,
        &source,
        count,
        block_id,
        block_title.as_deref(),
        &exam_title,
        &[],
    )
    .await;

    match outcome {
        Ok(result) => {
            tracing::info!(%job_id, %result, "exam generation done");
            let _ = sqlx::query(
                "UPDATE generation_jobs SET status='done', result=$2, finished_at=now() WHERE id=$1",
            )
            .bind(job_id)
            .bind(result)
            .execute(&pool)
            .await;
        }
        Err(e) => {
            tracing::error!(%job_id, error=%e, "exam generation failed");
            let _ = sqlx::query(
                "UPDATE generation_jobs SET status='failed', error=$2, finished_at=now() WHERE id=$1",
            )
            .bind(job_id)
            .bind(e.to_string())
            .execute(&pool)
            .await;
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn generate_exam_inner(
    pool: &PgPool,
    ai: &AiClient,
    subject_id: Uuid,
    source: &str,
    count: i32,
    block_id: Option<Uuid>,
    block_title: Option<&str>,
    exam_title: &str,
    blocks: &[BlockRef],
) -> AppResult<Value> {
    let prompt = prompts::exam_prompt(source, count, block_title, &block_titles(blocks));
    let raw = ai.generate_json(&prompt, schemas::exam_schema()).await?;
    let parsed: GenExam =
        serde_json::from_str(&raw).map_err(|e| AppError::AiSchema(e.to_string()))?;

    let time_limit_s = (count * 180).clamp(600, 5400);

    let mut tx = pool.begin().await?;
    let exam_id: Uuid = sqlx::query_scalar(
        "INSERT INTO exams (subject_id, title, time_limit_s, source) \
         VALUES ($1, $2, $3, 'ai') RETURNING id",
    )
    .bind(subject_id)
    .bind(exam_title)
    .bind(time_limit_s)
    .fetch_one(&mut *tx)
    .await?;

    let mut created = 0i64;
    let mut skipped = 0i64;
    let mut position = 0i32;
    for q in parsed.questions {
        let prompt_text = q.prompt.trim();
        let qtype = match parse_qtype(&q.qtype) {
            Some(t) if !prompt_text.is_empty() => t,
            _ => {
                skipped += 1;
                continue;
            }
        };
        let ai_graded = matches!(
            qtype,
            crate::models::QuestionType::ShortAnswer | crate::models::QuestionType::OpenEnded
        );
        let options_json: Option<Value> = q.options.map(|opts| {
            Value::Array(
                opts.into_iter()
                    .map(|o| json!({ "key": o.key, "text": o.text }))
                    .collect(),
            )
        });
        let points = q.points.unwrap_or(1).clamp(1, 10);
        let answer_key = if ai_graded { None } else { q.answer_key };
        let bid = resolve_block(q.block_hint.as_deref(), blocks, block_id);

        sqlx::query(
            "INSERT INTO questions \
             (exam_id, block_id, position, qtype, prompt, options, answer_key, explanation, points, ai_graded) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        )
        .bind(exam_id)
        .bind(bid)
        .bind(position)
        .bind(qtype)
        .bind(prompt_text)
        .bind(options_json)
        .bind(answer_key)
        .bind(q.explanation)
        .bind(points)
        .bind(ai_graded)
        .execute(&mut *tx)
        .await?;
        created += 1;
        position += 1;
    }

    if created == 0 {
        // nothing usable — roll back the empty exam
        tx.rollback().await?;
        return Err(AppError::AiSchema("aucune question exploitable".into()));
    }
    tx.commit().await?;

    Ok(json!({ "exam_id": exam_id, "created": created, "skipped": skipped }))
}

#[derive(Deserialize)]
struct GradeOut {
    score: f32,
    feedback: String,
}

/// Grade one free-text answer with the AI. Never fails the request: returns
/// (0, message) if the AI is unavailable or unparmseable.
pub async fn grade_answer(
    ai: &AiClient,
    question: &str,
    rubric: Option<&str>,
    response: &str,
    max_points: i32,
) -> (f32, String) {
    let prompt = prompts::grade_prompt(question, rubric, response, max_points);
    match ai.generate_json(&prompt, schemas::grade_schema()).await {
        Ok(raw) => match serde_json::from_str::<GradeOut>(&raw) {
            Ok(g) => (g.score.clamp(0.0, max_points as f32), g.feedback),
            Err(_) => (0.0, "Correction IA illisible.".to_string()),
        },
        Err(e) => {
            tracing::warn!(error=%e, "AI grading failed");
            (0.0, "Correction IA indisponible.".to_string())
        }
    }
}

// ---------------------------------------------------------------------------
// Feynman menu generation (Milestone 3)
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct GenFeynman {
    concepts: Vec<GenConcept>,
}

#[derive(Deserialize)]
struct GenConcept {
    title: String,
    hint: Option<String>,
    block_hint: Option<String>,
}

/// Background task: generate a Feynman concept menu from a source document.
pub async fn run_feynman(
    pool: PgPool,
    ai: AiClient,
    job_id: Uuid,
    subject_id: Uuid,
    source: String,
    count: i32,
    block_id: Option<Uuid>,
    block_title: Option<String>,
) {
    let _ = sqlx::query("UPDATE generation_jobs SET status = 'running' WHERE id = $1")
        .bind(job_id)
        .execute(&pool)
        .await;

    let outcome = generate_feynman_inner(
        &pool,
        &ai,
        subject_id,
        &source,
        count,
        block_id,
        block_title.as_deref(),
        &[],
    )
    .await;

    match outcome {
        Ok(result) => {
            tracing::info!(%job_id, %result, "feynman generation done");
            let _ = sqlx::query(
                "UPDATE generation_jobs SET status='done', result=$2, finished_at=now() WHERE id=$1",
            )
            .bind(job_id)
            .bind(result)
            .execute(&pool)
            .await;
        }
        Err(e) => {
            tracing::error!(%job_id, error=%e, "feynman generation failed");
            let _ = sqlx::query(
                "UPDATE generation_jobs SET status='failed', error=$2, finished_at=now() WHERE id=$1",
            )
            .bind(job_id)
            .bind(e.to_string())
            .execute(&pool)
            .await;
        }
    }
}

async fn generate_feynman_inner(
    pool: &PgPool,
    ai: &AiClient,
    subject_id: Uuid,
    source: &str,
    count: i32,
    block_id: Option<Uuid>,
    block_title: Option<&str>,
    blocks: &[BlockRef],
) -> AppResult<Value> {
    let prompt = prompts::feynman_prompt(source, count, block_title, &block_titles(blocks));
    let raw = ai.generate_json(&prompt, schemas::feynman_schema()).await?;
    let parsed: GenFeynman =
        serde_json::from_str(&raw).map_err(|e| AppError::AiSchema(e.to_string()))?;

    let mut created = 0i64;
    let mut skipped = 0i64;
    let mut tx = pool.begin().await?;
    for c in parsed.concepts {
        let title = c.title.trim();
        if title.is_empty() {
            skipped += 1;
            continue;
        }
        let bid = resolve_block(c.block_hint.as_deref(), blocks, block_id);
        sqlx::query(
            "INSERT INTO feynman_concepts (subject_id, block_id, title, hint, source) \
             VALUES ($1, $2, $3, $4, 'ai')",
        )
        .bind(subject_id)
        .bind(bid)
        .bind(title)
        .bind(c.hint)
        .execute(&mut *tx)
        .await?;
        created += 1;
    }
    tx.commit().await?;

    Ok(json!({ "created": created, "skipped": skipped }))
}

// ---------------------------------------------------------------------------
// Cornell note generation
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct GenCornell {
    title: Option<String>,
    body: String,
    summary: Option<String>,
    #[serde(default)]
    cues: Vec<GenCue>,
}

#[derive(Deserialize)]
struct GenCue {
    question: String,
    answer: Option<String>,
}

/// Background task: generate a Cornell note (body + summary + margin recall
/// cues) from a source document.
pub async fn run_cornell(
    pool: PgPool,
    ai: AiClient,
    job_id: Uuid,
    subject_id: Uuid,
    source: String,
    count: i32,
    block_id: Option<Uuid>,
    block_title: Option<String>,
    note_title: String,
) {
    let _ = sqlx::query("UPDATE generation_jobs SET status = 'running' WHERE id = $1")
        .bind(job_id)
        .execute(&pool)
        .await;

    let outcome = generate_cornell_inner(
        &pool,
        &ai,
        subject_id,
        &source,
        count,
        block_id,
        block_title.as_deref(),
        &note_title,
    )
    .await;

    match outcome {
        Ok(result) => {
            tracing::info!(%job_id, %result, "cornell generation done");
            let _ = sqlx::query(
                "UPDATE generation_jobs SET status='done', result=$2, finished_at=now() WHERE id=$1",
            )
            .bind(job_id)
            .bind(result)
            .execute(&pool)
            .await;
        }
        Err(e) => {
            tracing::error!(%job_id, error=%e, "cornell generation failed");
            let _ = sqlx::query(
                "UPDATE generation_jobs SET status='failed', error=$2, finished_at=now() WHERE id=$1",
            )
            .bind(job_id)
            .bind(e.to_string())
            .execute(&pool)
            .await;
        }
    }
}

async fn generate_cornell_inner(
    pool: &PgPool,
    ai: &AiClient,
    subject_id: Uuid,
    source: &str,
    count: i32,
    block_id: Option<Uuid>,
    block_title: Option<&str>,
    note_title: &str,
) -> AppResult<Value> {
    let prompt = prompts::cornell_prompt(source, count, block_title);
    let raw = ai.generate_json(&prompt, schemas::cornell_schema()).await?;
    let parsed: GenCornell =
        serde_json::from_str(&raw).map_err(|e| AppError::AiSchema(e.to_string()))?;

    let body = parsed.body.trim();
    if body.is_empty() {
        return Err(AppError::AiSchema("note Cornell vide".into()));
    }
    let title = parsed
        .title
        .as_deref()
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .unwrap_or(note_title);

    let mut tx = pool.begin().await?;
    let note_id: Uuid = sqlx::query_scalar(
        "INSERT INTO cornell_notes (subject_id, block_id, title, body, summary) \
         VALUES ($1, $2, $3, $4, $5) RETURNING id",
    )
    .bind(subject_id)
    .bind(block_id)
    .bind(title)
    .bind(body)
    .bind(parsed.summary.as_deref().map(str::trim).filter(|s| !s.is_empty()))
    .fetch_one(&mut *tx)
    .await?;

    let mut cues = 0i64;
    for c in &parsed.cues {
        let q = c.question.trim();
        if q.is_empty() {
            continue;
        }
        sqlx::query("INSERT INTO cornell_cues (note_id, question, answer) VALUES ($1, $2, $3)")
            .bind(note_id)
            .bind(q)
            .bind(c.answer.as_deref().map(str::trim).filter(|a| !a.is_empty()))
            .execute(&mut *tx)
            .await?;
        cues += 1;
    }
    tx.commit().await?;

    Ok(json!({ "note_id": note_id, "cues": cues }))
}

// ---------------------------------------------------------------------------
// Schema-stub generation (dual coding)
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct GenSchemas {
    schemas: Vec<GenSchemaItem>,
}

#[derive(Deserialize)]
struct GenSchemaItem {
    title: String,
    reference: Option<String>,
    block_hint: Option<String>,
}

/// Background task: generate schema stubs (title + reference of what to draw)
/// from a source document. The drawing is left to the learner (active encoding).
pub async fn run_schemas(
    pool: PgPool,
    ai: AiClient,
    job_id: Uuid,
    subject_id: Uuid,
    source: String,
    count: i32,
    block_id: Option<Uuid>,
    block_title: Option<String>,
) {
    let _ = sqlx::query("UPDATE generation_jobs SET status = 'running' WHERE id = $1")
        .bind(job_id)
        .execute(&pool)
        .await;

    let outcome = generate_schemas_inner(
        &pool,
        &ai,
        subject_id,
        &source,
        count,
        block_id,
        block_title.as_deref(),
        &[],
    )
    .await;

    match outcome {
        Ok(result) => {
            tracing::info!(%job_id, %result, "schema generation done");
            let _ = sqlx::query(
                "UPDATE generation_jobs SET status='done', result=$2, finished_at=now() WHERE id=$1",
            )
            .bind(job_id)
            .bind(result)
            .execute(&pool)
            .await;
        }
        Err(e) => {
            tracing::error!(%job_id, error=%e, "schema generation failed");
            let _ = sqlx::query(
                "UPDATE generation_jobs SET status='failed', error=$2, finished_at=now() WHERE id=$1",
            )
            .bind(job_id)
            .bind(e.to_string())
            .execute(&pool)
            .await;
        }
    }
}

async fn generate_schemas_inner(
    pool: &PgPool,
    ai: &AiClient,
    subject_id: Uuid,
    source: &str,
    count: i32,
    block_id: Option<Uuid>,
    block_title: Option<&str>,
    blocks: &[BlockRef],
) -> AppResult<Value> {
    let prompt = prompts::schemas_prompt(source, count, block_title, &block_titles(blocks));
    let raw = ai.generate_json(&prompt, schemas::schemas_schema()).await?;
    let parsed: GenSchemas =
        serde_json::from_str(&raw).map_err(|e| AppError::AiSchema(e.to_string()))?;

    let mut created = 0i64;
    let mut skipped = 0i64;
    let mut tx = pool.begin().await?;
    for sc in parsed.schemas {
        let title = sc.title.trim();
        if title.is_empty() {
            skipped += 1;
            continue;
        }
        let bid = resolve_block(sc.block_hint.as_deref(), blocks, block_id);
        sqlx::query(
            "INSERT INTO schema_assets (subject_id, block_id, title, reference) \
             VALUES ($1, $2, $3, $4)",
        )
        .bind(subject_id)
        .bind(bid)
        .bind(title)
        .bind(sc.reference.as_deref().map(str::trim).filter(|r| !r.is_empty()))
        .execute(&mut *tx)
        .await?;
        created += 1;
    }
    tx.commit().await?;

    Ok(json!({ "created": created, "skipped": skipped }))
}

// ---------------------------------------------------------------------------
// "Tout générer d'un coup" : planning pass + bundle orchestration
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct GenPlanBlock {
    title: String,
    code: Option<String>,
    summary: Option<String>,
}

#[derive(Deserialize)]
struct GenPlan {
    #[serde(default)]
    blocks: Vec<GenPlanBlock>,
    flashcards: i32,
    exam_questions: i32,
    feynman_concepts: i32,
    map_nodes: i32,
    #[serde(default)]
    cornell_cues: i32,
    #[serde(default)]
    schemas: i32,
}

/// Planning pass: one cheap AI call that reads the whole course and proposes a
/// block breakdown + a quantity for each support. Synchronous (the user waits
/// on the preview, then edits it). Counts are clamped to sane bounds; at most
/// 12 blocks are kept.
pub async fn plan(ai: &AiClient, source: &str) -> AppResult<StudyPlan> {
    let prompt = prompts::plan_prompt(source);
    let raw = ai.generate_json(&prompt, schemas::plan_schema()).await?;
    let parsed: GenPlan =
        serde_json::from_str(&raw).map_err(|e| AppError::AiSchema(e.to_string()))?;

    let blocks = parsed
        .blocks
        .into_iter()
        .filter_map(|b| {
            let title = b.title.trim().to_string();
            if title.is_empty() {
                return None;
            }
            Some(crate::models::PlanBlock {
                title,
                code: b.code.map(|c| c.trim().to_string()).filter(|c| !c.is_empty()),
                summary: b.summary.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
            })
        })
        .take(12)
        .collect();

    Ok(StudyPlan {
        blocks,
        flashcards: parsed.flashcards.clamp(1, 50),
        exam_questions: parsed.exam_questions.clamp(1, 50),
        feynman_concepts: parsed.feynman_concepts.clamp(1, 30),
        map_nodes: parsed.map_nodes.clamp(6, 20),
        cornell_cues: parsed.cornell_cues.clamp(0, 20),
        schemas: parsed.schemas.clamp(0, 8),
    })
}

/// Background task: generate ALL supports at once from a course, following a
/// (possibly user-edited) study plan. Creates the blocks first, then generates
/// flashcards / exam / Feynman / concept map, auto-classifying each item into
/// the new blocks via `block_hint`. Tolerant of partial failure: each support
/// records its own outcome (counts or error) and the job only fails outright if
/// nothing at all could be produced.
pub async fn run_bundle(
    pool: PgPool,
    ai: AiClient,
    job_id: Uuid,
    subject_id: Uuid,
    source: String,
    plan: StudyPlan,
    exam_title: String,
    map_title: String,
) {
    let _ = sqlx::query("UPDATE generation_jobs SET status = 'running' WHERE id = $1")
        .bind(job_id)
        .execute(&pool)
        .await;

    let outcome =
        run_bundle_inner(&pool, &ai, subject_id, &source, plan, &exam_title, &map_title).await;

    match outcome {
        Ok(result) => {
            tracing::info!(%job_id, %result, "bundle generation done");
            let _ = sqlx::query(
                "UPDATE generation_jobs SET status='done', result=$2, finished_at=now() WHERE id=$1",
            )
            .bind(job_id)
            .bind(result)
            .execute(&pool)
            .await;
        }
        Err(e) => {
            tracing::error!(%job_id, error=%e, "bundle generation failed");
            let _ = sqlx::query(
                "UPDATE generation_jobs SET status='failed', error=$2, finished_at=now() WHERE id=$1",
            )
            .bind(job_id)
            .bind(e.to_string())
            .execute(&pool)
            .await;
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn run_bundle_inner(
    pool: &PgPool,
    ai: &AiClient,
    subject_id: Uuid,
    source: &str,
    plan: StudyPlan,
    exam_title: &str,
    map_title: &str,
) -> AppResult<Value> {
    // 1) Create the blocks proposed by the plan, keeping their ids so generated
    //    items can be auto-classified into them.
    let mut blocks: Vec<BlockRef> = Vec::new();
    for (i, b) in plan.blocks.iter().enumerate() {
        let title = b.title.trim();
        if title.is_empty() {
            continue;
        }
        let id: Uuid = sqlx::query_scalar(
            "INSERT INTO blocks (subject_id, code, title, summary, position) \
             VALUES ($1, $2, $3, $4, $5) RETURNING id",
        )
        .bind(subject_id)
        .bind(b.code.as_deref().map(str::trim).filter(|c| !c.is_empty()))
        .bind(title)
        .bind(b.summary.as_deref().map(str::trim).filter(|s| !s.is_empty()))
        .bind(i as i32)
        .fetch_one(pool)
        .await?;
        blocks.push(BlockRef {
            id,
            title: title.to_string(),
        });
    }

    // 2) Generate each support in turn. Sequential, to stay friendly to the AI
    //    provider's rate limits; a failed support is recorded but doesn't abort
    //    the rest.
    let mut result = serde_json::Map::new();
    result.insert("blocks".into(), json!(blocks.len()));
    let mut any_ok = !blocks.is_empty();

    macro_rules! step {
        ($key:literal, $fut:expr) => {{
            match $fut.await {
                Ok(v) => {
                    any_ok = true;
                    result.insert($key.into(), v);
                }
                Err(e) => {
                    tracing::warn!(step = $key, error = %e, "bundle step failed");
                    result.insert($key.into(), json!({ "error": e.to_string() }));
                }
            }
        }};
    }

    if plan.flashcards > 0 {
        step!(
            "flashcards",
            generate_and_insert(pool, ai, subject_id, source, plan.flashcards, None, None, &blocks)
        );
    }
    if plan.exam_questions > 0 {
        step!(
            "exam",
            generate_exam_inner(
                pool, ai, subject_id, source, plan.exam_questions, None, None, exam_title, &blocks
            )
        );
    }
    if plan.feynman_concepts > 0 {
        step!(
            "feynman",
            generate_feynman_inner(pool, ai, subject_id, source, plan.feynman_concepts, None, None, &blocks)
        );
    }
    if plan.map_nodes > 0 {
        step!(
            "concept_map",
            generate_map_inner(pool, ai, subject_id, source, None, None, map_title, Some(plan.map_nodes))
        );
    }
    if plan.cornell_cues > 0 {
        step!(
            "cornell",
            generate_cornell_inner(
                pool, ai, subject_id, source, plan.cornell_cues, None, None, "Fiche Cornell"
            )
        );
    }
    if plan.schemas > 0 {
        step!(
            "schemas",
            generate_schemas_inner(pool, ai, subject_id, source, plan.schemas, None, None, &blocks)
        );
    }

    if !any_ok {
        return Err(AppError::AiSchema("aucun support n'a pu être généré".into()));
    }
    Ok(Value::Object(result))
}
