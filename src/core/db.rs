use anyhow::{Context, Result};
use sqlx::postgres::{PgPool, PgPoolOptions};

pub async fn create_pool(database_url: &str) -> Result<PgPool> {
    ensure_database_exists(database_url).await?;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    tracing::info!("database pool established");
    Ok(pool)
}

pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    sqlx::migrate!().run(pool).await?;
    tracing::info!("database migrations applied");
    Ok(())
}

async fn ensure_database_exists(database_url: &str) -> Result<()> {
    let url: url::Url = database_url.parse().context("invalid DATABASE_URL")?;
    let db_name = url.path().trim_start_matches('/');
    if db_name.is_empty() {
        anyhow::bail!("DATABASE_URL must include a database name");
    }

    let mut maintenance_url = url.clone();
    maintenance_url.set_path("/postgres");
    let maintenance_pool = PgPool::connect(maintenance_url.as_str()).await?;

    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pg_database WHERE datname = $1)")
            .bind(db_name)
            .fetch_one(&maintenance_pool)
            .await?;

    if !exists {
        let stmt = format!("CREATE DATABASE \"{}\"", db_name);
        sqlx::query(&stmt).execute(&maintenance_pool).await?;
        tracing::info!(db = db_name, "database created");
    }

    maintenance_pool.close().await;
    Ok(())
}
