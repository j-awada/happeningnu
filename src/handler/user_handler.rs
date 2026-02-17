use axum::{
    Router,
};

use crate::AppState;

pub fn user_router() -> Router<AppState> {
    Router::new()
}
