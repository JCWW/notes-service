use axum::{Json, Router, routing::get};

/// Shared application state.
/// 
#[derive(Clone, Default)]
pub struct AppState {}

/// Builds the full application router.
///
pub fn build_router(state: AppState) -> Router {
    Router::new().route("/", get(root)).with_state(state)
}

async fn root() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "message": "hello world"
    }))
}