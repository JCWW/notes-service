mod handlers;
pub mod model;
pub mod repo;

use axum::{
    routing::{delete, get, post},
    Router,
};

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(handlers::create).get(handlers::list))
        .route(
            "/{team_id}/members",
            get(handlers::list_members).post(handlers::add_member),
        )
        .route(
            "/{team_id}/members/{member_id}",
            delete(handlers::remove_member),
        )
}