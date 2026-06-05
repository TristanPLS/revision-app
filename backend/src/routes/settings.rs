use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::PgPool;

use crate::{
    ai::{AiProvider, AiSettings},
    config::Config,
    error::{AppError, AppResult},
    state::AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/settings", get(get_settings).put(put_settings))
        .route("/settings/ai/test", post(test_settings))
}

/// Ligne `app_settings` (singleton, colonnes nullables).
#[derive(sqlx::FromRow)]
struct SettingsRow {
    ai_provider: Option<String>,
    ai_model: Option<String>,
    ai_base_url: Option<String>,
    ai_api_key: Option<String>,
}

/// Réglages effectifs au démarrage : la ligne `app_settings` (si elle existe)
/// prime sur l'environnement ; les champs NULL retombent sur les défauts du
/// provider.
pub async fn load_initial(pool: &PgPool, cfg: &Config) -> AiSettings {
    let row = sqlx::query_as::<_, SettingsRow>(
        "SELECT ai_provider, ai_model, ai_base_url, ai_api_key FROM app_settings WHERE id = 1",
    )
    .fetch_optional(pool)
    .await
    .unwrap_or_else(|e| {
        tracing::warn!(error = %e, "could not read app_settings; falling back to env");
        None
    });

    match row {
        None => AiSettings::from_env(cfg),
        Some(r) => {
            let provider = r
                .ai_provider
                .as_deref()
                .and_then(AiProvider::parse)
                .unwrap_or(AiProvider::Gemini);
            // Réutilise la résolution partagée (défauts du bon provider).
            AiSettings::resolve(provider, r.ai_model, r.ai_base_url, r.ai_api_key)
        }
    }
}

/// Masque la clé pour l'affichage : jamais renvoyée en clair.
fn mask_key(key: &Option<String>) -> Option<String> {
    key.as_ref().map(|k| {
        if k.chars().count() > 8 {
            let tail: String = k
                .chars()
                .rev()
                .take(4)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect();
            format!("••••••••{tail}")
        } else {
            "••••••••".to_string()
        }
    })
}

fn settings_payload(s: &AiSettings) -> Value {
    json!({
        "provider": s.provider.as_str(),
        "model": s.model,
        "base_url": s.base_url,
        "api_key_set": s.api_key.is_some(),
        "api_key_hint": mask_key(&s.api_key),
        "key_required": s.provider.requires_key(),
        "configured": s.is_configured(),
        "defaults": {
            "gemini": { "base_url": AiProvider::Gemini.default_base_url(), "model": AiProvider::Gemini.default_model() },
            "openai": { "base_url": AiProvider::OpenAi.default_base_url(), "model": AiProvider::OpenAi.default_model() },
            "anthropic": { "base_url": AiProvider::Anthropic.default_base_url(), "model": AiProvider::Anthropic.default_model() },
        }
    })
}

async fn get_settings(State(s): State<AppState>) -> AppResult<Json<Value>> {
    Ok(Json(settings_payload(&s.ai.snapshot())))
}

#[derive(Deserialize)]
struct PutSettings {
    provider: Option<String>,
    model: Option<String>,
    base_url: Option<String>,
    /// Absent = inchangée ; chaîne vide = effacer la clé.
    api_key: Option<String>,
}

async fn put_settings(
    State(s): State<AppState>,
    Json(body): Json<PutSettings>,
) -> AppResult<Json<Value>> {
    let current = s.ai.snapshot();

    let provider = match &body.provider {
        Some(p) => AiProvider::parse(p).ok_or_else(|| {
            AppError::Validation("provider invalide (gemini | openai | anthropic)".into())
        })?,
        None => current.provider,
    };
    let provider_changed = provider != current.provider;

    // Modèle / URL : valeur fournie > valeur courante (sauf changement de
    // provider, où l'ancien modèle n'a plus de sens) > défaut du provider.
    let model = body
        .model
        .map(|m| m.trim().to_string())
        .filter(|m| !m.is_empty())
        .or(if provider_changed {
            None
        } else {
            Some(current.model.clone())
        });
    let base_url = body
        .base_url
        .map(|u| u.trim().trim_end_matches('/').to_string())
        .filter(|u| !u.is_empty())
        .or(if provider_changed {
            None
        } else {
            Some(current.base_url.clone())
        });
    // Clé : valeur fournie remplace ; chaîne vide efface. Si absente du body,
    // on garde la clé courante — SAUF en cas de changement de provider, où la
    // clé d'un autre fournisseur est invalide (un clé Gemini n'authentifie pas
    // Anthropic). On l'efface alors pour ne pas annoncer `configured:true` à
    // tort, et l'UI force la ressaisie.
    let api_key = match body.api_key {
        Some(k) if k.trim().is_empty() => None,
        Some(k) => Some(k.trim().to_string()),
        None if provider_changed => None,
        None => current.api_key.clone(),
    };

    let new = AiSettings::resolve(provider, model, base_url, api_key);

    // Persiste (la clé est stockée en clair — instance mono-utilisateur,
    // documenté dans le README), puis recharge le client à chaud.
    sqlx::query(
        "INSERT INTO app_settings (id, ai_provider, ai_model, ai_base_url, ai_api_key, updated_at) \
         VALUES (1, $1, $2, $3, $4, now()) \
         ON CONFLICT (id) DO UPDATE SET \
           ai_provider = $1, ai_model = $2, ai_base_url = $3, ai_api_key = $4, updated_at = now()",
    )
    .bind(new.provider.as_str())
    .bind(&new.model)
    .bind(&new.base_url)
    .bind(&new.api_key)
    .execute(&s.pool)
    .await?;

    s.ai.update(new.clone());
    tracing::info!(
        provider = new.provider.as_str(),
        model = %new.model,
        configured = new.is_configured(),
        "AI settings updated"
    );
    Ok(Json(settings_payload(&new)))
}

/// Mini-appel de validation des réglages courants (enregistrés).
/// Renvoie toujours 200 : { ok, model?, error? } — plus simple côté UI.
async fn test_settings(State(s): State<AppState>) -> AppResult<Json<Value>> {
    let started = std::time::Instant::now();
    match s.ai.test_connection().await {
        Ok(model) => Ok(Json(json!({
            "ok": true,
            "model": model,
            "latency_ms": started.elapsed().as_millis() as u64,
        }))),
        Err(e) => Ok(Json(json!({ "ok": false, "error": e.to_string() }))),
    }
}
