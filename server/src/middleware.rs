use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

use crate::{auth, config::AppConfig, errors::AppError};

#[derive(Debug, Clone)]
pub struct Session {
    pub player_id: Option<Uuid>,
}

impl Session {
    pub fn new(player_id: Option<Uuid>) -> Self {
        Self { player_id }
    }
}

pub async fn session_middleware(
    State(config): State<AppConfig>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|value| value.to_str().ok());

    let player_id = if let Some(auth_header) = auth_header {
        if let Some(token) = auth_header.strip_prefix("Bearer ") {
            let claims = auth::decode_token(token, &config.jwt_secret);
            match claims {
                Ok(claims) => Some(claims.sub),
                Err(_) => None, // Token invalid, treated as anonymous
            }
        } else {
            None
        }
    } else {
        None
    };

    // Fallback to legacy header for compatibility if needed, or remove it.
    // The requirement says "Issue JWT session token", implying we move to JWT.
    // I will disable the legacy header to be secure.

    req.extensions_mut().insert(Session::new(player_id));
    let response = next.run(req).await;
    Ok(response)
}
