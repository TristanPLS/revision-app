pub mod blocks;
pub mod cornell;
pub mod exams;
pub mod feynman;
pub mod flashcards;
pub mod health;
pub mod sessions;
pub mod sources;
pub mod stats;
pub mod subjects;

use crate::state::AppState;
use axum::Router;

/// Build the full router with everything nested under `/api`.
pub fn router(state: AppState) -> Router {
    let api = Router::new()
        .merge(health::routes())
        .merge(subjects::routes())
        .merge(blocks::routes())
        .merge(sources::routes())
        .merge(flashcards::routes())
        .merge(exams::routes())
        .merge(feynman::routes())
        .merge(cornell::routes())
        .merge(sessions::routes())
        .merge(stats::routes());

    Router::new().nest("/api", api).with_state(state)
}
