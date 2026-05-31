use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, patch},
    Json, Router,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    models::{Block, CreateBlock},
    state::AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/subjects/{id}/blocks", get(list).post(create))
        .route("/blocks/{id}", patch(update).delete(delete_one))
}

async fn list(State(s): State<AppState>, Path(subject_id): Path<Uuid>) -> AppResult<Json<Vec<Block>>> {
    let rows = sqlx::query_as::<_, Block>(
        "SELECT id, subject_id, code, title, summary, position, created_at \
         FROM blocks WHERE subject_id = $1 ORDER BY position, created_at",
    )
    .bind(subject_id)
    .fetch_all(&s.pool)
    .await?;
    Ok(Json(rows))
}

async fn create(
    State(s): State<AppState>,
    Path(subject_id): Path<Uuid>,
    Json(body): Json<CreateBlock>,
) -> AppResult<(StatusCode, Json<Block>)> {
    if body.title.trim().is_empty() {
        return Err(AppError::Validation("le titre est requis".into()));
    }
    let row = sqlx::query_as::<_, Block>(
        "INSERT INTO blocks (subject_id, code, title, summary, position) \
         VALUES ($1, $2, $3, $4, COALESCE($5, 0)) \
         RETURNING id, subject_id, code, title, summary, position, created_at",
    )
    .bind(subject_id)
    .bind(body.code)
    .bind(body.title.trim())
    .bind(body.summary)
    .bind(body.position)
    .fetch_one(&s.pool)
    .await?;
    Ok((StatusCode::CREATED, Json(row)))
}

#[derive(Deserialize)]
struct UpdateBlock {
    code: Option<String>,
    title: Option<String>,
    summary: Option<String>,
    position: Option<i32>,
}

async fn update(
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateBlock>,
) -> AppResult<Json<Block>> {
    let row = sqlx::query_as::<_, Block>(
        "UPDATE blocks SET \
           code = COALESCE($2, code), \
           title = COALESCE($3, title), \
           summary = COALESCE($4, summary), \
           position = COALESCE($5, position) \
         WHERE id = $1 \
         RETURNING id, subject_id, code, title, summary, position, created_at",
    )
    .bind(id)
    .bind(body.code)
    .bind(body.title)
    .bind(body.summary)
    .bind(body.position)
    .fetch_optional(&s.pool)
    .await?
    .ok_or(AppError::NotFound)?;
    Ok(Json(row))
}

async fn delete_one(State(s): State<AppState>, Path(id): Path<Uuid>) -> AppResult<StatusCode> {
    let res = sqlx::query("DELETE FROM blocks WHERE id = $1")
        .bind(id)
        .execute(&s.pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}
