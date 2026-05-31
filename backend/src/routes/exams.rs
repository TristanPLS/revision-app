use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    ai,
    error::{AppError, AppResult},
    models::{
        AttemptResult, AttemptStart, BlockScore, Exam, ExamDetail, ExamListItem, QuestionPublic,
        ResultItem, SubmitAttempt,
    },
    state::AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/subjects/{id}/exams", get(list))
        .route("/exams/{id}", get(get_one).delete(delete_one))
        .route("/exams/{id}/attempts", post(start_attempt))
        .route("/attempts/{id}/submit", post(submit))
        .route("/attempts/{id}", get(results))
}

async fn list(State(s): State<AppState>, Path(subject_id): Path<Uuid>) -> AppResult<Json<Vec<ExamListItem>>> {
    let rows = sqlx::query_as::<_, ExamListItem>(
        "SELECT e.id, e.title, e.time_limit_s, e.created_at, \
           (SELECT COUNT(*) FROM questions q WHERE q.exam_id = e.id) AS question_count, \
           (SELECT COUNT(*) FROM exam_attempts a WHERE a.exam_id = e.id AND a.status = 'graded') AS attempt_count, \
           (SELECT MAX(a.score) FROM exam_attempts a WHERE a.exam_id = e.id) AS best_score, \
           (SELECT MAX(a.max_score) FROM exam_attempts a WHERE a.exam_id = e.id) AS max_score \
         FROM exams e WHERE e.subject_id = $1 ORDER BY e.created_at DESC",
    )
    .bind(subject_id)
    .fetch_all(&s.pool)
    .await?;
    Ok(Json(rows))
}

async fn get_one(State(s): State<AppState>, Path(id): Path<Uuid>) -> AppResult<Json<ExamDetail>> {
    let exam = sqlx::query_as::<_, Exam>(
        "SELECT id, subject_id, title, time_limit_s, source, created_at FROM exams WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&s.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let questions = sqlx::query_as::<_, QuestionPublic>(
        "SELECT id, block_id, position, qtype, prompt, options, points \
         FROM questions WHERE exam_id = $1 ORDER BY position",
    )
    .bind(id)
    .fetch_all(&s.pool)
    .await?;

    Ok(Json(ExamDetail {
        id: exam.id,
        subject_id: exam.subject_id,
        title: exam.title,
        time_limit_s: exam.time_limit_s,
        questions,
    }))
}

async fn delete_one(State(s): State<AppState>, Path(id): Path<Uuid>) -> AppResult<StatusCode> {
    let res = sqlx::query("DELETE FROM exams WHERE id = $1")
        .bind(id)
        .execute(&s.pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn start_attempt(State(s): State<AppState>, Path(exam_id): Path<Uuid>) -> AppResult<Json<AttemptStart>> {
    let time_limit_s: Option<i32> =
        sqlx::query_scalar("SELECT time_limit_s FROM exams WHERE id = $1")
            .bind(exam_id)
            .fetch_optional(&s.pool)
            .await?
            .ok_or(AppError::NotFound)?;

    let (attempt_id, started_at) = sqlx::query_as::<_, (Uuid, DateTime<Utc>)>(
        "INSERT INTO exam_attempts (exam_id) VALUES ($1) RETURNING id, started_at",
    )
    .bind(exam_id)
    .fetch_one(&s.pool)
    .await?;

    Ok(Json(AttemptStart {
        attempt_id,
        started_at,
        time_limit_s,
    }))
}

#[derive(sqlx::FromRow)]
struct QFull {
    id: Uuid,
    prompt: String,
    answer_key: Option<String>,
    explanation: Option<String>,
    points: i32,
    ai_graded: bool,
}

async fn submit(
    State(s): State<AppState>,
    Path(attempt_id): Path<Uuid>,
    Json(body): Json<SubmitAttempt>,
) -> AppResult<Json<AttemptResult>> {
    let exam_id: Uuid = sqlx::query_scalar("SELECT exam_id FROM exam_attempts WHERE id = $1")
        .bind(attempt_id)
        .fetch_optional(&s.pool)
        .await?
        .ok_or(AppError::NotFound)?;

    let questions = sqlx::query_as::<_, QFull>(
        "SELECT id, prompt, answer_key, explanation, points, ai_graded \
         FROM questions WHERE exam_id = $1",
    )
    .bind(exam_id)
    .fetch_all(&s.pool)
    .await?;

    let responses: HashMap<Uuid, String> = body
        .answers
        .into_iter()
        .filter_map(|a| a.response.map(|r| (a.question_id, r)))
        .collect();

    let mut total_awarded = 0.0f32;
    let mut max_score = 0.0f32;

    for q in &questions {
        max_score += q.points as f32;
        let resp = responses.get(&q.id);
        let (awarded, is_correct, feedback): (f32, Option<bool>, Option<String>) = match resp {
            None => (0.0, Some(false), None),
            Some(r) if r.trim().is_empty() => (0.0, Some(false), None),
            Some(r) => {
                if q.ai_graded {
                    let (score, fb) = ai::generate::grade_answer(
                        &s.ai,
                        &q.prompt,
                        q.explanation.as_deref(),
                        r,
                        q.points,
                    )
                    .await;
                    (score, Some(score >= q.points as f32 * 0.5), Some(fb))
                } else {
                    let correct = q
                        .answer_key
                        .as_deref()
                        .map(|k| k.trim().eq_ignore_ascii_case(r.trim()))
                        .unwrap_or(false);
                    (if correct { q.points as f32 } else { 0.0 }, Some(correct), None)
                }
            }
        };
        total_awarded += awarded;

        sqlx::query(
            "INSERT INTO exam_answers (attempt_id, question_id, response, is_correct, awarded, ai_feedback) \
             VALUES ($1, $2, $3, $4, $5, $6) \
             ON CONFLICT (attempt_id, question_id) DO UPDATE SET \
               response = EXCLUDED.response, is_correct = EXCLUDED.is_correct, \
               awarded = EXCLUDED.awarded, ai_feedback = EXCLUDED.ai_feedback",
        )
        .bind(attempt_id)
        .bind(q.id)
        .bind(resp)
        .bind(is_correct)
        .bind(awarded)
        .bind(feedback)
        .execute(&s.pool)
        .await?;
    }

    sqlx::query(
        "UPDATE exam_attempts SET status = 'graded', submitted_at = now(), score = $2, max_score = $3 \
         WHERE id = $1",
    )
    .bind(attempt_id)
    .bind(total_awarded)
    .bind(max_score)
    .execute(&s.pool)
    .await?;

    Ok(Json(build_result(&s, attempt_id).await?))
}

async fn results(State(s): State<AppState>, Path(attempt_id): Path<Uuid>) -> AppResult<Json<AttemptResult>> {
    Ok(Json(build_result(&s, attempt_id).await?))
}

async fn build_result(s: &AppState, attempt_id: Uuid) -> AppResult<AttemptResult> {
    let attempt = sqlx::query_as::<_, (crate::models::AttemptStatus, Option<f32>, Option<f32>)>(
        "SELECT status, score, max_score FROM exam_attempts WHERE id = $1",
    )
    .bind(attempt_id)
    .fetch_optional(&s.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let items = sqlx::query_as::<_, ResultItem>(
        "SELECT q.id AS question_id, q.block_id, b.title AS block_title, q.qtype, q.prompt, \
           q.options, q.answer_key, q.explanation, q.points, \
           a.response, a.is_correct, a.awarded, a.ai_feedback \
         FROM questions q \
         LEFT JOIN blocks b ON b.id = q.block_id \
         LEFT JOIN exam_answers a ON a.question_id = q.id AND a.attempt_id = $1 \
         WHERE q.exam_id = (SELECT exam_id FROM exam_attempts WHERE id = $1) \
         ORDER BY q.position",
    )
    .bind(attempt_id)
    .fetch_all(&s.pool)
    .await?;

    // Aggregate per block (preserving first-seen order).
    let mut order: Vec<Option<Uuid>> = Vec::new();
    let mut agg: HashMap<Option<Uuid>, (String, f32, f32)> = HashMap::new();
    for it in &items {
        let entry = agg.entry(it.block_id).or_insert_with(|| {
            order.push(it.block_id);
            (
                it.block_title.clone().unwrap_or_else(|| "Sans bloc".to_string()),
                0.0,
                0.0,
            )
        });
        entry.1 += it.awarded.unwrap_or(0.0);
        entry.2 += it.points as f32;
    }
    let by_block = order
        .into_iter()
        .map(|k| {
            let (title, awarded, max) = agg.get(&k).cloned().unwrap();
            BlockScore {
                block_id: k,
                title,
                awarded,
                max,
            }
        })
        .collect();

    Ok(AttemptResult {
        attempt_id,
        status: attempt.0,
        score: attempt.1,
        max_score: attempt.2,
        items,
        by_block,
    })
}
