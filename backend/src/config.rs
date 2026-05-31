use std::env;

/// Runtime configuration, read once from the environment at startup.
#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub backend_port: u16,
    /// `None` when no real key is configured — AI endpoints then return 503.
    pub gemini_api_key: Option<String>,
    pub gemini_base_url: String,
    pub ai_model: String,
    pub fsrs_retention: f32,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: env::var("DATABASE_URL").unwrap_or_else(|_| {
                "postgres://revision:change-me-long-random@localhost:5432/revision".into()
            }),
            backend_port: env::var("BACKEND_PORT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(8080),
            gemini_api_key: env::var("GEMINI_API_KEY")
                .ok()
                .filter(|s| !s.trim().is_empty() && s != "PUT-YOUR-AI-STUDIO-KEY-HERE"),
            gemini_base_url: env::var("GEMINI_BASE_URL")
                .unwrap_or_else(|_| "https://generativelanguage.googleapis.com/v1beta".into()),
            ai_model: env::var("AI_MODEL").unwrap_or_else(|_| "gemma-3-27b-it".into()),
            fsrs_retention: env::var("FSRS_RETENTION")
                .ok()
                .and_then(|s| s.parse().ok())
                .map(|r: f32| r.clamp(0.7, 0.97))
                .unwrap_or(0.9),
        }
    }
}
