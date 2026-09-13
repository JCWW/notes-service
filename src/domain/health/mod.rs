mod handlers;

use axum::{Router, routing::get};

use crate::AppState;

/// Routes owned by the health domain.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(handlers::health))
        .route("/ready", get(handlers::ready))
}
