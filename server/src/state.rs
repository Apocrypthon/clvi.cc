use sqlx::{PgPool, Postgres, Transaction, FromRow};
use uuid::Uuid;

use crate::models::Player;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
}

impl AppState {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    pub async fn process_remediation(
        &self,
        player_id: Uuid,
        item_id: i64,
        success: bool,
        timing_ms: i64,
    ) -> Result<f64, sqlx::Error> {
        let mut tx = self.db_pool.begin().await?;

        let token_increment = if success {
            let multiplier = match timing_ms {
                0..=500 => 1.5,
                501..=1000 => 1.0,
                _ => 0.5,
            };
            0.01 * multiplier
        } else {
            0.0
        };

        let token_completed =
            update_guardian_token(&mut tx, token_increment, player_id).await?;

        validate_trash_item(&mut tx, item_id).await?;

        if success {
            update_player_stats(&mut tx, player_id, token_completed).await?;
        }

        tx.commit().await?;
        Ok(token_increment)
    }

    pub async fn get_leaderboard(&self) -> Result<Vec<Player>, sqlx::Error> {
        let rows = sqlx::query_as::<_, Player>(
            "SELECT id, username, wallet_address, guardian_tokens_completed, skill_rating, last_login \
             FROM players \
             ORDER BY guardian_tokens_completed DESC, skill_rating DESC \
             LIMIT 100"
        )
        .fetch_all(&self.db_pool)
        .await?;

        Ok(rows)
    }
}

#[derive(FromRow)]
struct GuardianTokenRow {
    is_completed: Option<bool>,
}

async fn update_guardian_token(
    tx: &mut Transaction<'_, Postgres>,
    token_increment: f64,
    player_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let record = sqlx::query_as::<_, GuardianTokenRow>(
        "WITH target AS (
            SELECT id, current_progress
            FROM guardian_tokens
            WHERE is_completed = false
            ORDER BY created_at ASC
            LIMIT 1
            FOR UPDATE
         )
         UPDATE guardian_tokens
         SET current_progress = current_progress + $1,
             last_contributor_id = $2,
             is_completed = CASE WHEN current_progress + $1 >= 1.0 THEN true ELSE false END
         WHERE id IN (SELECT id FROM target)
         RETURNING is_completed",
    )
    .bind(token_increment)
    .bind(player_id)
    .fetch_optional(&mut **tx)
    .await?;

    Ok(record.map(|row| row.is_completed.unwrap_or(false)).unwrap_or(false))
}

async fn update_player_stats(
    tx: &mut Transaction<'_, Postgres>,
    player_id: Uuid,
    token_completed: bool,
) -> Result<(), sqlx::Error> {
    let token_delta = if token_completed { 1 } else { 0 };
    sqlx::query(
        "UPDATE players SET \
            guardian_tokens_completed = guardian_tokens_completed + $2, \
            skill_rating = skill_rating + 0.1, \
            last_login = NOW() \
         WHERE id = $1",
    )
    .bind(player_id)
    .bind(token_delta)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

async fn validate_trash_item(
    tx: &mut Transaction<'_, Postgres>,
    item_id: i64,
) -> Result<(), sqlx::Error> {
    // We can just check if any row exists
    let exists: (bool,) = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM trash_items WHERE id = $1) as \"exists\"",
    )
    .bind(item_id)
    .fetch_one(&mut **tx)
    .await?;

    if exists.0 {
        Ok(())
    } else {
        Err(sqlx::Error::RowNotFound)
    }
}
