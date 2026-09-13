use anyhow::Context;
use notes_service::{build_router, config::Config, AppState};
 
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Absent .env is normal — in production the environment is already set.
    dotenvy::dotenv().ok();
    notes_service::telemetry::init("notes_service=debug,tower_http=debug,info");

    let config = Config::from_env()?;
    let app = build_router(AppState::default());

    let listener = tokio::net::TcpListener::bind(config.bind_addr)
        .await
        .with_context(|| format!("could not bind {}", config.bind_addr))?;

    tracing::info!(addr = %listener.local_addr()?, "listening on");
 
    axum::serve(listener, app).await.context("server error")?;

    Ok(())
}