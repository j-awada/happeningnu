use axum::{ Router, routing::get, response::Html };
use crate::{ AppState, handler::user_handler::user_router, handler::event_handler::event_router };

pub fn routes() -> Router<AppState> {
    Router::new()
    .route("/", get(|| async { Html("<h1>Home page..</h2") }))
    .merge(user_router())
    .merge(event_router())
}