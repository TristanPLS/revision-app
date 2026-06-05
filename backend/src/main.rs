mod ai;
mod config;
mod db;
mod error;
mod models;
mod routes;
mod srs;
mod state;

use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,revision_backend=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cfg = config::Config::from_env();
    ai::set_max_source_chars(cfg.ai_max_source_chars);

    let pool = db::connect(&cfg.database_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    tracing::info!("migrations applied");

    // Jobs orphelins : un redémarrage en pleine génération laisserait des jobs
    // 'running' éternels (et un spinner infini côté frontend). Une seule
    // instance backend par design, donc ce balayage est sûr.
    let orphans = sqlx::query(
        "UPDATE generation_jobs \
         SET status = 'failed', error = 'Interrompu par un redémarrage du serveur', finished_at = now() \
         WHERE status IN ('pending', 'running')",
    )
    .execute(&pool)
    .await?;
    if orphans.rows_affected() > 0 {
        tracing::warn!(
            count = orphans.rows_affected(),
            "orphaned generation jobs marked failed"
        );
    }

    // Réglages IA : la ligne app_settings (page Réglages) prime sur l'env.
    let settings = routes::settings::load_initial(&pool, &cfg).await;
    tracing::info!(
        provider = settings.provider.as_str(),
        model = %settings.model,
        ai_configured = settings.is_configured(),
        "starting revision-backend"
    );
    let ai = ai::AiClient::new(settings);

    let state = state::AppState {
        pool,
        ai,
        cfg: cfg.clone(),
    };

    // Pas de couche CORS : le frontend appelle l'API en même origine via les
    // rewrites Next.js (proxy serveur → serveur). Un CORS permissif laisserait
    // n'importe quel site visité par l'utilisateur lire/écrire l'API — dont la
    // clé BYOK via /api/settings.
    let app = routes::router(state).layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0, 0, 0, 0], cfg.backend_port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("listening on http://{addr}");
    axum::serve(listener, app).await?;
    Ok(())
}
