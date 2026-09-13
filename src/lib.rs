pub mod config;
pub mod domain;
pub mod routes;
pub mod telemetry;

pub use config::Config;
pub use routes::build_router;

/// Shared application state.
///
#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
}