use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    models::{CreateSubject, Subject, UpdateSubject},
    state::AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/subjects", get(list).post(create))
        .route("/subjects/{id}", get(get_one).patch(update).delete(delete_one))
}

#[derive(Serialize, sqlx::FromRow)]
struct SubjectListItem {
    id: Uuid,
    name: String,
    description: Option<String>,
    exam_date: Option<NaiveDate>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    card_count: i64,
    due_count: i64,
}

async fn list(State(s): State<AppState>) -> AppResult<Json<Vec<SubjectListItem>>> {
    let rows = sqlx::query_as::<_, SubjectListItem>(
        "SELECT s.id, s.name, s.description, s.exam_date, s.created_at, s.updated_at, \
         COUNT(f.id) AS card_count, \
         COUNT(f.id) FILTER (WHERE f.due <= now()) AS due_count \
         FROM subjects s LEFT JOIN flashcards f ON f.subject_id = s.id \
         GROUP BY s.id ORDER BY s.created_at DESC",
    )
    .fetch_all(&s.pool)
    .await?;
    Ok(Json(rows))
}

async fn create(
    State(s): State<AppState>,
    Json(body): Json<CreateSubject>,
) -> AppResult<(StatusCode, Json<Subject>)> {
    if body.name.trim().is_empty() {
        return Err(AppError::Validation("le nom est requis".into()));
    }
    let row = sqlx::query_as::<_, Subject>(
        "INSERT INTO subjects (name, description, exam_date) VALUES ($1, $2, $3) \
         RETURNING id, name, description, exam_date, created_at, updated_at",
    )
    .bind(body.name.trim())
    .bind(body.description)
    .bind(body.exam_date)
    .fetch_one(&s.pool)
    .await?;
    Ok((StatusCode::CREATED, Json(row)))
}

async fn get_one(State(s): State<AppState>, Path(id): Path<Uuid>) -> AppResult<Json<Subject>> {
    let row = sqlx::query_as::<_, Subject>(
        "SELECT id, name, description, exam_date, created_at, updated_at FROM subjects WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&s.pool)
    .await?
    .ok_or(AppError::NotFound)?;
    Ok(Json(row))
}

async fn update(
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateSubject>,
) -> AppResult<Json<Subject>> {
    let row = sqlx::query_as::<_, Subject>(
        "UPDATE subjects SET \
           name = COALESCE($2, name), \
           description = COALESCE($3, description), \
           exam_date = COALESCE($4, exam_date), \
           updated_at = now() \
         WHERE id = $1 \
         RETURNING id, name, description, exam_date, created_at, updated_at",
    )
    .bind(id)
    .bind(body.name)
    .bind(body.description)
    .bind(body.exam_date)
    .fetch_optional(&s.pool)
    .await?
    .ok_or(AppError::NotFound)?;
    Ok(Json(row))
}

async fn delete_one(State(s): State<AppState>, Path(id): Path<Uuid>) -> AppResult<StatusCode> {
    let res = sqlx::query("DELETE FROM subjects WHERE id = $1")
        .bind(id)
        .execute(&s.pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}
