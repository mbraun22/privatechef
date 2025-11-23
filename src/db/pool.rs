use sqlx::{PgPool, Pool, Postgres};
use crate::config::DatabaseConfig;

pub type DbPool = Pool<Postgres>;

pub async fn create_pool(config: &DatabaseConfig) -> Result<DbPool, sqlx::Error> {
    PgPool::connect_with(
        config.url.parse().map_err(|e| {
            sqlx::Error::Configuration(format!("Invalid database URL: {}", e).into())
        })?
    )
    .await
}

