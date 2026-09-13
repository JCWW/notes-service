use anyhow::Context;
use notes_service::{AppState, build_router, config::Config, telemetry};
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Absent .env is normal — in production the environment is already set.
    dotenvy::dotenv().ok();
    notes_service::telemetry::init("notes_service=debug,tower_http=debug,info");

    let config = Config::from_env()?;
    let db = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
        .context("could not connect to Postgres — is `docker compose up` running?")?;

    // Applied at boot so `cargo run` against an empty database just works.
    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .context("migrations failed")?;

    let app = build_router(AppState { db });

    let listener = tokio::net::TcpListener::bind(config.bind_addr)
        .await
        .with_context(|| format!("could not bind {}", config.bind_addr))?;

    tracing::info!(addr = %listener.local_addr()?, "listening on");

    axum::serve(listener, app).await.context("server error")?;

    Ok(())
}
