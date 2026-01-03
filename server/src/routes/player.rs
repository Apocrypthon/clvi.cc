use axum::{extract::State, Extension, Json, extract::Query};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::{auth, errors::AppError, middleware::Session, state::AppState};

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
    pub email: Option<String>,
    pub password: String,
    pub display_name: Option<String>,
    pub wallet_address: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreatePlayerResponse {
    pub id: Uuid,
    pub username: String,
    pub email: Option<String>,
    pub is_email_verified: bool,
    pub display_name: Option<String>,
    pub wallet_address: Option<String>,
    pub token: String,
    pub refresh_token: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub id: Uuid,
    pub username: String,
    pub token: String,
    pub refresh_token: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyEmailRequest {
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct VerifyEmailResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct RefreshTokenResponse {
    pub token: String,
    pub refresh_token: String,
}

#[derive(Debug, FromRow)]
struct CreatePlayerRow {
    id: Uuid,
    username: String,
    email: Option<String>,
    is_email_verified: bool,
    display_name: Option<String>,
    wallet_address: Option<String>,
}

#[derive(Debug, FromRow)]
struct LoginRow {
    id: Uuid,
    username: String,
}

#[derive(Debug, FromRow)]
struct RefreshTokenRow {
    player_id: Uuid,
}

async fn create_refresh_token(pool: &PgPool, player_id: Uuid) -> Result<String, AppError> {
    let refresh_token = Uuid::new_v4().to_string().replace("-", ""); // Simple random string

    sqlx::query(
        "INSERT INTO refresh_tokens (token_hash, player_id, expires_at)
         VALUES (encode(digest($1, 'sha256'), 'hex'), $2, NOW() + INTERVAL '30 days')"
    )
    .bind(&refresh_token)
    .bind(player_id)
    .execute(pool)
    .await?;

    Ok(refresh_token)
}

pub async fn create(
    State(state): State<AppState>,
    Json(payload): Json<CreatePlayerRequest>,
) -> Result<Json<CreatePlayerResponse>, AppError> {
    let verification_token = if payload.email.is_some() {
        Some(Uuid::new_v4().to_string())
    } else {
        None
    };

    if let Some(token) = &verification_token {
        tracing::info!("Generated verification token for {}: {}", payload.username, token);
    }

    let row = sqlx::query_as::<_, CreatePlayerRow>(
        r#"
        INSERT INTO players (username, email, password_hash, display_name, wallet_address, email_verification_token, email_verification_sent_at)
        VALUES (
            $1,
            $2,
            crypt($3, gen_salt('bf')),
            $4,
            $5,
            $6,
            CASE WHEN $6 IS NOT NULL THEN NOW() ELSE NULL END
        )
        RETURNING id, username, email, is_email_verified, display_name, wallet_address
        "#
    )
    .bind(&payload.username)
    .bind(&payload.email)
    .bind(&payload.password)
    .bind(&payload.display_name)
    .bind(&payload.wallet_address)
    .bind(&verification_token)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        // Handle unique constraint violation
        if let Some(db_error) = e.as_database_error() {
             if db_error.code().as_deref() == Some("23505") { // Unique violation
                 let constraint = db_error.constraint().unwrap_or("unknown");
                 if constraint.contains("email") {
                     return AppError::Conflict(format!("Email '{:?}' already taken", payload.email));
                 }
                 return AppError::Conflict(format!("Username '{}' already taken", payload.username));
             }
        }
        AppError::Database(e)
    })?;

    let token = auth::encode_token(row.id, &state.config.jwt_secret)?;
    let refresh_token = create_refresh_token(&state.db_pool, row.id).await?;

    Ok(Json(CreatePlayerResponse {
        id: row.id,
        username: row.username,
        email: row.email,
        is_email_verified: row.is_email_verified,
        display_name: row.display_name,
        wallet_address: row.wallet_address,
        token,
        refresh_token,
    }))
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    let row = sqlx::query_as::<_, LoginRow>(
        r#"
        SELECT id, username
        FROM players
        WHERE username = $1
        AND password_hash IS NOT NULL
        AND password_hash = crypt($2, password_hash)
        "#,
    )
    .bind(payload.username)
    .bind(payload.password)
    .fetch_optional(&state.db_pool)
    .await?
    .ok_or(AppError::Unauthorized)?;

    let token = auth::encode_token(row.id, &state.config.jwt_secret)?;
    let refresh_token = create_refresh_token(&state.db_pool, row.id).await?;

    Ok(Json(LoginResponse {
        id: row.id,
        username: row.username,
        token,
        refresh_token,
    }))
}

pub async fn refresh(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenRequest>,
) -> Result<Json<RefreshTokenResponse>, AppError> {
    // 1. Find valid token
    let row = sqlx::query_as::<_, RefreshTokenRow>(
        r#"
        DELETE FROM refresh_tokens
        WHERE token_hash = encode(digest($1, 'sha256'), 'hex')
        AND expires_at > NOW()
        RETURNING player_id
        "#
    )
    .bind(&payload.refresh_token)
    .fetch_optional(&state.db_pool)
    .await?
    .ok_or(AppError::Unauthorized)?;

    // 2. Issue new pair
    let token = auth::encode_token(row.player_id, &state.config.jwt_secret)?;
    let refresh_token = create_refresh_token(&state.db_pool, row.player_id).await?;

    Ok(Json(RefreshTokenResponse {
        token,
        refresh_token,
    }))
}

pub async fn revoke(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenRequest>,
) -> Result<(), AppError> {
    sqlx::query(
        "DELETE FROM refresh_tokens WHERE token_hash = encode(digest($1, 'sha256'), 'hex')"
    )
    .bind(&payload.refresh_token)
    .execute(&state.db_pool)
    .await?;

    Ok(())
}

pub async fn verify_email(
    State(state): State<AppState>,
    Json(payload): Json<VerifyEmailRequest>,
) -> Result<Json<VerifyEmailResponse>, AppError> {
    let result = sqlx::query(
        r#"
        UPDATE players
        SET is_email_verified = TRUE,
            email_verification_token = NULL
        WHERE email_verification_token = $1
        AND email_verification_sent_at > NOW() - INTERVAL '24 hours'
        RETURNING id
        "#
    )
    .bind(&payload.token)
    .fetch_optional(&state.db_pool)
    .await?;

    if result.is_some() {
        Ok(Json(VerifyEmailResponse {
            success: true,
            message: "Email verified successfully".to_string(),
        }))
    } else {
        Ok(Json(VerifyEmailResponse {
            success: false,
            message: "Invalid or expired verification token".to_string(),
        }))
    }
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
