use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde_json::json;
 

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("not found")]
    NotFound,

    #[error("unauthorized")]
    Unauthorized,

    #[error("conflict")]
    Conflict,

    #[error("{0}")]
    Validation(String),

    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            AppError::NotFound => (
                StatusCode::NOT_FOUND,
                "not_found",
                "resource not found".to_owned(),
            ),
            AppError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "unauthorized",
                "missing or invalid credentials".to_owned(),
            ),
            AppError::Conflict => (
                StatusCode::CONFLICT,
                "conflict",
                "the resource changed since you last read it".to_owned(),
            ),
            AppError::Validation(msg) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "validation_failed",
                msg.clone(),
            ),
            // Internal detail is logged, never returned.
            AppError::Db(err) => {
                tracing::error!(error = %err, "database error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "internal server error".to_owned(),
                )
            }
        };

        (status, Json(json!({ "error": { "code": code, "message": message } }))).into_response()
    }
}