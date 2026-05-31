use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::time::Duration;

/// Build the Postgres connection pool. Retries briefly so the backend can start
/// alongside a freshly-booted database container.
pub async fn connect(database_url: &str) -> anyhow::Result<PgPool> {
    let mut last_err = None;
    for attempt in 1..=10 {
        match PgPoolOptions::new()
            .max_connections(10)
            .acquire_timeout(Duration::from_secs(10))
            .connect(database_url)
            .await
        {
            Ok(pool) => return Ok(pool),
            Err(e) => {
                tracing::warn!(attempt, error = %e, "database not ready, retrying in 2s");
                last_err = Some(e);
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
    }
    Err(anyhow::anyhow!(
        "could not connect to database after 10 attempts: {}",
        last_err.unwrap()
    ))
}
