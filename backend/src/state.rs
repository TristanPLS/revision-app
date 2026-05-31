use crate::{ai::AiClient, config::Config};
use sqlx::PgPool;

/// Shared application state. Clone-cheap (`PgPool` and `reqwest::Client` are
/// internally reference-counted).
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub ai: AiClient,
    pub cfg: Config,
}
