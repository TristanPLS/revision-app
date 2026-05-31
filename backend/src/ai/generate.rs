use std::collections::HashMap;

use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

use super::{prompts, schemas, AiClient};
use crate::error::{AppError, AppResult};

#[derive(Deserialize)]
struct GenFlashcards {
    flashcards: Vec<GenCard>,
}

#[derive(Deserialize)]
struct GenCard {
    front: String,
    back: String,
    #[allow(dead_code)]
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
) -> AppResult<Value> {
    let prompt = prompts::flashcards_prompt(source, count, block_title);
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
        sqlx::query(
            "INSERT INTO flashcards (subject_id, block_id, front, back, source) \
             VALUES ($1, $2, $3, $4, 'ai')",
        )
        .bind(subject_id)
        .bind(block_id)
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

    let outcome =
        generate_map_inner(&pool, &ai, subject_id, &source, block_id, block_title.as_deref(), &map_title)
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

async fn generate_map_inner(
    pool: &PgPool,
    ai: &AiClient,
    subject_id: Uuid,
    source: &str,
    block_id: Option<Uuid>,
    block_title: Option<&str>,
    map_title: &str,
) -> AppResult<Value> {
    let prompt = prompts::concept_map_prompt(source, block_title);
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
    #[allow(dead_code)]
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
) -> AppResult<Value> {
    let prompt = prompts::exam_prompt(source, count, block_title);
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

        sqlx::query(
            "INSERT INTO questions \
             (exam_id, block_id, position, qtype, prompt, options, answer_key, explanation, points, ai_graded) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        )
        .bind(exam_id)
        .bind(block_id)
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

    let outcome =
        generate_feynman_inner(&pool, &ai, subject_id, &source, count, block_id, block_title.as_deref())
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
) -> AppResult<Value> {
    let prompt = prompts::feynman_prompt(source, count, block_title);
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
        sqlx::query(
            "INSERT INTO feynman_concepts (subject_id, block_id, title, hint, source) \
             VALUES ($1, $2, $3, $4, 'ai')",
        )
        .bind(subject_id)
        .bind(block_id)
        .bind(title)
        .bind(c.hint)
        .execute(&mut *tx)
        .await?;
        created += 1;
    }
    tx.commit().await?;

    Ok(json!({ "created": created, "skipped": skipped }))
}
