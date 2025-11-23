use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub jwt: JwtConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JwtConfig {
    pub secret: String,
    pub expiration: i64,
    pub refresh_expiration: i64,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        dotenv::dotenv().ok();

        Ok(Config {
            server: ServerConfig {
                host: env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
                port: env::var("SERVER_PORT")
                    .unwrap_or_else(|_| "8080".to_string())
                    .parse()
                    .map_err(|_| "Invalid SERVER_PORT".to_string())?,
            },
            database: DatabaseConfig {
                url: env::var("DATABASE_URL")
                    .map_err(|_| "DATABASE_URL must be set".to_string())?,
                max_connections: env::var("DATABASE_MAX_CONNECTIONS")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()
                    .unwrap_or(10),
            },
            redis: RedisConfig {
                url: env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
            },
            jwt: JwtConfig {
                secret: env::var("JWT_SECRET")
                    .map_err(|_| "JWT_SECRET must be set".to_string())?,
                expiration: env::var("JWT_EXPIRATION")
                    .unwrap_or_else(|_| "3600".to_string())
                    .parse()
                    .unwrap_or(3600),
                refresh_expiration: env::var("JWT_REFRESH_EXPIRATION")
                    .unwrap_or_else(|_| "86400".to_string())
                    .parse()
                    .unwrap_or(86400),
            },
        })
    }
}

