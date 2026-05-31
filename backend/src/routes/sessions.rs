use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::post,
    Json, Router,
};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    models::{StartSession, StudySession},
    state::AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/sessions", post(start).get(list))
        .route("/sessions/{id}/close", post(close))
}

const SESSION_COLS: &str = "id, subject_id, started_at, ended_at, duration_s, mode, notes";

/// Soft session-length cap from the methodology (45 min).
const CAP_SECONDS: i32 = 45 * 60;

async fn start(
    State(s): State<AppState>,
    Json(body): Json<StartSession>,
) -> AppResult<(StatusCode, Json<StudySession>)> {
    let row = sqlx::query_as::<_, StudySession>(&format!(
        "INSERT INTO study_sessions (subject_id, mode) VALUES ($1, $2) RETURNING {SESSION_COLS}"
    ))
    .bind(body.subject_id)
    .bind(body.mode)
    .fetch_one(&s.pool)
    .await?;
    Ok((StatusCode::CREATED, Json(row)))
}

async fn close(State(s): State<AppState>, Path(id): Path<Uuid>) -> AppResult<Json<Value>> {
    let row = sqlx::query_as::<_, StudySession>(&format!(
        "UPDATE study_sessions \
         SET ended_at = now(), \
             duration_s = GREATEST(0, EXTRACT(EPOCH FROM (now() - started_at))::int) \
         WHERE id = $1 RETURNING {SESSION_COLS}"
    ))
    .bind(id)
    .fetch_optional(&s.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let duration_s = row.duration_s.unwrap_or(0);
    let over_cap = duration_s > CAP_SECONDS;
    Ok(Json(json!({
        "session": row,
        "duration_min": duration_s / 60,
        "over_cap": over_cap,
        "nudge": if over_cap {
            Some("Session > 45 min — fais une vraie pause, ton cerveau consolide mieux ainsi.")
        } else {
            None
        }
    })))
}

async fn list(State(s): State<AppState>) -> AppResult<Json<Vec<StudySession>>> {
    let rows = sqlx::query_as::<_, StudySession>(&format!(
        "SELECT {SESSION_COLS} FROM study_sessions ORDER BY started_at DESC LIMIT 50"
    ))
    .fetch_all(&s.pool)
    .await?;
    Ok(Json(rows))
}
