//! AI generation: Google AI Studio (Gemini REST endpoint serving Gemma),
//! structured JSON output, and the per-artifact generation pipeline.

pub mod client;
pub mod generate;
pub mod prompts;
pub mod schemas;

pub use client::AiClient;

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
    let t = t.strip_prefix("```json").or_else(|| t.strip_prefix("```")).unwrap_or(t);
    let t = t.strip_suffix("```").unwrap_or(t);
    t.trim().to_string()
}
