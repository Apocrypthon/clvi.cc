use axum::{extract::State, Extension, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{errors::AppError, middleware::Session, state::AppState};

#[derive(Debug, Deserialize)]
pub struct RemediationRequest {
    pub player_id: Option<Uuid>,
    pub item_id: i64,
    pub success: bool,
    pub timing_ms: i64,
}

#[derive(Debug, Serialize)]
pub struct RemediationResponse {
    pub player_id: Uuid,
    pub item_id: i64,
    pub token_increment: f64,
}

#[derive(Debug, Serialize)]
pub struct LeaderboardResponse {
    pub players: Vec<crate::models::LeaderboardPlayer>,
}

pub async fn process(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Json(payload): Json<RemediationRequest>,
) -> Result<Json<RemediationResponse>, AppError> {
    let player_id = session
        .player_id
        .or(payload.player_id)
        .ok_or(AppError::Unauthorized)?;

    let token_increment = state
        .process_remediation(player_id, payload.item_id, payload.success, payload.timing_ms)
        .await?;

    Ok(Json(RemediationResponse {
        player_id,
        item_id: payload.item_id,
        token_increment,
    }))
}

pub async fn leaderboard(
    State(state): State<AppState>,
) -> Result<Json<LeaderboardResponse>, AppError> {
    let players = state.get_leaderboard().await?;
    Ok(Json(LeaderboardResponse { players }))
}
