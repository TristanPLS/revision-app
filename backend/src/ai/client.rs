use std::time::Duration;

use serde_json::{json, Value};

use super::{strip_fence, truncate};
use crate::{
    config::Config,
    error::{AppError, AppResult},
};

/// Thin client over the Gemini `generateContent` REST endpoint. Configurable
/// base URL + model + API key so it points at Google AI Studio (and could be
/// repointed at any compatible endpoint).
#[derive(Clone)]
pub struct AiClient {
    http: reqwest::Client,
    base_url: String,
    model: String,
    api_key: Option<String>,
}

impl AiClient {
    pub fn new(cfg: &Config) -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(180))
            .connect_timeout(Duration::from_secs(15))
            .build()
            .unwrap_or_default();
        Self {
            http,
            base_url: cfg.gemini_base_url.trim_end_matches('/').to_string(),
            model: cfg.ai_model.clone(),
            api_key: cfg.gemini_api_key.clone(),
        }
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    pub fn is_configured(&self) -> bool {
        self.api_key.is_some()
    }

    /// Generate constrained JSON. Returns the raw JSON text payload (a string
    /// that should parse against `schema`). Retries once without `responseSchema`
    /// if the provider rejects it (some Gemma models accept JSON mode but not
    /// full schemas).
    pub async fn generate_json(&self, prompt: &str, schema: Value) -> AppResult<String> {
        match self.call(prompt, Some(&schema)).await {
            Err(AppError::AiProvider(400, _)) => {
                tracing::warn!("responseSchema rejected (400); retrying with JSON mode only");
                self.call(prompt, None).await
            }
            other => other,
        }
    }

    async fn call(&self, prompt: &str, schema: Option<&Value>) -> AppResult<String> {
        let key = self.api_key.as_ref().ok_or(AppError::AiNotConfigured)?;
        let url = format!(
            "{}/models/{}:generateContent?key={}",
            self.base_url, self.model, key
        );

        let mut generation_config = json!({
            "responseMimeType": "application/json",
            "temperature": 0.3
        });
        if let Some(s) = schema {
            generation_config["responseSchema"] = s.clone();
        }
        let body = json!({
            "contents": [{ "parts": [{ "text": prompt }] }],
            "generationConfig": generation_config
        });

        let resp = self.http.post(&url).json(&body).send().await?;
        let status = resp.status();
        let text = resp.text().await?;

        if !status.is_success() {
            return Err(AppError::AiProvider(status.as_u16(), truncate(&text, 500)));
        }

        let v: Value =
            serde_json::from_str(&text).map_err(|e| AppError::AiSchema(format!("réponse non-JSON: {e}")))?;

        match v["candidates"][0]["content"]["parts"][0]["text"].as_str() {
            Some(s) if !s.trim().is_empty() => Ok(strip_fence(s)),
            _ => {
                tracing::debug!(raw = %truncate(&text, 800), "empty AI candidate");
                Err(AppError::AiEmpty)
            }
        }
    }
}
