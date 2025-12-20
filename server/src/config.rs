use std::{env, num::ParseIntError};

use crate::errors::AppError;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub server_port: u16,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, AppError> {
        let database_url =
            env::var("DATABASE_URL").map_err(|_| AppError::MissingConfig("DATABASE_URL".into()))?;

        let server_port = env::var("SERVER_PORT")
            .ok()
            .map(|port| parse_port(&port))
            .transpose()?
            .unwrap_or(3000);

        Ok(Self {
            database_url,
            server_port,
        })
    }
}

fn parse_port(value: &str) -> Result<u16, AppError> {
    let port: u16 = value
        .parse()
        .map_err(|err: ParseIntError| AppError::BadRequest(format!("Invalid port: {err}")))?;
    Ok(port)
}
