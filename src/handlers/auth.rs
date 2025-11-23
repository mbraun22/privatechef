use actix_web::{web, HttpResponse};
use bcrypt::{hash, verify, DEFAULT_COST};
use uuid::Uuid;

use crate::db::DbPool;
use crate::models::{CreateUser, User, UserResponse, Role};
use crate::errors::AppError;
use crate::utils::{create_refresh_token, create_token};
use crate::config::Config;

pub async fn register(
    pool: web::Data<DbPool>,
    config: web::Data<Config>,
    user_data: web::Json<CreateUser>,
) -> Result<HttpResponse, AppError> {
    // Check if user already exists
    let existing_user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE email = $1"
    )
    .bind(&user_data.email)
    .fetch_optional(pool.get_ref())
    .await?;

    if existing_user.is_some() {
        return Err(AppError::ValidationError("User with this email already exists".to_string()));
    }

    // Default role to diner if not specified
    // Only admins can create users with other roles (handled in admin endpoint)
    let role = user_data.role.unwrap_or(Role::Diner);

    // Hash password
    let password_hash = hash(&user_data.password, DEFAULT_COST)?;

    // Create user
    // Convert role to string for binding (sqlx will handle the conversion)
    let role_str = role.to_string();
    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (id, email, password_hash, role, created_at, updated_at)
        VALUES (gen_random_uuid(), $1, $2, $3::varchar, NOW(), NOW())
        RETURNING *
        "#
    )
    .bind(&user_data.email)
    .bind(&password_hash)
    .bind(&role_str)
    .fetch_one(pool.get_ref())
    .await?;

    // Generate tokens
    let token = create_token(user.id.to_string(), &config.jwt)?;
    let refresh_token = create_refresh_token(user.id.to_string(), &config.jwt)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "user": UserResponse::from(user),
        "token": token,
        "refresh_token": refresh_token
    })))
}

pub async fn login(
    pool: web::Data<DbPool>,
    config: web::Data<Config>,
    credentials: web::Json<serde_json::Value>,
) -> Result<HttpResponse, AppError> {
    let email = credentials.get("email")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::ValidationError("Email is required".to_string()))?;
    
    let password = credentials.get("password")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::ValidationError("Password is required".to_string()))?;

    // Find user
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE email = $1"
    )
    .bind(email)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::Unauthorized("Invalid credentials".to_string()))?;

    // Verify password
    if !verify(password, &user.password_hash)? {
        return Err(AppError::Unauthorized("Invalid credentials".to_string()));
    }

    // Generate tokens
    let token = create_token(user.id.to_string(), &config.jwt)?;
    let refresh_token = create_refresh_token(user.id.to_string(), &config.jwt)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "user": UserResponse::from(user),
        "token": token,
        "refresh_token": refresh_token
    })))
}

pub async fn get_me(
    pool: web::Data<DbPool>,
    user_id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE id = $1"
    )
    .bind(*user_id)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    Ok(HttpResponse::Ok().json(UserResponse::from(user)))
}

