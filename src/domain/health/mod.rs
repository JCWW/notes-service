mod handlers;

use axum::{routing::get, Router};

use crate::AppState;

/// Routes owned by the health domain.
pub fn router() -> Router<AppState> {
    Router::new().route("/health", get(handlers::health))
}