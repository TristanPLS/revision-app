use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    models::{CreateSchema, SchemaAsset, SchemaListItem, UpdateSchema},
    state::AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/subjects/{id}/schemas", get(list).post(create))
        .route(
            "/schemas/{id}",
            get(get_one).patch(update).delete(delete_one),
        )
}

const SCHEMA_COLS: &str =
    "id, subject_id, block_id, title, reference, drawing, created_at, updated_at";

async fn list(
    State(s): State<AppState>,
    Path(subject_id): Path<Uuid>,
) -> AppResult<Json<Vec<SchemaListItem>>> {
    let rows = sqlx::query_as::<_, SchemaListItem>(
        "SELECT id, title, created_at, (drawing IS NOT NULL) AS has_drawing \
         FROM schema_assets WHERE subject_id = $1 ORDER BY created_at DESC",
    )
    .bind(subject_id)
    .fetch_all(&s.pool)
    .await?;
    Ok(Json(rows))
}

async fn create(
    State(s): State<AppState>,
    Path(subject_id): Path<Uuid>,
    Json(body): Json<CreateSchema>,
) -> AppResult<(StatusCode, Json<SchemaAsset>)> {
    if body.title.trim().is_empty() {
        return Err(AppError::Validation("le titre est requis".into()));
    }
    let row = sqlx::query_as::<_, SchemaAsset>(&format!(
        "INSERT INTO schema_assets (subject_id, block_id, title, reference) \
         VALUES ($1, $2, $3, $4) RETURNING {SCHEMA_COLS}"
    ))
    .bind(subject_id)
    .bind(body.block_id)
    .bind(body.title.trim())
    .bind(body.reference)
    .fetch_one(&s.pool)
    .await?;
    Ok((StatusCode::CREATED, Json(row)))
}

async fn get_one(State(s): State<AppState>, Path(id): Path<Uuid>) -> AppResult<Json<SchemaAsset>> {
    let row = sqlx::query_as::<_, SchemaAsset>(&format!(
        "SELECT {SCHEMA_COLS} FROM schema_assets WHERE id = $1"
    ))
    .bind(id)
    .fetch_optional(&s.pool)
    .await?
    .ok_or(AppError::NotFound)?;
    Ok(Json(row))
}

async fn update(
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateSchema>,
) -> AppResult<Json<SchemaAsset>> {
    let row = sqlx::query_as::<_, SchemaAsset>(&format!(
        "UPDATE schema_assets SET \
           title = COALESCE($2, title), \
           reference = COALESCE($3, reference), \
           drawing = COALESCE($4, drawing), \
           updated_at = now() \
         WHERE id = $1 RETURNING {SCHEMA_COLS}"
    ))
    .bind(id)
    .bind(body.title)
    .bind(body.reference)
    .bind(body.drawing)
    .fetch_optional(&s.pool)
    .await?
    .ok_or(AppError::NotFound)?;
    Ok(Json(row))
}

async fn delete_one(State(s): State<AppState>, Path(id): Path<Uuid>) -> AppResult<StatusCode> {
    let res = sqlx::query("DELETE FROM schema_assets WHERE id = $1")
        .bind(id)
        .execute(&s.pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}
