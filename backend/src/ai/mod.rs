//! AI generation: multi-provider client (Gemini / OpenAI-compatible /
//! Anthropic), structured JSON output, and the per-artifact generation
//! pipeline.

pub mod client;
pub mod generate;
pub mod prompts;
pub mod schemas;

use std::sync::OnceLock;

pub use client::{AiClient, AiProvider, AiSettings};

/// Max chars of course text injected into prompts. Set once at boot from
/// `AI_MAX_SOURCE_CHARS` (default 16 000 — sized for Gemma; a 128k+ model can
/// absorb much more, a small local model less).
static MAX_SOURCE_CHARS: OnceLock<usize> = OnceLock::new();

pub fn set_max_source_chars(n: usize) {
    let _ = MAX_SOURCE_CHARS.set(n.max(1_000));
}

pub fn max_source_chars() -> usize {
    *MAX_SOURCE_CHARS.get_or_init(|| 16_000)
}

/// Truncate a string to at most `n` chars (char-boundary safe).
pub fn truncate(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(n).collect();
        out.push_str(" …[tronqué]");
        out
    }
}

/// Strip a leading/trailing ```json … ``` fence if the model wrapped its output.
pub fn strip_fence(s: &str) -> String {
    let t = s.trim();
    let t = t
        .strip_prefix("```json")
        .or_else(|| t.strip_prefix("```"))
        .unwrap_or(t);
    let t = t.strip_suffix("```").unwrap_or(t);
    t.trim().to_string()
}
