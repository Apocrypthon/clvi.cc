use axum::{extract::State, Extension, Json, extract::Query};
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

#[derive(Debug, Deserialize)]
pub struct CreatePlayerRequest {
    pub username: String,
    pub display_name: Option<String>,
    pub wallet_address: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreatePlayerResponse {
    pub id: Uuid,
    pub username: String,
    pub display_name: Option<String>,
    pub wallet_address: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub id: Uuid,
    pub username: String,
}

#[derive(Debug, FromRow)]
struct CreatePlayerRow {
    id: Uuid,
    username: String,
    display_name: Option<String>,
    wallet_address: Option<String>,
}

#[derive(Debug, FromRow)]
struct LoginRow {
    id: Uuid,
    username: String,
}

pub async fn create(
    State(state): State<AppState>,
    Json(payload): Json<CreatePlayerRequest>,
) -> Result<Json<CreatePlayerResponse>, AppError> {
    let row = sqlx::query_as::<_, CreatePlayerRow>(
        r#"
        INSERT INTO players (username, display_name, wallet_address)
        VALUES ($1, $2, $3)
        RETURNING id, username, display_name, wallet_address
        "#
    )
    .bind(&payload.username)
    .bind(&payload.display_name)
    .bind(&payload.wallet_address)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        // Handle unique constraint violation on username
        if let Some(db_error) = e.as_database_error() {
             if db_error.code().as_deref() == Some("23505") { // Unique violation
                 return AppError::Conflict(format!("Username '{}' already taken", payload.username));
             }
        }
        AppError::Database(e)
    })?;

    Ok(Json(CreatePlayerResponse {
        id: row.id,
        username: row.username,
        display_name: row.display_name,
        wallet_address: row.wallet_address,
    }))
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    let row = sqlx::query_as::<_, LoginRow>(
        "SELECT id, username FROM players WHERE username = $1",
    )
    .bind(payload.username)
    .fetch_optional(&state.db_pool)
    .await?
    .ok_or(AppError::NotFound("Player not found".to_string()))?;

    Ok(Json(LoginResponse {
        id: row.id,
        username: row.username,
    }))
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
        r#"
        SELECT
            player_id,
            SUM(CASE WHEN transaction_type = 'collect' THEN quantity ELSE 0 END) as collected_total,
            SUM(CASE WHEN transaction_type = 'recycle' THEN quantity ELSE 0 END) as recycled_total
        FROM transactions
        WHERE player_id = $1
        GROUP BY player_id
        "#
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
