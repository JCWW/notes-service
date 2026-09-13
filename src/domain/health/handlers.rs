use axum::{extract::State, http::StatusCode, Json};
use serde::Serialize;

use crate::AppState;

#[derive(Serialize)]
pub struct HealthResponse {
    status: &'static str,
}

/// Liveness: is this process running. Never touches the database, so a
/// database outage does not get the container killed and restarted.
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

/// Readiness: can this process actually serve traffic.
pub async fn ready(State(state): State<AppState>) -> Result<Json<HealthResponse>, StatusCode> {
    sqlx::query("SELECT 1")
        .execute(&state.db)
        .await
        .map_err(|e| {
            tracing::warn!(error = %e, "readiness check failed");
            StatusCode::SERVICE_UNAVAILABLE
        })?;

    Ok(Json(HealthResponse { status: "ok" }))
}