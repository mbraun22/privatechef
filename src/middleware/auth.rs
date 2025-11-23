use actix_web::{dev::ServiceRequest, Error, HttpMessage};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use uuid::Uuid;

use crate::config::Config;
use crate::utils::verify_token;
use crate::errors::AppError;
use crate::cache::RedisClient;

pub struct AuthenticatedUser {
    pub user_id: Uuid,
}

/// Validator that checks Bearer token (for API routes)
pub async fn validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let config = match req.app_data::<actix_web::web::Data<Config>>() {
        Some(cfg) => cfg,
        None => {
            let error = actix_web::error::ErrorInternalServerError("Configuration not found");
            return Err((error, req));
        }
    };

    let token = credentials.token();
    
    match verify_token(token, &config.jwt.secret) {
        Ok(claims) => {
            let user_id = match Uuid::parse_str(&claims.sub) {
                Ok(id) => id,
                Err(_) => {
                    let error = actix_web::error::ErrorUnauthorized("Invalid user ID in token");
                    return Err((error, req));
                }
            };

            req.extensions_mut().insert(AuthenticatedUser { user_id });
            Ok(req)
        }
        Err(_) => {
            let error = actix_web::error::ErrorUnauthorized("Invalid or expired token");
            Err((error, req))
        }
    }
}

/// Extract user ID from session cookie (for web routes)
pub async fn extract_user_id_from_session(
    req: &actix_web::HttpRequest,
    redis: &actix_web::web::Data<RedisClient>,
) -> Result<Uuid, AppError> {
    // Get session ID from cookie
    tracing::debug!("Extracting session - all cookies: {:?}", req.cookies());
    let session_id = match req.cookie("session_id") {
        Some(cookie) => {
            let id = cookie.value().to_string();
            tracing::info!("Found session_id cookie: {}", id);
            id
        }
        None => {
            tracing::warn!("No session_id cookie found in request");
            tracing::warn!("Available cookies: {:?}", req.cookies());
            return Err(AppError::Unauthorized("No session found. Please log in.".to_string()));
        }
    };

    // Get session data from Redis
    tracing::debug!("Looking up session in Redis: {}", session_id);
    match redis.get_session_data(&session_id).await {
        Ok(Some(session_data)) => {
            tracing::debug!("Session found for user_id: {}", session_data.user_id);
            Ok(session_data.user_id)
        }
        Ok(None) => {
            tracing::warn!("Session not found in Redis: {}", session_id);
            Err(AppError::Unauthorized("Invalid or expired session. Please log in again.".to_string()))
        }
        Err(e) => {
            tracing::error!("Redis error when looking up session: {:?}", e);
            Err(AppError::Unauthorized("Session lookup failed".to_string()))
        }
    }
}

pub fn extract_user_id(req: &actix_web::HttpRequest) -> Result<Uuid, AppError> {
    req.extensions()
        .get::<AuthenticatedUser>()
        .map(|user| user.user_id)
        .ok_or_else(|| AppError::Unauthorized("User not authenticated".to_string()))
}

