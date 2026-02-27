use axum::{Router};
use crate::{ AppState, handler::user_handler::user_router, handler::event_handler::event_router };

pub fn routes() -> Router<AppState> {
    Router::new()
    .merge(user_router())
    .merge(event_router())
}