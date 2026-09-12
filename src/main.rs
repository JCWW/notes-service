use anyhow::Context;
use notes_service::{build_router, config::Config, AppState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Absent .env is normal — in production the environment is already set.
    dotenvy::dotenv().ok();

    let config = Config::from_env()?;
    let app = build_router(AppState::default());

    let listener = tokio::net::TcpListener::bind(config.bind_addr)
        .await
        .with_context(|| format!("could not bind {}", config.bind_addr))?;

    println!("listening on {}", listener.local_addr()?);

    axum::serve(listener, app).await.context("server error")?;

    Ok(())
}