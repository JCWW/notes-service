pub mod domain;
pub mod routes;

pub use routes::build_router;
 
/// Shared application state.
/// 
#[derive(Clone, Default)]
pub struct AppState {}

 