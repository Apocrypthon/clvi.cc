use axum::{
    routing::{get, post},
    Router,
};

use crate::state::AppState;

pub mod action;
pub mod player;
pub mod remediation;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/action/collect", post(action::collect))
        .route("/action/recycle", post(action::recycle))
        .route("/player", post(player::create))
        .route("/player/login", post(player::login))
        .route("/player/state", get(player::state))
        .route("/remediation/process", post(remediation::process))
        .route("/leaderboard", get(remediation::leaderboard))
}
