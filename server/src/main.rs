use axum::{
    middleware as axum_middleware,
    routing::{get, post},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod errors;
mod middleware;
mod routes;
mod state;
mod models;

use crate::{config::AppConfig, errors::AppError, middleware as app_middleware, state::AppState};

#[tokio::main]
async fn main() -> Result<(), AppError> {
    dotenvy::dotenv().ok();
    setup_tracing();

    let config = AppConfig::from_env()?;
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?;

    let state = AppState::new(pool);

    let app = Router::new()
        .merge(routes::router())
        .with_state(state)
        .layer(axum_middleware::from_fn(app_middleware::session_middleware));

    let address = format!("0.0.0.0:{}", config.server_port);
    tracing::info!("listening on {address}");
    let listener = tokio::net::TcpListener::bind(&address).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn setup_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "server=debug,axum::rejection=trace,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}
