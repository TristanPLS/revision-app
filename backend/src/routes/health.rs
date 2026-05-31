use axum::{extract::State, routing::get, Json, Router};
use serde_json::{json, Value};

use crate::{error::AppResult, state::AppState};

pub fn routes() -> Router<AppState> {
    Router::new().route("/health", get(health))
}

async fn health(State(state): State<AppState>) -> AppResult<Json<Value>> {
    sqlx::query("SELECT 1").execute(&state.pool).await?;
    Ok(Json(json!({
        "status": "ok",
        "ai_configured": state.ai.is_configured(),
        "model": state.ai.model(),
    })))
}
