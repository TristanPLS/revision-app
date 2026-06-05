use std::sync::{Arc, RwLock};
use std::time::Duration;

use serde_json::{json, Value};

use super::{strip_fence, truncate};
use crate::{
    config::Config,
    error::{AppError, AppResult},
};

/// Fournisseur IA supporté. `OpenAi` désigne le format "OpenAI-compatible"
/// (chat/completions) qui couvre OpenAI, Ollama, LM Studio, Groq, Mistral,
/// vLLM… `Anthropic` utilise l'API Messages native.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AiProvider {
    Gemini,
    OpenAi,
    Anthropic,
}

impl AiProvider {
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "gemini" | "google" => Some(Self::Gemini),
            "openai" | "openai-compatible" | "ollama" | "lmstudio" | "groq" => Some(Self::OpenAi),
            "anthropic" | "claude" => Some(Self::Anthropic),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Gemini => "gemini",
            Self::OpenAi => "openai",
            Self::Anthropic => "anthropic",
        }
    }

    pub fn default_base_url(&self) -> &'static str {
        match self {
            Self::Gemini => "https://generativelanguage.googleapis.com/v1beta",
            Self::OpenAi => "https://api.openai.com/v1",
            Self::Anthropic => "https://api.anthropic.com",
        }
    }

    /// Modèle par défaut. `None` pour OpenAI-compatible : il n'existe aucun
    /// défaut raisonnable commun à OpenAI, Ollama, Groq… — l'utilisateur doit
    /// le renseigner.
    pub fn default_model(&self) -> Option<&'static str> {
        match self {
            Self::Gemini => Some("gemma-3-27b-it"),
            Self::OpenAi => None,
            Self::Anthropic => Some("claude-opus-4-8"),
        }
    }

    /// Gemini et Anthropic exigent une clé ; les serveurs OpenAI-compatibles
    /// locaux (Ollama, LM Studio) n'en demandent pas.
    pub fn requires_key(&self) -> bool {
        matches!(self, Self::Gemini | Self::Anthropic)
    }
}

/// Réglages IA effectifs (provider + modèle + URL + clé), rechargeables à
/// chaud via la page Réglages.
#[derive(Clone, Debug)]
pub struct AiSettings {
    pub provider: AiProvider,
    pub model: String,
    pub base_url: String,
    pub api_key: Option<String>,
}

impl AiSettings {
    /// Réglages initiaux dérivés de l'environnement (fallback quand aucune
    /// ligne `app_settings` n'existe encore).
    pub fn from_env(cfg: &Config) -> Self {
        let provider = AiProvider::parse(&cfg.ai_provider).unwrap_or(AiProvider::Gemini);
        Self {
            provider,
            model: cfg.ai_model.clone(),
            base_url: cfg.gemini_base_url.trim_end_matches('/').to_string(),
            api_key: cfg.gemini_api_key.clone(),
        }
    }

    pub fn is_configured(&self) -> bool {
        let has_key = self.api_key.is_some();
        let has_model = !self.model.trim().is_empty();
        if self.provider.requires_key() {
            has_key && has_model
        } else {
            has_model
        }
    }
}

/// Client IA multi-provider. Les réglages vivent derrière un `RwLock` partagé :
/// un PUT /api/settings met à jour la clé/le modèle sans redémarrage, y compris
/// pour les jobs de génération déjà en file (ils tiennent un clone du client).
#[derive(Clone)]
pub struct AiClient {
    http: reqwest::Client,
    settings: Arc<RwLock<AiSettings>>,
}

impl AiClient {
    pub fn new(settings: AiSettings) -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(180))
            .connect_timeout(Duration::from_secs(15))
            .build()
            .unwrap_or_default();
        Self {
            http,
            settings: Arc::new(RwLock::new(settings)),
        }
    }

    /// Instantané des réglages courants.
    pub fn snapshot(&self) -> AiSettings {
        self.settings
            .read()
            .expect("settings lock poisoned")
            .clone()
    }

    /// Remplace les réglages à chaud.
    pub fn update(&self, new: AiSettings) {
        *self.settings.write().expect("settings lock poisoned") = new;
    }

    pub fn model(&self) -> String {
        self.snapshot().model
    }

    pub fn is_configured(&self) -> bool {
        self.snapshot().is_configured()
    }

    /// Generate constrained JSON. Returns the raw JSON text payload (a string
    /// that should parse against `schema`).
    ///
    /// Resilience: si le provider rejette la sortie structurée (400), on
    /// réessaie en mode JSON libre (les prompts exigent déjà du JSON pur) ;
    /// un 429/5xx transitoire est réessayé 2 fois avec backoff — utile sur le
    /// tier gratuit Gemini pendant un bundle "Tout générer".
    pub async fn generate_json(&self, prompt: &str, schema: Value) -> AppResult<String> {
        let settings = self.snapshot();
        let mut with_schema = true;
        let mut transient_retries: usize = 0;
        loop {
            let schema_arg = if with_schema { Some(&schema) } else { None };
            match self.call(&settings, prompt, schema_arg).await {
                Err(AppError::AiProvider(400, msg)) if with_schema => {
                    tracing::warn!(provider = settings.provider.as_str(), %msg,
                        "structured output rejected (400); retrying in plain JSON mode");
                    with_schema = false;
                }
                Err(AppError::AiProvider(code, msg))
                    if (code == 429 || code >= 500) && transient_retries < 2 =>
                {
                    let delay = [2u64, 6][transient_retries];
                    transient_retries += 1;
                    tracing::warn!(code, %msg, attempt = transient_retries,
                        "transient AI provider error; retrying in {delay}s");
                    tokio::time::sleep(Duration::from_secs(delay)).await;
                }
                other => return other,
            }
        }
    }

    /// Appel de validation pour la page Réglages : une mini-génération JSON,
    /// sans retry transitoire (feedback rapide). Renvoie le nom du modèle.
    pub async fn test_connection(&self) -> AppResult<String> {
        let settings = self.snapshot();
        let prompt = "Réponds UNIQUEMENT avec ce JSON exact : {\"ok\": true}";
        let schema = json!({
            "type": "object",
            "properties": { "ok": { "type": "boolean" } },
            "required": ["ok"]
        });
        match self.call(&settings, prompt, Some(&schema)).await {
            Err(AppError::AiProvider(400, _)) => self.call(&settings, prompt, None).await?,
            other => other?,
        };
        Ok(settings.model)
    }

    async fn call(
        &self,
        s: &AiSettings,
        prompt: &str,
        schema: Option<&Value>,
    ) -> AppResult<String> {
        if !s.is_configured() {
            return Err(AppError::AiNotConfigured);
        }
        match s.provider {
            AiProvider::Gemini => self.call_gemini(s, prompt, schema).await,
            AiProvider::OpenAi => self.call_openai(s, prompt, schema).await,
            AiProvider::Anthropic => self.call_anthropic(s, prompt, schema).await,
        }
    }

    /// Gemini `generateContent`. La clé passe par le header `x-goog-api-key`
    /// (jamais en query param : une erreur reqwest affiche l'URL complète dans
    /// les logs, ce qui ferait fuiter la clé).
    async fn call_gemini(
        &self,
        s: &AiSettings,
        prompt: &str,
        schema: Option<&Value>,
    ) -> AppResult<String> {
        let key = s.api_key.as_ref().ok_or(AppError::AiNotConfigured)?;
        let url = format!("{}/models/{}:generateContent", s.base_url, s.model);

        let mut generation_config = json!({
            "responseMimeType": "application/json",
            "temperature": 0.3
        });
        if let Some(sc) = schema {
            generation_config["responseSchema"] = sc.clone();
        }
        let body = json!({
            "contents": [{ "parts": [{ "text": prompt }] }],
            "generationConfig": generation_config
        });

        let resp = self
            .http
            .post(&url)
            .header("x-goog-api-key", key)
            .json(&body)
            .send()
            .await?;
        let status = resp.status();
        let text = resp.text().await?;
        if !status.is_success() {
            return Err(AppError::AiProvider(status.as_u16(), truncate(&text, 500)));
        }

        let v: Value = serde_json::from_str(&text)
            .map_err(|e| AppError::AiSchema(format!("réponse non-JSON: {e}")))?;
        match v["candidates"][0]["content"]["parts"][0]["text"].as_str() {
            Some(out) if !out.trim().is_empty() => Ok(strip_fence(out)),
            _ => {
                tracing::debug!(raw = %truncate(&text, 800), "empty AI candidate");
                Err(AppError::AiEmpty)
            }
        }
    }

    /// Format OpenAI-compatible `chat/completions` (OpenAI, Ollama, LM Studio,
    /// Groq, Mistral, vLLM…). Avec schéma : `response_format: json_schema` ;
    /// le fallback sans schéma omet `response_format` (compat maximale — tous
    /// les prompts exigent déjà du JSON pur). Pas de `temperature` : certains
    /// modèles récents la rejettent.
    async fn call_openai(
        &self,
        s: &AiSettings,
        prompt: &str,
        schema: Option<&Value>,
    ) -> AppResult<String> {
        let url = format!("{}/chat/completions", s.base_url);

        let mut body = json!({
            "model": s.model,
            "messages": [{ "role": "user", "content": prompt }]
        });
        if let Some(sc) = schema {
            body["response_format"] = json!({
                "type": "json_schema",
                "json_schema": { "name": "output", "schema": sc }
            });
        }

        let mut req = self.http.post(&url).json(&body);
        if let Some(key) = &s.api_key {
            req = req.header("Authorization", format!("Bearer {key}"));
        }
        let resp = req.send().await?;
        let status = resp.status();
        let text = resp.text().await?;
        if !status.is_success() {
            return Err(AppError::AiProvider(status.as_u16(), truncate(&text, 500)));
        }

        let v: Value = serde_json::from_str(&text)
            .map_err(|e| AppError::AiSchema(format!("réponse non-JSON: {e}")))?;
        match v["choices"][0]["message"]["content"].as_str() {
            Some(out) if !out.trim().is_empty() => Ok(strip_fence(out)),
            _ => {
                tracing::debug!(raw = %truncate(&text, 800), "empty AI candidate");
                Err(AppError::AiEmpty)
            }
        }
    }

    /// API Anthropic Messages. Sortie structurée via tool use forcé
    /// (`tool_choice` sur un outil dont l'`input_schema` est notre schéma) —
    /// plus tolérant que `output_config.format`, qui exige
    /// `additionalProperties: false` partout. Pas de `temperature` (rejetée
    /// par Opus 4.7+).
    async fn call_anthropic(
        &self,
        s: &AiSettings,
        prompt: &str,
        schema: Option<&Value>,
    ) -> AppResult<String> {
        let key = s.api_key.as_ref().ok_or(AppError::AiNotConfigured)?;
        let url = format!("{}/v1/messages", s.base_url);

        let mut body = json!({
            "model": s.model,
            "max_tokens": 16000,
            "messages": [{ "role": "user", "content": prompt }]
        });
        if let Some(sc) = schema {
            body["tools"] = json!([{
                "name": "emit",
                "description": "Retourne le résultat structuré demandé.",
                "input_schema": sc
            }]);
            body["tool_choice"] = json!({ "type": "tool", "name": "emit" });
        }

        let resp = self
            .http
            .post(&url)
            .header("x-api-key", key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await?;
        let status = resp.status();
        let text = resp.text().await?;
        if !status.is_success() {
            return Err(AppError::AiProvider(status.as_u16(), truncate(&text, 500)));
        }

        let v: Value = serde_json::from_str(&text)
            .map_err(|e| AppError::AiSchema(format!("réponse non-JSON: {e}")))?;
        let blocks = v["content"].as_array().cloned().unwrap_or_default();
        // Tool use forcé : le JSON structuré est l'input de l'outil.
        for block in &blocks {
            if block["type"] == "tool_use" {
                return serde_json::to_string(&block["input"])
                    .map_err(|e| AppError::AiSchema(e.to_string()));
            }
        }
        // Mode texte (fallback sans schéma).
        for block in &blocks {
            if block["type"] == "text" {
                if let Some(out) = block["text"].as_str() {
                    if !out.trim().is_empty() {
                        return Ok(strip_fence(out));
                    }
                }
            }
        }
        tracing::debug!(raw = %truncate(&text, 800), "empty AI candidate");
        Err(AppError::AiEmpty)
    }
}
