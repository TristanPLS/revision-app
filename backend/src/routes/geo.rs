use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    geo_answer,
    models::{
        CardState, GeoAnswerRequest, GeoAnswerResponse, GeoCountry, GeoKind, GeoQueueItem, GeoStats,
    },
    srs::{Fsrs, MemoryState},
    state::AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/geo/countries", get(countries))
        .route("/geo/queue", get(queue))
        .route("/geo/cards/{id}/answer", post(answer))
        .route("/geo/stats", get(stats))
}

/// `kind` arrives as a raw query-string value; axum's enum rejection would not
/// speak French, so parse it by hand.
fn parse_kind(raw: &str) -> Result<GeoKind, AppError> {
    match raw {
        "flag" => Ok(GeoKind::Flag),
        "capital" => Ok(GeoKind::Capital),
        _ => Err(AppError::BadRequest(
            "kind doit valoir 'flag' ou 'capital'".into(),
        )),
    }
}

#[derive(Deserialize)]
struct CountriesParams {
    continent: Option<String>,
}

/// No `capital_fr` here: a queue item carries `iso2`, so exposing capitals for
/// the whole referential would hand the client the expected answer before it
/// answers. Capitals are revealed one at a time, by `answer`.
const COUNTRY_COLS: &str = "iso2, name_fr, continent";

async fn countries(
    State(s): State<AppState>,
    Query(q): Query<CountriesParams>,
) -> AppResult<Json<Vec<GeoCountry>>> {
    let rows = match q.continent {
        Some(c) => {
            sqlx::query_as::<_, GeoCountry>(&format!(
                "SELECT {COUNTRY_COLS} FROM geo_countries WHERE continent = $1 ORDER BY name_fr"
            ))
            .bind(c)
            .fetch_all(&s.pool)
            .await?
        }
        None => {
            sqlx::query_as::<_, GeoCountry>(&format!(
                "SELECT {COUNTRY_COLS} FROM geo_countries ORDER BY name_fr"
            ))
            .fetch_all(&s.pool)
            .await?
        }
    };
    Ok(Json(rows))
}

#[derive(Deserialize)]
struct GeoQueueParams {
    kind: Option<String>,
    continent: Option<String>,
    limit: Option<i64>,
}

#[derive(sqlx::FromRow)]
struct FlagQueueRow {
    card_id: Uuid,
    iso2: String,
    continent: String,
    state: CardState,
    due: DateTime<Utc>,
    reps: i32,
    options: Vec<String>,
}

#[derive(sqlx::FromRow)]
struct CapitalQueueRow {
    card_id: Uuid,
    iso2: String,
    country_name: String,
    continent: String,
    state: CardState,
    due: DateTime<Utc>,
    reps: i32,
}

/// Cards ordered by due date (due ones first). Unlike the flashcard queue this
/// does *not* filter on `due <= now()`: geography is also played freely, and an
/// empty queue would be a dead end on a 197-country deck.
/// The payload never contains the expected answer — see `GeoQueueItem`.
async fn queue(
    State(s): State<AppState>,
    Query(q): Query<GeoQueueParams>,
) -> AppResult<Json<Vec<GeoQueueItem>>> {
    let kind = match q.kind.as_deref() {
        Some(k) => parse_kind(k)?,
        None => {
            return Err(AppError::BadRequest(
                "le paramètre kind est requis ('flag' ou 'capital')".into(),
            ))
        }
    };
    let limit = q.limit.unwrap_or(20).clamp(1, 50);
    let continent_clause = match q.continent {
        Some(_) => "AND co.continent = $3",
        None => "",
    };

    let items: Vec<GeoQueueItem> = match kind {
        GeoKind::Flag => {
            // The right name hides among 3 random distractors — same continent
            // first when available — and the 4 options are shuffled in SQL, so
            // the client cannot tell which one is expected.
            let sql = format!(
                "SELECT c.id AS card_id, co.iso2, co.continent, c.state, c.due, c.reps, \
                   ARRAY(SELECT n FROM unnest(array_append(d.distractors, co.name_fr)) AS n \
                         ORDER BY random()) AS options \
                 FROM geo_cards c \
                 JOIN geo_countries co ON co.iso2 = c.iso2 \
                 CROSS JOIN LATERAL (\
                   SELECT ARRAY(\
                     SELECT o.name_fr FROM geo_countries o \
                     WHERE o.iso2 <> co.iso2 \
                     ORDER BY (o.continent = co.continent) DESC, random() LIMIT 3\
                   ) AS distractors\
                 ) d \
                 WHERE c.kind = $1 {continent_clause} \
                 ORDER BY c.due ASC LIMIT $2"
            );
            let mut query = sqlx::query_as::<_, FlagQueueRow>(&sql)
                .bind(kind)
                .bind(limit);
            if let Some(c) = q.continent {
                query = query.bind(c);
            }
            query
                .fetch_all(&s.pool)
                .await?
                .into_iter()
                .map(|r| GeoQueueItem::Flag {
                    card_id: r.card_id,
                    iso2: r.iso2,
                    options: r.options,
                    continent: r.continent,
                    state: r.state,
                    due: r.due,
                    reps: r.reps,
                })
                .collect()
        }
        GeoKind::Capital => {
            let sql = format!(
                "SELECT c.id AS card_id, co.iso2, co.name_fr AS country_name, co.continent, \
                   c.state, c.due, c.reps \
                 FROM geo_cards c \
                 JOIN geo_countries co ON co.iso2 = c.iso2 \
                 WHERE c.kind = $1 {continent_clause} \
                 ORDER BY c.due ASC LIMIT $2"
            );
            let mut query = sqlx::query_as::<_, CapitalQueueRow>(&sql)
                .bind(kind)
                .bind(limit);
            if let Some(c) = q.continent {
                query = query.bind(c);
            }
            query
                .fetch_all(&s.pool)
                .await?
                .into_iter()
                .map(|r| GeoQueueItem::Capital {
                    card_id: r.card_id,
                    iso2: r.iso2,
                    country_name: r.country_name,
                    continent: r.continent,
                    state: r.state,
                    due: r.due,
                    reps: r.reps,
                })
                .collect()
        }
    };
    Ok(Json(items))
}

#[derive(sqlx::FromRow)]
struct GeoCardFull {
    kind: GeoKind,
    stability: Option<f32>,
    difficulty: Option<f32>,
    state: CardState,
    last_reviewed: Option<DateTime<Utc>>,
    reps: i32,
    lapses: i32,
    name_fr: String,
    name_accepted: Vec<String>,
    capital_fr: String,
    capital_accepted: Vec<String>,
}

/// Upper bound on a submitted answer: past this, it is not a country name, and
/// Levenshtein is O(n·m) against every accepted answer.
const MAX_GIVEN_CHARS: usize = 200;

/// True when the input is *exactly* a real answer of some other country — then
/// it is a genuine mistake (Kingston for Kingstown, Panama for Manama), not the
/// typo the fuzzy pass assumed. Only called when the fuzzy pass hit, so the
/// referential scan stays off the hot path.
async fn is_another_countrys_answer(
    s: &AppState,
    kind: GeoKind,
    given: &str,
    accepted: &[String],
) -> Result<bool, AppError> {
    let column = match kind {
        GeoKind::Flag => "name_accepted",
        GeoKind::Capital => "capital_accepted",
    };
    let others: Vec<String> =
        sqlx::query_scalar(&format!("SELECT unnest({column}) FROM geo_countries"))
            .fetch_all(&s.pool)
            .await?;
    let others: Vec<String> = others
        .into_iter()
        .filter(|o| !accepted.iter().any(|a| a == o))
        .collect();
    Ok(geo_answer::matches_exact(given, &others))
}

/// Compare the typed answer, reschedule the card with FSRS, log the answer.
async fn answer(
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<GeoAnswerRequest>,
) -> AppResult<Json<GeoAnswerResponse>> {
    // Guard on the normalized form: pure punctuation would otherwise slip
    // through and burn the card as a wrong answer.
    if geo_answer::normalize(&body.given).is_empty() {
        return Err(AppError::BadRequest(
            "ta réponse est vide — écris un nom avant de valider".into(),
        ));
    }
    if body.given.chars().count() > MAX_GIVEN_CHARS {
        return Err(AppError::BadRequest(
            "ta réponse est beaucoup trop longue".into(),
        ));
    }

    let card = sqlx::query_as::<_, GeoCardFull>(
        "SELECT c.kind, c.stability, c.difficulty, c.state, c.last_reviewed, c.reps, c.lapses, \
           co.name_fr, co.name_accepted, co.capital_fr, co.capital_accepted \
         FROM geo_cards c JOIN geo_countries co ON co.iso2 = c.iso2 WHERE c.id = $1",
    )
    .bind(id)
    .fetch_optional(&s.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let (expected, accepted) = match card.kind {
        GeoKind::Flag => (card.name_fr, card.name_accepted),
        GeoKind::Capital => (card.capital_fr, card.capital_accepted),
    };
    let correct = match card.kind {
        // Multiple choice: the answer is a click on a displayed label, so typo
        // tolerance would only serve to validate a neighbouring wrong option
        // (Islande for Irlande, Zambie for Gambie).
        GeoKind::Flag => geo_answer::matches_exact(&body.given, &accepted),
        GeoKind::Capital => {
            geo_answer::matches_exact(&body.given, &accepted)
                || (geo_answer::matches_typed(&body.given, &accepted)
                    && !is_another_countrys_answer(&s, card.kind, &body.given, &accepted).await?)
        }
    };
    // Binary input: right → Good (3), wrong → Again (1). Hard/Easy would need
    // a self-assessment the typed answer does not carry.
    let rating: i16 = if correct { 3 } else { 1 };

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
    let out = fsrs.schedule(prev, card.state, elapsed_days, rating);
    let due = now + Duration::days(out.scheduled_days as i64);
    let new_reps = card.reps + 1;
    let new_lapses = card.lapses + i32::from(out.lapsed);

    let mut tx = s.pool.begin().await?;
    sqlx::query(
        "UPDATE geo_cards SET stability=$2, difficulty=$3, state=$4, due=$5, \
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
        "INSERT INTO geo_answers (card_id, session_id, answered_at, given, correct, rating, \
         elapsed_days, stability_after, difficulty_after, scheduled_days) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
    )
    .bind(id)
    .bind(body.session_id)
    .bind(now)
    .bind(body.given)
    .bind(correct)
    .bind(rating)
    .bind(elapsed_days as i32)
    .bind(out.stability)
    .bind(out.difficulty)
    .bind(out.scheduled_days)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    let accepted_alternatives = accepted.into_iter().filter(|a| *a != expected).collect();

    Ok(Json(GeoAnswerResponse {
        correct,
        expected,
        accepted_alternatives,
        next_due: due,
        scheduled_days: out.scheduled_days,
        leitner_box: out.leitner_box,
        state: out.state,
    }))
}

#[derive(Deserialize)]
struct StatsParams {
    kind: Option<String>,
}

#[derive(sqlx::FromRow)]
struct GeoCardTally {
    total_cards: i64,
    new_cards: i64,
    mastered: i64,
    due_now: i64,
}

#[derive(sqlx::FromRow)]
struct GeoAnswerTally {
    answers_total: i64,
    answers_correct: i64,
}

// "Mastered" = Leitner boxes 4–5, same stability cut as stats.rs.
const CARD_TALLY: &str = "SELECT \
    COUNT(*) AS total_cards, \
    COUNT(*) FILTER (WHERE state = 'new') AS new_cards, \
    COUNT(*) FILTER (WHERE state = 'review' AND stability >= 10) AS mastered, \
    COUNT(*) FILTER (WHERE due <= now()) AS due_now \
  FROM geo_cards";

const ANSWER_TALLY: &str = "SELECT \
    COUNT(*) AS answers_total, \
    COUNT(*) FILTER (WHERE a.correct) AS answers_correct \
  FROM geo_answers a JOIN geo_cards c ON c.id = a.card_id";

async fn stats(
    State(s): State<AppState>,
    Query(q): Query<StatsParams>,
) -> AppResult<Json<GeoStats>> {
    let kind = q.kind.as_deref().map(parse_kind).transpose()?;

    let (cards, answers) = match kind {
        Some(k) => (
            sqlx::query_as::<_, GeoCardTally>(&format!("{CARD_TALLY} WHERE kind = $1"))
                .bind(k)
                .fetch_one(&s.pool)
                .await?,
            sqlx::query_as::<_, GeoAnswerTally>(&format!("{ANSWER_TALLY} WHERE c.kind = $1"))
                .bind(k)
                .fetch_one(&s.pool)
                .await?,
        ),
        None => (
            sqlx::query_as::<_, GeoCardTally>(CARD_TALLY)
                .fetch_one(&s.pool)
                .await?,
            sqlx::query_as::<_, GeoAnswerTally>(ANSWER_TALLY)
                .fetch_one(&s.pool)
                .await?,
        ),
    };

    Ok(Json(GeoStats {
        total_cards: cards.total_cards,
        new_cards: cards.new_cards,
        in_progress: cards.total_cards - cards.new_cards - cards.mastered,
        mastered: cards.mastered,
        due_now: cards.due_now,
        success_rate: (answers.answers_total > 0)
            .then(|| answers.answers_correct as f32 / answers.answers_total as f32),
    }))
}
