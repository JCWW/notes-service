use axum::{
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
};
use uuid::Uuid;

use crate::{AppState, error::AppError};

/// The authenticated caller.
///
/// STUB: the bearer token *is* the caller's user id. Real authentication —
/// JWT verification or a session lookup — is deliberately out of scope. What
/// matters for this exercise is that identity enters through one extractor,
/// so swapping the mechanism touches exactly this file.
#[derive(Debug, Clone, Copy)]
pub struct CurrentUser(pub Uuid);

impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .ok_or(AppError::Unauthorized)?;

        let user_id: Uuid = token.trim().parse().map_err(|_| AppError::Unauthorized)?;

        let exists =
            sqlx::query_scalar!("SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)", user_id)
                .fetch_one(&state.db)
                .await?
                .unwrap_or(false);

        if !exists {
            return Err(AppError::Unauthorized);
        }

        Ok(CurrentUser(user_id))
    }
}
