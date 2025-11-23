use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::config::RedisConfig;
use crate::errors::AppError;

#[derive(Clone)]
pub struct RedisClient {
    client: redis::Client,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionData {
    pub user_id: Uuid,
    pub email: String,
    pub role: String,
}

impl RedisClient {
    pub async fn new(config: &RedisConfig) -> Result<Self, AppError> {
        let client = redis::Client::open(config.url.as_str())?;
        Ok(RedisClient { client })
    }

    pub async fn get_connection(&self) -> Result<redis::aio::Connection, AppError> {
        Ok(self.client.get_async_connection().await?)
    }

    pub async fn set_session(&self, key: &str, value: &str, ttl: usize) -> Result<(), AppError> {
        let mut conn = self.get_connection().await?;
        let _: () = conn.set_ex(key, value, ttl as u64).await?;
        Ok(())
    }

    pub async fn get_session(&self, key: &str) -> Result<Option<String>, AppError> {
        let mut conn = self.get_connection().await?;
        let value: Option<String> = conn.get(key).await?;
        Ok(value)
    }

    pub async fn delete_session(&self, key: &str) -> Result<(), AppError> {
        let mut conn = self.get_connection().await?;
        let _: () = conn.del(key).await?;
        Ok(())
    }

    pub async fn set_cache(&self, key: &str, value: &str, ttl: usize) -> Result<(), AppError> {
        self.set_session(key, value, ttl).await
    }

    pub async fn get_cache(&self, key: &str) -> Result<Option<String>, AppError> {
        self.get_session(key).await
    }

    /// Create a new session and store session data
    pub async fn create_session(&self, session_id: &str, session_data: &SessionData, ttl: usize) -> Result<(), AppError> {
        let session_json = serde_json::to_string(session_data)
            .map_err(|e| AppError::InternalError(format!("Failed to serialize session: {}", e)))?;
        self.set_session(&format!("session:{}", session_id), &session_json, ttl).await
    }

    /// Get session data by session ID
    pub async fn get_session_data(&self, session_id: &str) -> Result<Option<SessionData>, AppError> {
        match self.get_session(&format!("session:{}", session_id)).await? {
            Some(json) => {
                serde_json::from_str(&json)
                    .map(Some)
                    .map_err(|e| AppError::InternalError(format!("Failed to deserialize session: {}", e)))
            }
            None => Ok(None),
        }
    }

    /// Delete a session
    pub async fn destroy_session(&self, session_id: &str) -> Result<(), AppError> {
        self.delete_session(&format!("session:{}", session_id)).await
    }
}

