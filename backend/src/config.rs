use std::env;

/// Runtime configuration, read once from the environment at startup.
#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub backend_port: u16,
    /// `None` when no real key is configured — AI endpoints then return 503.
    /// Env fallback only: the value persisted via the settings UI wins.
    pub gemini_api_key: Option<String>,
    /// Raw env values (no provider default baked in) so `AiSettings::from_env`
    /// can apply the *correct* provider's defaults — otherwise a non-Gemini
    /// `AI_PROVIDER` set via env would inherit Gemini's base_url/model.
    pub ai_base_url: Option<String>,
    pub ai_provider: String,
    pub ai_model: Option<String>,
    pub fsrs_retention: f32,
    /// Max chars of course text injected into AI prompts.
    pub ai_max_source_chars: usize,
}

/// Read an env var, treating unset, blank and placeholder values as absent.
/// (Docker compose interpolation passes empty strings for unset variables,
/// which would otherwise shadow our defaults.)
fn env_opt(name: &str) -> Option<String> {
    env::var(name)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: env_opt("DATABASE_URL").unwrap_or_else(|| {
                // Fail fast: a silent fallback would connect to an unexpected
                // database with a well-known password baked into the binary.
                panic!(
                    "DATABASE_URL manquante. Copie .env.example vers .env et renseigne-la \
                     (voir README, section Démarrage)."
                )
            }),
            backend_port: env_opt("BACKEND_PORT")
                .and_then(|s| s.parse().ok())
                .unwrap_or(8080),
            gemini_api_key: env_opt("GEMINI_API_KEY")
                .filter(|s| s != "PUT-YOUR-AI-STUDIO-KEY-HERE"),
            // `GEMINI_BASE_URL` kept as the env name for backward compat; it now
            // applies to whichever provider is selected.
            ai_base_url: env_opt("GEMINI_BASE_URL"),
            ai_provider: env_opt("AI_PROVIDER").unwrap_or_else(|| "gemini".into()),
            ai_model: env_opt("AI_MODEL"),
            fsrs_retention: env_opt("FSRS_RETENTION")
                .and_then(|s| s.parse().ok())
                .map(|r: f32| r.clamp(0.7, 0.97))
                .unwrap_or(0.9),
            ai_max_source_chars: env_opt("AI_MAX_SOURCE_CHARS")
                .and_then(|s| s.parse().ok())
                .unwrap_or(16_000),
        }
    }
}
