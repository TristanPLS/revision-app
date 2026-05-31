use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use uuid::Uuid;

use crate::{
    ai,
    error::{AppError, AppResult},
    models::{CreateFeynmanAttempt, CreateFeynmanConcept, FeynmanAttempt, FeynmanConcept, FeynmanConceptItem},
    state::AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/subjects/{id}/feynman", get(list).post(create))
        .route("/feynman/{id}", get(get_one).delete(delete_one))
        .route("/feynman/{id}/attempts", get(history).post(create_attempt))
}

async fn list(State(s): State<AppState>, Path(subject_id): Path<Uuid>) -> AppResult<Json<Vec<FeynmanConceptItem>>> {
    let rows = sqlx::query_as::<_, FeynmanConceptItem>(
        "SELECT c.id, c.block_id, c.title, c.hint, c.source, c.created_at, \
           (SELECT COUNT(*) FROM feynman_attempts a WHERE a.concept_id = c.id) AS attempts, \
           (SELECT a.self_rating FROM feynman_attempts a WHERE a.concept_id = c.id ORDER BY a.created_at DESC LIMIT 1) AS last_rating \
         FROM feynman_concepts c WHERE c.subject_id = $1 ORDER BY c.created_at DESC",
    )
    .bind(subject_id)
    .fetch_all(&s.pool)
    .await?;
    Ok(Json(rows))
}

const CONCEPT_COLS: &str = "id, subject_id, block_id, title, hint, source, created_at";

async fn create(
    State(s): State<AppState>,
    Path(subject_id): Path<Uuid>,
    Json(body): Json<CreateFeynmanConcept>,
) -> AppResult<(StatusCode, Json<FeynmanConcept>)> {
    if body.title.trim().is_empty() {
        return Err(AppError::Validation("le concept est requis".into()));
    }
    let row = sqlx::query_as::<_, FeynmanConcept>(&format!(
        "INSERT INTO feynman_concepts (subject_id, block_id, title, hint) \
         VALUES ($1, $2, $3, $4) RETURNING {CONCEPT_COLS}"
    ))
    .bind(subject_id)
    .bind(body.block_id)
    .bind(body.title.trim())
    .bind(body.hint)
    .fetch_one(&s.pool)
    .await?;
    Ok((StatusCode::CREATED, Json(row)))
}

async fn get_one(State(s): State<AppState>, Path(id): Path<Uuid>) -> AppResult<Json<FeynmanConcept>> {
    let row = sqlx::query_as::<_, FeynmanConcept>(&format!(
        "SELECT {CONCEPT_COLS} FROM feynman_concepts WHERE id = $1"
    ))
    .bind(id)
    .fetch_optional(&s.pool)
    .await?
    .ok_or(AppError::NotFound)?;
    Ok(Json(row))
}

async fn delete_one(State(s): State<AppState>, Path(id): Path<Uuid>) -> AppResult<StatusCode> {
    let res = sqlx::query("DELETE FROM feynman_concepts WHERE id = $1")
        .bind(id)
        .execute(&s.pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn history(State(s): State<AppState>, Path(concept_id): Path<Uuid>) -> AppResult<Json<Vec<FeynmanAttempt>>> {
    let rows = sqlx::query_as::<_, FeynmanAttempt>(
        "SELECT id, concept_id, self_rating, hesitations, duration_s, explanation, ai_feedback, ai_score, created_at \
         FROM feynman_attempts WHERE concept_id = $1 ORDER BY created_at DESC LIMIT 20",
    )
    .bind(concept_id)
    .fetch_all(&s.pool)
    .await?;
    Ok(Json(rows))
}

async fn create_attempt(
    State(s): State<AppState>,
    Path(concept_id): Path<Uuid>,
    Json(body): Json<CreateFeynmanAttempt>,
) -> AppResult<Json<FeynmanAttempt>> {
    let concept = sqlx::query_as::<_, FeynmanConcept>(&format!(
        "SELECT {CONCEPT_COLS} FROM feynman_concepts WHERE id = $1"
    ))
    .bind(concept_id)
    .fetch_optional(&s.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    // Optional AI feedback on a typed explanation (graded out of 100).
    let mut ai_feedback: Option<String> = None;
    let mut ai_score: Option<i16> = None;
    if let Some(expl) = body.explanation.as_deref() {
        if !expl.trim().is_empty() && s.ai.is_configured() {
            let (score, fb) =
                ai::generate::grade_answer(&s.ai, &concept.title, concept.hint.as_deref(), expl, 100)
                    .await;
            ai_feedback = Some(fb);
            ai_score = Some(score.round() as i16);
        }
    }

    let row = sqlx::query_as::<_, FeynmanAttempt>(
        "INSERT INTO feynman_attempts \
           (concept_id, session_id, self_rating, hesitations, duration_s, explanation, ai_feedback, ai_score) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
         RETURNING id, concept_id, self_rating, hesitations, duration_s, explanation, ai_feedback, ai_score, created_at",
    )
    .bind(concept_id)
    .bind(body.session_id)
    .bind(body.self_rating)
    .bind(body.hesitations.unwrap_or(0))
    .bind(body.duration_s)
    .bind(body.explanation)
    .bind(ai_feedback)
    .bind(ai_score)
    .fetch_one(&s.pool)
    .await?;

    Ok(Json(row))
}
