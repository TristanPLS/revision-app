use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use chrono::{Datelike, Local, NaiveDate, Timelike, Weekday};
use uuid::Uuid;

use crate::{
    error::AppResult,
    models::{BlockMastery, Guardrails, SubjectStats},
    state::AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/subjects/{id}/stats", get(subject_stats))
        .route("/guardrails", get(guardrails))
}

#[derive(sqlx::FromRow)]
struct StatsRow {
    total_cards: i64,
    due_now: i64,
    box1: i64,
    box2: i64,
    box3: i64,
    box4: i64,
    box5: i64,
}

async fn subject_stats(
    State(s): State<AppState>,
    Path(subject_id): Path<Uuid>,
) -> AppResult<Json<SubjectStats>> {
    // Leitner boxes derived from stability (≈ interval at retention 0.9).
    let row = sqlx::query_as::<_, StatsRow>(
        "SELECT \
           COUNT(*) AS total_cards, \
           COUNT(*) FILTER (WHERE due <= now()) AS due_now, \
           COUNT(*) FILTER (WHERE state IN ('new','learning','relearning') OR stability IS NULL OR stability < 1) AS box1, \
           COUNT(*) FILTER (WHERE state = 'review' AND stability >= 1  AND stability < 4)  AS box2, \
           COUNT(*) FILTER (WHERE state = 'review' AND stability >= 4  AND stability < 10) AS box3, \
           COUNT(*) FILTER (WHERE state = 'review' AND stability >= 10 AND stability < 30) AS box4, \
           COUNT(*) FILTER (WHERE state = 'review' AND stability >= 30) AS box5 \
         FROM flashcards WHERE subject_id = $1",
    )
    .bind(subject_id)
    .fetch_one(&s.pool)
    .await?;

    let reviews_total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM reviews r \
         JOIN flashcards f ON f.id = r.flashcard_id WHERE f.subject_id = $1",
    )
    .bind(subject_id)
    .fetch_one(&s.pool)
    .await?;

    let weakest_blocks = sqlx::query_as::<_, BlockMastery>(
        "SELECT b.id AS block_id, b.title, \
           COUNT(f.id) AS total, \
           COUNT(f.id) FILTER (WHERE f.due <= now()) AS due, \
           COALESCE(AVG(CASE WHEN f.state = 'review' AND f.stability >= 10 THEN 1.0 ELSE 0.0 END), 0)::float4 AS mastery \
         FROM blocks b LEFT JOIN flashcards f ON f.block_id = b.id \
         WHERE b.subject_id = $1 \
         GROUP BY b.id, b.title \
         ORDER BY mastery ASC, due DESC \
         LIMIT 3",
    )
    .bind(subject_id)
    .fetch_all(&s.pool)
    .await?;

    Ok(Json(SubjectStats {
        total_cards: row.total_cards,
        due_now: row.due_now,
        by_box: [row.box1, row.box2, row.box3, row.box4, row.box5],
        reviews_total,
        weakest_blocks,
    }))
}

#[derive(sqlx::FromRow)]
struct TodayRow {
    secs: i64,
    cnt: i64,
}

async fn guardrails(State(s): State<AppState>) -> AppResult<Json<Guardrails>> {
    let today = sqlx::query_as::<_, TodayRow>(
        "SELECT COALESCE(SUM(duration_s), 0)::bigint AS secs, COUNT(*)::bigint AS cnt \
         FROM study_sessions WHERE started_at >= date_trunc('day', now())",
    )
    .fetch_one(&s.pool)
    .await?;

    let dates: Vec<NaiveDate> = sqlx::query_scalar(
        "SELECT DISTINCT (started_at)::date AS d FROM study_sessions \
         WHERE started_at >= now() - interval '120 days' ORDER BY d DESC",
    )
    .fetch_all(&s.pool)
    .await?;

    let streak = compute_streak(&dates);

    let local = Local::now();
    let hour = local.hour();
    let after_22h = hour >= 22 || hour < 5;
    let rest_day_today = local.weekday() == Weekday::Sun;

    let mut nudges = Vec::new();
    if after_22h {
        nudges.push("Il est tard — la consolidation se fait en dormant. Va dormir 😴".to_string());
    }
    if rest_day_today {
        nudges.push("Aujourd'hui c'est repos. Laisse ton cerveau consolider — ferme l'app.".to_string());
    }
    if today.secs / 60 >= 45 {
        nudges.push("Bel effort aujourd'hui. Pense aux pauses : le rythme bat le marathon.".to_string());
    }

    Ok(Json(Guardrails {
        today_minutes: today.secs / 60,
        streak_days: streak,
        rest_day_today,
        after_22h,
        sessions_today: today.cnt,
        nudges,
    }))
}

/// Count consecutive days with at least one session, ending today (or yesterday
/// if nothing yet today).
fn compute_streak(dates: &[NaiveDate]) -> i64 {
    use std::collections::HashSet;
    let set: HashSet<NaiveDate> = dates.iter().copied().collect();
    let today = Local::now().date_naive();
    let mut day = today;
    if !set.contains(&day) {
        // allow the streak to end yesterday
        match day.pred_opt() {
            Some(y) => day = y,
            None => return 0,
        }
    }
    let mut streak = 0i64;
    while set.contains(&day) {
        streak += 1;
        match day.pred_opt() {
            Some(p) => day = p,
            None => break,
        }
    }
    streak
}
