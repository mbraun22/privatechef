use actix_web::{HttpResponse, ResponseError};
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    DatabaseError(sqlx::Error),
    RedisError(redis::RedisError),
    JwtError(jsonwebtoken::errors::Error),
    BcryptError(bcrypt::BcryptError),
    ValidationError(String),
    Unauthorized(String),
    NotFound(String),
    InternalError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AppError::DatabaseError(e) => write!(f, "Database error: {}", e),
            AppError::RedisError(e) => write!(f, "Redis error: {}", e),
            AppError::JwtError(e) => write!(f, "JWT error: {}", e),
            AppError::BcryptError(e) => write!(f, "Bcrypt error: {}", e),
            AppError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            AppError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AppError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::DatabaseError(e) => {
                // Log the actual database error for debugging
                tracing::error!("Database error: {:?}", e);
                // Only expose details in debug mode
                #[cfg(debug_assertions)]
                {
                    HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": "Database error occurred",
                        "details": format!("{}", e)
                    }))
                }
                #[cfg(not(debug_assertions))]
                {
                    HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": "Database error occurred"
                    }))
                }
            }
            AppError::RedisError(_) => {
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Cache error occurred"
                }))
            }
            AppError::JwtError(_) | AppError::Unauthorized(_) => {
                HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": self.to_string()
                }))
            }
            AppError::ValidationError(_) => {
                HttpResponse::BadRequest().json(serde_json::json!({
                    "error": self.to_string()
                }))
            }
            AppError::NotFound(_) => {
                HttpResponse::NotFound().json(serde_json::json!({
                    "error": self.to_string()
                }))
            }
            AppError::BcryptError(_) | AppError::InternalError(_) => {
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Internal server error"
                }))
            }
        }
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::DatabaseError(err)
    }
}

impl From<redis::RedisError> for AppError {
    fn from(err: redis::RedisError) -> Self {
        AppError::RedisError(err)
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        AppError::JwtError(err)
    }
}

impl From<bcrypt::BcryptError> for AppError {
    fn from(err: bcrypt::BcryptError) -> Self {
        AppError::BcryptError(err)
    }
}

