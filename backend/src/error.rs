use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// Application error. User-facing messages are generic French strings; full
/// detail goes to `tracing` only (never leaked to the client).
#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("requête invalide: {0}")]
    BadRequest(String),
    #[error("introuvable")]
    NotFound,
    #[error("validation: {0}")]
    Validation(String),
    #[error("conflit: {0}")]
    Conflict(String),
    #[error("erreur base de données")]
    Database(#[from] sqlx::Error),
    #[error("IA non configurée (GEMINI_API_KEY absente)")]
    AiNotConfigured,
    #[error("fournisseur IA {0}: {1}")]
    AiProvider(u16, String),
    #[error("réponse IA vide")]
    AiEmpty,
    #[error("schéma IA invalide: {0}")]
    AiSchema(String),
    #[error("erreur HTTP: {0}")]
    Http(#[from] reqwest::Error),
    #[error("erreur interne")]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, msg) = match &self {
            AppError::BadRequest(m) | AppError::Validation(m) => {
                (StatusCode::BAD_REQUEST, m.clone())
            }
            AppError::NotFound | AppError::Database(sqlx::Error::RowNotFound) => {
                (StatusCode::NOT_FOUND, "introuvable".to_string())
            }
            AppError::Conflict(m) => (StatusCode::CONFLICT, m.clone()),
            AppError::AiNotConfigured => (StatusCode::SERVICE_UNAVAILABLE, self.to_string()),
            AppError::AiProvider(..) | AppError::AiEmpty | AppError::AiSchema(_) => {
                (StatusCode::BAD_GATEWAY, "échec de génération IA".to_string())
            }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "erreur interne".to_string()),
        };

        if status.is_server_error() || status == StatusCode::BAD_GATEWAY {
            tracing::error!(error = %self, "request failed");
        }

        (status, Json(json!({ "error": msg }))).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
