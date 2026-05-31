use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{
    ai,
    error::{AppError, AppResult},
    models::{CreateSource, GenerateRequest, GenerationJob, JobKind, SourceDocument},
    state::AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/subjects/{id}/sources", get(list).post(create))
        .route("/sources/{id}", get(get_one).delete(delete_one))
        .route("/sources/{id}/generate", post(generate))
        .route("/jobs/{id}", get(job_status))
        .route("/subjects/{id}/jobs", get(list_jobs))
}

const SOURCE_COLS: &str = "id, subject_id, block_id, title, content, created_at";

async fn list(
    State(s): State<AppState>,
    Path(subject_id): Path<Uuid>,
) -> AppResult<Json<Vec<SourceDocument>>> {
    let rows = sqlx::query_as::<_, SourceDocument>(&format!(
        "SELECT {SOURCE_COLS} FROM source_documents WHERE subject_id = $1 ORDER BY created_at DESC"
    ))
    .bind(subject_id)
    .fetch_all(&s.pool)
    .await?;
    Ok(Json(rows))
}

async fn create(
    State(s): State<AppState>,
    Path(subject_id): Path<Uuid>,
    Json(body): Json<CreateSource>,
) -> AppResult<(StatusCode, Json<SourceDocument>)> {
    if body.content.trim().is_empty() {
        return Err(AppError::Validation("le contenu du cours est requis".into()));
    }
    let title = if body.title.trim().is_empty() {
        "Document".to_string()
    } else {
        body.title.trim().to_string()
    };
    let row = sqlx::query_as::<_, SourceDocument>(&format!(
        "INSERT INTO source_documents (subject_id, block_id, title, content) \
         VALUES ($1, $2, $3, $4) RETURNING {SOURCE_COLS}"
    ))
    .bind(subject_id)
    .bind(body.block_id)
    .bind(title)
    .bind(body.content)
    .fetch_one(&s.pool)
    .await?;
    Ok((StatusCode::CREATED, Json(row)))
}

async fn get_one(State(s): State<AppState>, Path(id): Path<Uuid>) -> AppResult<Json<SourceDocument>> {
    let row = sqlx::query_as::<_, SourceDocument>(&format!(
        "SELECT {SOURCE_COLS} FROM source_documents WHERE id = $1"
    ))
    .bind(id)
    .fetch_optional(&s.pool)
    .await?
    .ok_or(AppError::NotFound)?;
    Ok(Json(row))
}

async fn delete_one(State(s): State<AppState>, Path(id): Path<Uuid>) -> AppResult<StatusCode> {
    let res = sqlx::query("DELETE FROM source_documents WHERE id = $1")
        .bind(id)
        .execute(&s.pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

/// Trigger AI generation. Creates a job row, spawns the work, returns the job id.
async fn generate(
    State(s): State<AppState>,
    Path(source_id): Path<Uuid>,
    Json(req): Json<GenerateRequest>,
) -> AppResult<Json<Value>> {
    if !s.ai.is_configured() {
        return Err(AppError::AiNotConfigured);
    }
    if !matches!(
        req.kind,
        JobKind::Flashcards | JobKind::Exam | JobKind::Feynman | JobKind::ConceptMap
    ) {
        return Err(AppError::BadRequest(
            "kind non supporté".into(),
        ));
    }

    let src = sqlx::query_as::<_, SourceDocument>(&format!(
        "SELECT {SOURCE_COLS} FROM source_documents WHERE id = $1"
    ))
    .bind(source_id)
    .fetch_optional(&s.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let count = req.count.unwrap_or(10).clamp(1, 50);
    let block_id = req.block_id.or(src.block_id);
    let block_title = match block_id {
        Some(bid) => {
            sqlx::query_scalar::<_, String>("SELECT title FROM blocks WHERE id = $1")
                .bind(bid)
                .fetch_optional(&s.pool)
                .await?
        }
        None => None,
    };

    let job_id: Uuid = sqlx::query_scalar(
        "INSERT INTO generation_jobs (subject_id, source_id, kind, status, model) \
         VALUES ($1, $2, $3, 'pending', $4) RETURNING id",
    )
    .bind(src.subject_id)
    .bind(source_id)
    .bind(req.kind)
    .bind(s.ai.model())
    .fetch_one(&s.pool)
    .await?;

    let pool = s.pool.clone();
    let ai_client = s.ai.clone();
    let subject_id = src.subject_id;
    let content = src.content.clone();
    let kind = req.kind;
    let exam_title = req
        .title
        .clone()
        .unwrap_or_else(|| "Examen blanc".to_string());
    let map_title = req
        .title
        .clone()
        .unwrap_or_else(|| "Carte conceptuelle".to_string());
    tokio::spawn(async move {
        match kind {
            JobKind::Exam => {
                ai::generate::run_exam(
                    pool, ai_client, job_id, subject_id, content, count, block_id,
                    block_title, exam_title,
                )
                .await;
            }
            JobKind::ConceptMap => {
                ai::generate::run_concept_map(
                    pool, ai_client, job_id, subject_id, content, block_id, block_title, map_title,
                )
                .await;
            }
            JobKind::Feynman => {
                ai::generate::run_feynman(
                    pool, ai_client, job_id, subject_id, content, count, block_id, block_title,
                )
                .await;
            }
            _ => {
                ai::generate::run_flashcards(
                    pool, ai_client, job_id, subject_id, content, count, block_id, block_title,
                )
                .await;
            }
        }
    });

    Ok(Json(json!({ "job_id": job_id, "status": "pending" })))
}

async fn job_status(State(s): State<AppState>, Path(id): Path<Uuid>) -> AppResult<Json<GenerationJob>> {
    let row = sqlx::query_as::<_, GenerationJob>(
        "SELECT id, subject_id, source_id, kind, status, model, result, error, created_at, finished_at \
         FROM generation_jobs WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&s.pool)
    .await?
    .ok_or(AppError::NotFound)?;
    Ok(Json(row))
}

async fn list_jobs(
    State(s): State<AppState>,
    Path(subject_id): Path<Uuid>,
) -> AppResult<Json<Vec<GenerationJob>>> {
    let rows = sqlx::query_as::<_, GenerationJob>(
        "SELECT id, subject_id, source_id, kind, status, model, result, error, created_at, finished_at \
         FROM generation_jobs WHERE subject_id = $1 ORDER BY created_at DESC LIMIT 20",
    )
    .bind(subject_id)
    .fetch_all(&s.pool)
    .await?;
    Ok(Json(rows))
}
