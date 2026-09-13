mod handlers;
pub mod model;
mod repo;

use axum::{
    routing::{get, post},
    Router,
};

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(handlers::create).get(handlers::list))
        .route(
            "/{id}",
            get(handlers::get)
                .patch(handlers::update)
                .delete(handlers::delete),
        )
}