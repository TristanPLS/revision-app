use std::collections::{HashMap, VecDeque};

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chrono::{Duration, Utc};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    models::{CreateFlashcard, Flashcard, ReviewRequest, ReviewResponse, UpdateFlashcard, FLASHCARD_COLS},
    srs::{Fsrs, MemoryState},
    state::AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/subjects/{id}/flashcards", get(list).post(create))
        .route("/subjects/{id}/flashcards/queue", get(queue))
        .route("/subjects/{id}/interleave", get(interleave))
        .route("/flashcards/{id}", axum::routing::patch(update).delete(delete_one))
        .route("/flashcards/{id}/review", post(review))
}

#[derive(Deserialize)]
struct ListParams {
    block_id: Option<Uuid>,
}

async fn list(
    State(s): State<AppState>,
    Path(subject_id): Path<Uuid>,
    Query(q): Query<ListParams>,
) -> AppResult<Json<Vec<Flashcard>>> {
    let rows = match q.block_id {
        Some(bid) => {
            sqlx::query_as::<_, Flashcard>(&format!(
                "SELECT {FLASHCARD_COLS} FROM flashcards WHERE subject_id = $1 AND block_id = $2 ORDER BY created_at DESC"
            ))
            .bind(subject_id)
            .bind(bid)
            .fetch_all(&s.pool)
            .await?
        }
        None => {
            sqlx::query_as::<_, Flashcard>(&format!(
                "SELECT {FLASHCARD_COLS} FROM flashcards WHERE subject_id = $1 ORDER BY created_at DESC"
            ))
            .bind(subject_id)
            .fetch_all(&s.pool)
            .await?
        }
    };
    Ok(Json(rows))
}

async fn create(
    State(s): State<AppState>,
    Path(subject_id): Path<Uuid>,
    Json(body): Json<CreateFlashcard>,
) -> AppResult<(StatusCode, Json<Flashcard>)> {
    if body.front.trim().is_empty() || body.back.trim().is_empty() {
        return Err(AppError::Validation("recto et verso sont requis".into()));
    }
    let row = sqlx::query_as::<_, Flashcard>(&format!(
        "INSERT INTO flashcards (subject_id, block_id, front, back, hint, source) \
         VALUES ($1, $2, $3, $4, $5, 'manual') RETURNING {FLASHCARD_COLS}"
    ))
    .bind(subject_id)
    .bind(body.block_id)
    .bind(body.front.trim())
    .bind(body.back.trim())
    .bind(body.hint)
    .fetch_one(&s.pool)
    .await?;
    Ok((StatusCode::CREATED, Json(row)))
}

async fn update(
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateFlashcard>,
) -> AppResult<Json<Flashcard>> {
    let row = sqlx::query_as::<_, Flashcard>(&format!(
        "UPDATE flashcards SET \
           front = COALESCE($2, front), \
           back  = COALESCE($3, back), \
           hint  = COALESCE($4, hint) \
         WHERE id = $1 RETURNING {FLASHCARD_COLS}"
    ))
    .bind(id)
    .bind(body.front)
    .bind(body.back)
    .bind(body.hint)
    .fetch_optional(&s.pool)
    .await?
    .ok_or(AppError::NotFound)?;
    Ok(Json(row))
}

async fn delete_one(State(s): State<AppState>, Path(id): Path<Uuid>) -> AppResult<StatusCode> {
    let res = sqlx::query("DELETE FROM flashcards WHERE id = $1")
        .bind(id)
        .execute(&s.pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
struct QueueParams {
    limit: Option<i64>,
}

/// Due cards now, soonest first.
async fn queue(
    State(s): State<AppState>,
    Path(subject_id): Path<Uuid>,
    Query(q): Query<QueueParams>,
) -> AppResult<Json<Vec<Flashcard>>> {
    let limit = q.limit.unwrap_or(20).clamp(1, 100);
    let rows = sqlx::query_as::<_, Flashcard>(&format!(
        "SELECT {FLASHCARD_COLS} FROM flashcards \
         WHERE subject_id = $1 AND due <= now() ORDER BY due ASC LIMIT $2"
    ))
    .bind(subject_id)
    .bind(limit)
    .fetch_all(&s.pool)
    .await?;
    Ok(Json(rows))
}

#[derive(Deserialize)]
struct InterleaveParams {
    cards: Option<i64>,
}

/// Due cards woven across blocks (round-robin), so no two consecutive cards
/// share a block while multiple blocks still have due cards. This is the
/// methodology's interleaving, applied to the daily session.
async fn interleave(
    State(s): State<AppState>,
    Path(subject_id): Path<Uuid>,
    Query(q): Query<InterleaveParams>,
) -> AppResult<Json<Vec<Flashcard>>> {
    let limit = q.cards.unwrap_or(20).clamp(1, 100);
    // Pull a generous pool of due cards, then interleave and cap.
    let rows = sqlx::query_as::<_, Flashcard>(&format!(
        "SELECT {FLASHCARD_COLS} FROM flashcards \
         WHERE subject_id = $1 AND due <= now() ORDER BY due ASC LIMIT $2"
    ))
    .bind(subject_id)
    .bind(limit * 3)
    .fetch_all(&s.pool)
    .await?;

    let mut woven = interleave_by_block(rows);
    woven.truncate(limit as usize);
    Ok(Json(woven))
}

fn interleave_by_block(cards: Vec<Flashcard>) -> Vec<Flashcard> {
    let mut buckets: Vec<VecDeque<Flashcard>> = Vec::new();
    let mut index: HashMap<Option<Uuid>, usize> = HashMap::new();
    for c in cards {
        let key = c.block_id;
        let idx = *index.entry(key).or_insert_with(|| {
            buckets.push(VecDeque::new());
            buckets.len() - 1
        });
        buckets[idx].push_back(c);
    }
    let mut out = Vec::new();
    loop {
        let mut progressed = false;
        for b in buckets.iter_mut() {
            if let Some(c) = b.pop_front() {
                out.push(c);
                progressed = true;
            }
        }
        if !progressed {
            break;
        }
    }
    out
}

/// The FSRS review endpoint: recompute memory state, persist, log the review.
async fn review(
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<ReviewRequest>,
) -> AppResult<Json<ReviewResponse>> {
    if !(1..=4).contains(&body.rating) {
        return Err(AppError::BadRequest("rating doit être 1..4".into()));
    }

    let card = sqlx::query_as::<_, Flashcard>(&format!(
        "SELECT {FLASHCARD_COLS} FROM flashcards WHERE id = $1"
    ))
    .bind(id)
    .fetch_optional(&s.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let now = Utc::now();
    let prev = match (card.stability, card.difficulty) {
        (Some(stability), Some(difficulty)) => Some(MemoryState {
            stability,
            difficulty,
        }),
        _ => None,
    };
    let elapsed_days = card
        .last_reviewed
        .map(|t| (now - t).num_days())
        .unwrap_or(0);

    let fsrs = Fsrs::new(s.cfg.fsrs_retention);
    let out = fsrs.schedule(prev, card.state, elapsed_days, body.rating);
    let due = now + Duration::days(out.scheduled_days as i64);
    let new_reps = card.reps + 1;
    let new_lapses = card.lapses + i32::from(out.lapsed);

    let mut tx = s.pool.begin().await?;
    sqlx::query(
        "UPDATE flashcards SET stability=$2, difficulty=$3, state=$4, due=$5, \
         last_reviewed=$6, reps=$7, lapses=$8 WHERE id=$1",
    )
    .bind(id)
    .bind(out.stability)
    .bind(out.difficulty)
    .bind(out.state)
    .bind(due)
    .bind(now)
    .bind(new_reps)
    .bind(new_lapses)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO reviews (flashcard_id, session_id, rating, reviewed_at, elapsed_days, \
         stability_after, difficulty_after, scheduled_days) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(id)
    .bind(body.session_id)
    .bind(body.rating)
    .bind(now)
    .bind(elapsed_days as i32)
    .bind(out.stability)
    .bind(out.difficulty)
    .bind(out.scheduled_days)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    Ok(Json(ReviewResponse {
        id,
        state: out.state,
        due,
        stability: Some(out.stability),
        difficulty: Some(out.difficulty),
        scheduled_days: out.scheduled_days,
        reps: new_reps,
        lapses: new_lapses,
        leitner_box: out.leitner_box,
    }))
}
