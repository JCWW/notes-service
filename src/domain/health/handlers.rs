use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct HealthResponse {
    status: &'static str,
}

/// Liveness only: it answers "is this process up", not "can it serve
/// traffic". 
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}