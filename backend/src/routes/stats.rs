use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use chrono::{Datelike, Local, NaiveDate, Timelike, Weekday};
use uuid::Uuid;

use crate::{
    error::AppResult,
    models::{BlockMastery, FsrsInsights, Guardrails, SubjectStats},
    state::AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/subjects/{id}/stats", get(subject_stats))
        .route("/subjects/{id}/fsrs-insights", get(fsrs_insights))
        .route("/guardrails", get(guardrails))
}

// FSRS forgetting-curve constants (match srs.rs).
const FACTOR: f32 = 19.0 / 81.0;
const DECAY: f32 = -0.5;

#[derive(sqlx::FromRow)]
struct ReviewRow {
    flashcard_id: Uuid,
    rating: i16,
    elapsed_days: i32,
    stability_after: Option<f32>,
    scheduled_days: i32,
}

/// Data-driven FSRS insights from the real review log: measured retention,
/// model calibration, rating distribution, and a retention recommendation.
/// (Full 19-weight training is deferred — it needs a training framework and
/// far more review history.)
async fn fsrs_insights(
    State(s): State<AppState>,
    Path(subject_id): Path<Uuid>,
) -> AppResult<Json<FsrsInsights>> {
    let rows = sqlx::query_as::<_, ReviewRow>(
        "SELECT r.flashcard_id, r.rating, r.elapsed_days, r.stability_after, r.scheduled_days \
         FROM reviews r JOIN flashcards f ON f.id = r.flashcard_id \
         WHERE f.subject_id = $1 ORDER BY r.flashcard_id, r.reviewed_at",
    )
    .bind(subject_id)
    .fetch_all(&s.pool)
    .await?;

    let reviews_total = rows.len() as i64;
    let mut rating_counts = [0i64; 4];
    let mut intervals: Vec<i32> = Vec::new();
    let mut cards = std::collections::HashSet::new();

    let mut prev_card: Option<Uuid> = None;
    let mut prev_stab: Option<f32> = None;
    let mut mature = 0i64;
    let mut recalled = 0i64;
    let mut pred_sum = 0.0f64;
    let mut pred_n = 0i64;

    for r in &rows {
        cards.insert(r.flashcard_id);
        if (1..=4).contains(&r.rating) {
            rating_counts[(r.rating - 1) as usize] += 1;
        }
        intervals.push(r.scheduled_days);

        let same_card = prev_card == Some(r.flashcard_id);
        if same_card && r.elapsed_days >= 1 {
            mature += 1;
            if r.rating >= 2 {
                recalled += 1;
            }
            if let Some(sb) = prev_stab {
                if sb > 0.0 {
                    let rr = (1.0 + FACTOR * r.elapsed_days as f32 / sb).powf(DECAY);
                    pred_sum += rr as f64;
                    pred_n += 1;
                }
            }
        }
        prev_card = Some(r.flashcard_id);
        prev_stab = r.stability_after;
    }

    let measured = if mature > 0 {
        Some(recalled as f32 / mature as f32)
    } else {
        None
    };
    let predicted = if pred_n > 0 {
        Some((pred_sum / pred_n as f64) as f32)
    } else {
        None
    };
    intervals.sort_unstable();
    let median_interval_days = intervals.get(intervals.len() / 2).copied();
    let target = s.cfg.fsrs_retention;

    Ok(Json(FsrsInsights {
        reviews_total,
        cards_reviewed: cards.len() as i64,
        measured_retention: measured,
        predicted_retention: predicted,
        rating_counts,
        median_interval_days,
        target_retention: target,
        recommendation: build_reco(reviews_total, measured, target),
    }))
}

fn build_reco(total: i64, measured: Option<f32>, target: f32) -> String {
    if total < 30 {
        return format!(
            "Encore peu de révisions ({total}). Vise ~100 révisions pour une optimisation fiable des paramètres."
        );
    }
    match measured {
        None => "Pas encore assez de révisions espacées pour mesurer ta rétention.".to_string(),
        Some(m) => {
            let mp = (m * 100.0).round();
            let tp = (target * 100.0).round();
            if m > target + 0.05 {
                format!(
                    "Rétention mesurée {mp}% > cible {tp}%. Tu retiens mieux que prévu : tu peux allonger les intervalles (baisser FSRS_RETENTION vers ~{:.2}).",
                    (m - 0.03).max(0.80)
                )
            } else if m < target - 0.05 {
                format!(
                    "Rétention mesurée {mp}% < cible {tp}%. Tu oublies plus que prévu : raccourcis les intervalles (monte FSRS_RETENTION vers ~{:.2}) ou révise plus régulièrement.",
                    (target + 0.03).min(0.95)
                )
            } else {
                format!("Bien calibré : rétention mesurée {mp}% ≈ cible {tp}%.")
            }
        }
    }
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
    let after_22h = !(5..22).contains(&hour);
    let rest_day_today = local.weekday() == Weekday::Sun;

    let mut nudges = Vec::new();
    if after_22h {
        nudges.push("Il est tard — la consolidation se fait en dormant. Va dormir 😴".to_string());
    }
    if rest_day_today {
        nudges.push(
            "Aujourd'hui c'est repos. Laisse ton cerveau consolider — ferme l'app.".to_string(),
        );
    }
    if today.secs / 60 >= 45 {
        nudges.push(
            "Bel effort aujourd'hui. Pense aux pauses : le rythme bat le marathon.".to_string(),
        );
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
