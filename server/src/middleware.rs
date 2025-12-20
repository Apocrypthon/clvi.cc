use axum::{http::Request, response::Response};
use uuid::Uuid;

use crate::errors::AppError;

#[derive(Debug, Clone)]
pub struct Session {
    pub player_id: Option<Uuid>,
}

impl Session {
    pub fn new(player_id: Option<Uuid>) -> Self {
        Self { player_id }
    }
}

pub async fn session_middleware<B>(
    mut req: Request<B>,
    next: axum::middleware::Next<B>,
) -> Result<Response, AppError> {
    let player_id = req
        .headers()
        .get("x-player-id")
        .and_then(|value| value.to_str().ok())
        .map(|value| Uuid::parse_str(value))
        .transpose()
        .map_err(|_| AppError::Unauthorized)?;

    req.extensions_mut().insert(Session::new(player_id));
    let response = next.run(req).await;
    Ok(response)
}
