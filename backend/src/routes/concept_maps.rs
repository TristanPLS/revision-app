use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    models::{ConceptMap, ConceptMapDetail, ConceptMapEdge, ConceptMapListItem, ConceptMapNode},
    state::AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/subjects/{id}/maps", get(list))
        .route("/maps/{id}", get(get_one).delete(delete_one))
}

async fn list(State(s): State<AppState>, Path(subject_id): Path<Uuid>) -> AppResult<Json<Vec<ConceptMapListItem>>> {
    let rows = sqlx::query_as::<_, ConceptMapListItem>(
        "SELECT m.id, m.title, m.source, m.created_at, \
           (SELECT COUNT(*) FROM concept_map_nodes n WHERE n.map_id = m.id) AS node_count \
         FROM concept_maps m WHERE m.subject_id = $1 ORDER BY m.created_at DESC",
    )
    .bind(subject_id)
    .fetch_all(&s.pool)
    .await?;
    Ok(Json(rows))
}

async fn get_one(State(s): State<AppState>, Path(id): Path<Uuid>) -> AppResult<Json<ConceptMapDetail>> {
    let map = sqlx::query_as::<_, ConceptMap>(
        "SELECT id, subject_id, block_id, title, source, created_at FROM concept_maps WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&s.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let nodes = sqlx::query_as::<_, ConceptMapNode>(
        "SELECT id, label, parent_id FROM concept_map_nodes WHERE map_id = $1",
    )
    .bind(id)
    .fetch_all(&s.pool)
    .await?;

    let edges = sqlx::query_as::<_, ConceptMapEdge>(
        "SELECT id, from_node, to_node, label FROM concept_map_edges WHERE map_id = $1",
    )
    .bind(id)
    .fetch_all(&s.pool)
    .await?;

    Ok(Json(ConceptMapDetail {
        id: map.id,
        subject_id: map.subject_id,
        title: map.title,
        nodes,
        edges,
    }))
}

async fn delete_one(State(s): State<AppState>, Path(id): Path<Uuid>) -> AppResult<StatusCode> {
    let res = sqlx::query("DELETE FROM concept_maps WHERE id = $1")
        .bind(id)
        .execute(&s.pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}
