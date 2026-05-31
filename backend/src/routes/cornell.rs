use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    models::{
        CornellCue, CornellNote, CornellNoteDetail, CornellNoteItem, CreateCornellNote, Flashcard,
        FLASHCARD_COLS,
    },
    state::AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/subjects/{id}/cornell", get(list).post(create))
        .route("/cornell/{id}", get(get_one).delete(delete_one))
        .route("/cornell/cues/{id}/to-flashcard", post(cue_to_flashcard))
}

async fn list(State(s): State<AppState>, Path(subject_id): Path<Uuid>) -> AppResult<Json<Vec<CornellNoteItem>>> {
    let rows = sqlx::query_as::<_, CornellNoteItem>(
        "SELECT n.id, n.title, n.created_at, \
           (SELECT COUNT(*) FROM cornell_cues c WHERE c.note_id = n.id) AS cue_count \
         FROM cornell_notes n WHERE n.subject_id = $1 ORDER BY n.created_at DESC",
    )
    .bind(subject_id)
    .fetch_all(&s.pool)
    .await?;
    Ok(Json(rows))
}

async fn create(
    State(s): State<AppState>,
    Path(subject_id): Path<Uuid>,
    Json(body): Json<CreateCornellNote>,
) -> AppResult<(StatusCode, Json<CornellNoteDetail>)> {
    if body.title.trim().is_empty() || body.body.trim().is_empty() {
        return Err(AppError::Validation("titre et contenu requis".into()));
    }

    let mut tx = s.pool.begin().await?;
    let note = sqlx::query_as::<_, CornellNote>(
        "INSERT INTO cornell_notes (subject_id, block_id, title, body, summary) \
         VALUES ($1, $2, $3, $4, $5) \
         RETURNING id, subject_id, block_id, title, body, summary, created_at",
    )
    .bind(subject_id)
    .bind(body.block_id)
    .bind(body.title.trim())
    .bind(body.body)
    .bind(body.summary)
    .fetch_one(&mut *tx)
    .await?;

    for cue in &body.cues {
        if cue.question.trim().is_empty() {
            continue;
        }
        sqlx::query("INSERT INTO cornell_cues (note_id, question, answer) VALUES ($1, $2, $3)")
            .bind(note.id)
            .bind(cue.question.trim())
            .bind(cue.answer.as_deref())
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;

    Ok((StatusCode::CREATED, Json(build_detail(&s, note.id).await?)))
}

async fn get_one(State(s): State<AppState>, Path(id): Path<Uuid>) -> AppResult<Json<CornellNoteDetail>> {
    Ok(Json(build_detail(&s, id).await?))
}

async fn delete_one(State(s): State<AppState>, Path(id): Path<Uuid>) -> AppResult<StatusCode> {
    let res = sqlx::query("DELETE FROM cornell_notes WHERE id = $1")
        .bind(id)
        .execute(&s.pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn build_detail(s: &AppState, note_id: Uuid) -> AppResult<CornellNoteDetail> {
    let note = sqlx::query_as::<_, CornellNote>(
        "SELECT id, subject_id, block_id, title, body, summary, created_at FROM cornell_notes WHERE id = $1",
    )
    .bind(note_id)
    .fetch_optional(&s.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let cues = sqlx::query_as::<_, CornellCue>(
        "SELECT id, note_id, question, answer, flashcard_id FROM cornell_cues WHERE note_id = $1 ORDER BY id",
    )
    .bind(note_id)
    .fetch_all(&s.pool)
    .await?;

    Ok(CornellNoteDetail {
        id: note.id,
        subject_id: note.subject_id,
        block_id: note.block_id,
        title: note.title,
        body: note.body,
        summary: note.summary,
        created_at: note.created_at,
        cues,
    })
}

#[derive(sqlx::FromRow)]
struct CueJoin {
    question: String,
    answer: Option<String>,
    flashcard_id: Option<Uuid>,
    note_id: Uuid,
    subject_id: Uuid,
    block_id: Option<Uuid>,
}

/// Convert a Cornell cue (margin question) into a flashcard, linking both ways.
async fn cue_to_flashcard(State(s): State<AppState>, Path(cue_id): Path<Uuid>) -> AppResult<Json<Flashcard>> {
    let cue = sqlx::query_as::<_, CueJoin>(
        "SELECT c.question, c.answer, c.flashcard_id, c.note_id, n.subject_id, n.block_id \
         FROM cornell_cues c JOIN cornell_notes n ON n.id = c.note_id WHERE c.id = $1",
    )
    .bind(cue_id)
    .fetch_optional(&s.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    if cue.flashcard_id.is_some() {
        return Err(AppError::Conflict("déjà convertie en flashcard".into()));
    }

    let back = cue
        .answer
        .as_deref()
        .filter(|a| !a.trim().is_empty())
        .unwrap_or("(à compléter)")
        .to_string();

    let mut tx = s.pool.begin().await?;
    let card = sqlx::query_as::<_, Flashcard>(&format!(
        "INSERT INTO flashcards (subject_id, block_id, front, back, source, cornell_note_id) \
         VALUES ($1, $2, $3, $4, 'cornell', $5) RETURNING {FLASHCARD_COLS}"
    ))
    .bind(cue.subject_id)
    .bind(cue.block_id)
    .bind(cue.question)
    .bind(back)
    .bind(cue.note_id)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query("UPDATE cornell_cues SET flashcard_id = $1 WHERE id = $2")
        .bind(card.id)
        .bind(cue_id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;

    Ok(Json(card))
}
