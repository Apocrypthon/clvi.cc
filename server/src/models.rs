use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Player {
    pub id: Uuid,
    pub username: String,
    pub wallet_address: Option<String>,
    pub guardian_tokens_completed: i32,
    pub skill_rating: f64,
    pub last_login: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct TrashItem {
    pub id: i64,
    pub category: String,
    pub base_value: f64,
    pub required_accuracy: f64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct GuardianTokenState {
    pub id: Uuid,
    pub current_progress: f64,
    pub is_completed: bool,
    pub last_contributor_id: Option<Uuid>,
}
