use axum::{
    routing::{get, post},
    Router,
};

use crate::state::AppState;

pub mod action;
pub mod player;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/action/collect", post(action::collect))
        .route("/action/recycle", post(action::recycle))
        .route("/player/state", get(player::state))
}
