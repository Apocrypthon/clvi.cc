use axum::{extract::State, Extension, Json, Query};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::{errors::AppError, middleware::Session, state::AppState};

#[derive(Debug, Deserialize)]
pub struct PlayerStateQuery {
    pub player_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct PlayerStateResponse {
    pub player_id: Uuid,
    pub collected_total: i64,
    pub recycled_total: i64,
}

#[derive(Debug, FromRow)]
struct PlayerStateRow {
    player_id: Uuid,
    collected_total: Option<i64>,
    recycled_total: Option<i64>,
}

pub async fn state(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Query(query): Query<PlayerStateQuery>,
) -> Result<Json<PlayerStateResponse>, AppError> {
    let player_id = session
        .player_id
        .or(query.player_id)
        .ok_or(AppError::Unauthorized)?;

    let row = sqlx::query_as::<_, PlayerStateRow>(
        "SELECT player_id, SUM(collected) as collected_total, SUM(recycled) as recycled_total \
         FROM player_resources WHERE player_id = $1 GROUP BY player_id",
    )
    .bind(player_id)
    .fetch_optional(&state.db_pool)
    .await?
    .unwrap_or(PlayerStateRow {
        player_id,
        collected_total: Some(0),
        recycled_total: Some(0),
    });

    Ok(Json(PlayerStateResponse {
        player_id,
        collected_total: row.collected_total.unwrap_or_default(),
        recycled_total: row.recycled_total.unwrap_or_default(),
    }))
}
