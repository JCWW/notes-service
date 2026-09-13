//! Route composition.
//!
//! Domains own their own routers. This module's only job is deciding how
//! they are mounted relative to each other.

use axum::Router;
use tower_http::trace::TraceLayer;

use crate::{domain, AppState};

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .merge(domain::health::router())
        .nest("/notes", domain::notes::router())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}