use axum::{extract::State, Extension, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{errors::AppError, middleware::Session, state::AppState};

#[derive(Debug, Deserialize)]
pub struct ActionRequest {
    pub player_id: Option<Uuid>,
    pub resource: String,
    pub amount: i32,
}

#[derive(Debug, Serialize)]
pub struct ActionResponse {
    pub player_id: Uuid,
    pub action: String,
    pub resource: String,
    pub amount: i32,
    pub status: String,
}

pub async fn collect(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Json(payload): Json<ActionRequest>,
) -> Result<Json<ActionResponse>, AppError> {
    handle_action(&state, session, payload, "collect").await
}

pub async fn recycle(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Json(payload): Json<ActionRequest>,
) -> Result<Json<ActionResponse>, AppError> {
    handle_action(&state, session, payload, "recycle").await
}

async fn handle_action(
    state: &AppState,
    session: Session,
    payload: ActionRequest,
    action: &str,
) -> Result<Json<ActionResponse>, AppError> {
    if payload.amount <= 0 {
        return Err(AppError::BadRequest("amount must be positive".into()));
    }

    let player_id = session
        .player_id
        .or(payload.player_id)
        .ok_or(AppError::Unauthorized)?;

    let mut tx = state.db_pool.begin().await?;

    sqlx::query(
        "INSERT INTO player_actions (player_id, action, resource, amount) VALUES ($1, $2, $3, $4)",
    )
    .bind(player_id)
    .bind(action)
    .bind(&payload.resource)
    .bind(payload.amount)
    .execute(&mut *tx)
    .await?;

    sqlx::query("UPDATE player_state SET updated_at = NOW() WHERE player_id = $1")
        .bind(player_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(Json(ActionResponse {
        player_id,
        action: action.into(),
        resource: payload.resource,
        amount: payload.amount,
        status: "recorded".into(),
    }))
}
