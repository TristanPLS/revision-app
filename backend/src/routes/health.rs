use axum::{extract::State, routing::get, Json, Router};
use serde_json::{json, Value};

use crate::{error::AppResult, state::AppState};

pub fn routes() -> Router<AppState> {
    Router::new().route("/health", get(health))
}

async fn health(State(state): State<AppState>) -> AppResult<Json<Value>> {
    sqlx::query("SELECT 1").execute(&state.pool).await?;
    let ai = state.ai.snapshot();
    Ok(Json(json!({
        "status": "ok",
        "ai_configured": ai.is_configured(),
        "provider": ai.provider.as_str(),
        "model": ai.model,
    })))
}
