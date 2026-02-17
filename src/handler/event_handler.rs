use axum::{
    Router,
};

use crate::AppState;

pub fn event_router() -> Router<AppState> {
    Router::new()
}
