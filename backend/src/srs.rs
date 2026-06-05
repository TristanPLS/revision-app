//! FSRS-5 spaced-repetition scheduler, implemented directly (no `burn`/`fsrs`
//! dependency). Uses the published FSRS-5 default weights. Parameter
//! *optimization* (which would require a training framework) is deferred; this
//! module only does scheduling, which is the part the review loop needs.
//!
//! References: Open Spaced Repetition — FSRS-5 algorithm specification.

use crate::models::CardState;

const DECAY: f32 = -0.5;
/// FACTOR = 0.9^(1/DECAY) - 1 = 19/81.
const FACTOR: f32 = 19.0 / 81.0;

/// FSRS-5 default parameters (w0..w18).
const W: [f32; 19] = [
    0.40255, 1.18385, 3.173, 15.69105, 7.1949, 0.5345, 1.4604, 0.0046, 1.54575, 0.1192, 1.01925,
    1.9395, 0.11, 0.29605, 2.2698, 0.2315, 2.9898, 0.51655, 0.6621,
];

#[derive(Debug, Clone, Copy)]
pub struct MemoryState {
    pub stability: f32,
    pub difficulty: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct ScheduleOutcome {
    pub stability: f32,
    pub difficulty: f32,
    pub state: CardState,
    pub scheduled_days: i32,
    pub leitner_box: u8,
    /// true when a Review card was forgotten (rating Again) — caller bumps `lapses`.
    pub lapsed: bool,
}

pub struct Fsrs {
    retention: f32,
}

impl Fsrs {
    pub fn new(retention: f32) -> Self {
        Self {
            retention: retention.clamp(0.7, 0.97),
        }
    }

    /// Compute the next memory state and interval given the previous state
    /// (`None` on the very first review), the days since last review, and the
    /// rating (1=Again, 2=Hard, 3=Good, 4=Easy).
    pub fn schedule(
        &self,
        prev: Option<MemoryState>,
        prev_state: CardState,
        elapsed_days: i64,
        rating: i16,
    ) -> ScheduleOutcome {
        let g = rating.clamp(1, 4) as i32;

        let (stability, difficulty) = match prev {
            None => (init_stability(g), init_difficulty(g)),
            Some(m) => {
                let t = elapsed_days.max(0) as f32;
                let r = retrievability(t, m.stability);
                let d = next_difficulty(m.difficulty, g);
                // Stability formulas use the *previous* difficulty (FSRS spec).
                let s = if g == 1 {
                    next_stability_forget(m.difficulty, m.stability, r)
                } else {
                    next_stability_recall(m.difficulty, m.stability, r, g)
                };
                (s, d)
            }
        };

        let stability = stability.clamp(0.01, 36500.0);
        let difficulty = difficulty.clamp(1.0, 10.0);
        let scheduled_days = next_interval(stability, self.retention);
        let state = next_state(prev_state, g);
        let lapsed = g == 1 && matches!(prev_state, CardState::Review);

        ScheduleOutcome {
            stability,
            difficulty,
            state,
            scheduled_days,
            leitner_box: leitner_box(stability, state),
            lapsed,
        }
    }
}

fn init_stability(g: i32) -> f32 {
    W[(g - 1) as usize].max(0.1)
}

fn init_difficulty(g: i32) -> f32 {
    (W[4] - (W[5] * (g as f32 - 1.0)).exp() + 1.0).clamp(1.0, 10.0)
}

fn retrievability(t: f32, s: f32) -> f32 {
    (1.0 + FACTOR * t / s).powf(DECAY)
}

fn next_interval(stability: f32, retention: f32) -> i32 {
    let ivl = (stability / FACTOR) * (retention.powf(1.0 / DECAY) - 1.0);
    (ivl.round() as i32).max(1)
}

fn linear_damping(delta_d: f32, d: f32) -> f32 {
    delta_d * (10.0 - d) / 9.0
}

fn next_difficulty(d: f32, g: i32) -> f32 {
    let delta = -W[6] * (g as f32 - 3.0);
    let d2 = d + linear_damping(delta, d);
    // mean reversion toward the "easy" initial difficulty
    (W[7] * init_difficulty(4) + (1.0 - W[7]) * d2).clamp(1.0, 10.0)
}

fn next_stability_recall(d: f32, s: f32, r: f32, g: i32) -> f32 {
    let hard_penalty = if g == 2 { W[15] } else { 1.0 };
    let easy_bonus = if g == 4 { W[16] } else { 1.0 };
    s * (1.0
        + W[8].exp()
            * (11.0 - d)
            * s.powf(-W[9])
            * ((W[10] * (1.0 - r)).exp() - 1.0)
            * hard_penalty
            * easy_bonus)
}

fn next_stability_forget(d: f32, s: f32, r: f32) -> f32 {
    let new_s = W[11] * d.powf(-W[12]) * ((s + 1.0).powf(W[13]) - 1.0) * (W[14] * (1.0 - r)).exp();
    // a lapse must not increase stability
    new_s.min(s)
}

fn next_state(prev: CardState, g: i32) -> CardState {
    match prev {
        CardState::New => {
            if g == 1 {
                CardState::Learning
            } else {
                CardState::Review
            }
        }
        CardState::Learning | CardState::Relearning => {
            if g == 1 {
                prev
            } else {
                CardState::Review
            }
        }
        CardState::Review => {
            if g == 1 {
                CardState::Relearning
            } else {
                CardState::Review
            }
        }
    }
}

/// Derive a 1–5 Leitner box from stability (≈ interval at retention 0.9), for
/// display only. New/learning/relearning cards live in box 1.
fn leitner_box(stability: f32, state: CardState) -> u8 {
    if matches!(
        state,
        CardState::New | CardState::Learning | CardState::Relearning
    ) {
        return 1;
    }
    match stability {
        s if s < 1.0 => 1,
        s if s < 4.0 => 2,
        s if s < 10.0 => 3,
        s if s < 30.0 => 4,
        _ => 5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_review_intervals_increase_with_rating() {
        let f = Fsrs::new(0.9);
        let again = f.schedule(None, CardState::New, 0, 1).scheduled_days;
        let hard = f.schedule(None, CardState::New, 0, 2).scheduled_days;
        let good = f.schedule(None, CardState::New, 0, 3).scheduled_days;
        let easy = f.schedule(None, CardState::New, 0, 4).scheduled_days;
        assert!(
            again <= hard && hard <= good && good <= easy,
            "expected monotonic intervals, got {again} {hard} {good} {easy}"
        );
        assert!(
            easy >= 7,
            "Easy on a new card should schedule at least a week, got {easy}"
        );
    }

    #[test]
    fn recall_grows_stability_and_stays_review() {
        let f = Fsrs::new(0.9);
        let m = MemoryState {
            stability: 5.0,
            difficulty: 5.0,
        };
        let out = f.schedule(Some(m), CardState::Review, 5, 3);
        assert!(out.stability > 5.0, "Good recall should grow stability");
        assert!(matches!(out.state, CardState::Review));
        assert!(!out.lapsed);
    }

    #[test]
    fn lapse_marks_relearning_and_shrinks() {
        let f = Fsrs::new(0.9);
        let m = MemoryState {
            stability: 30.0,
            difficulty: 5.0,
        };
        let out = f.schedule(Some(m), CardState::Review, 30, 1);
        assert!(out.lapsed);
        assert!(matches!(out.state, CardState::Relearning));
        assert!(out.stability <= 30.0);
        assert_eq!(out.leitner_box, 1);
    }

    #[test]
    fn easy_beats_good_on_review() {
        let f = Fsrs::new(0.9);
        let m = MemoryState {
            stability: 10.0,
            difficulty: 5.0,
        };
        let good = f.schedule(Some(m), CardState::Review, 10, 3).scheduled_days;
        let easy = f.schedule(Some(m), CardState::Review, 10, 4).scheduled_days;
        assert!(easy >= good);
    }
}
