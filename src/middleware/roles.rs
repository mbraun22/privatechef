use actix_web::HttpRequest;
use crate::db::DbPool;
use crate::models::{User, Role};
use crate::errors::AppError;
use crate::middleware::auth::extract_user_id;

/// Extract the authenticated user's role from the database
pub async fn extract_user_role(
    req: &HttpRequest,
    pool: &actix_web::web::Data<DbPool>,
) -> Result<Role, AppError> {
    let user_id = extract_user_id(req)?;
    
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE id = $1"
    )
    .bind(user_id)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    Ok(user.role)
}

/// Check if user has one of the required roles
pub async fn require_roles(
    req: &HttpRequest,
    pool: &actix_web::web::Data<DbPool>,
    allowed_roles: &[Role],
) -> Result<(), AppError> {
    let user_role = extract_user_role(req, pool).await?;
    
    if allowed_roles.contains(&user_role) {
        Ok(())
    } else {
        Err(AppError::Unauthorized(
            format!("Access denied. Required roles: {:?}", allowed_roles)
        ))
    }
}

/// Check if user is admin
pub async fn require_admin(
    req: &HttpRequest,
    pool: &actix_web::web::Data<DbPool>,
) -> Result<(), AppError> {
    require_roles(req, pool, &[Role::Admin]).await
}

/// Check if user is admin or moderator
pub async fn require_admin_or_mod(
    req: &HttpRequest,
    pool: &actix_web::web::Data<DbPool>,
) -> Result<(), AppError> {
    require_roles(req, pool, &[Role::Admin, Role::Mod]).await
}

/// Check if user is chef
pub async fn require_chef(
    req: &HttpRequest,
    pool: &actix_web::web::Data<DbPool>,
) -> Result<(), AppError> {
    require_roles(req, pool, &[Role::Chef]).await
}

/// Check if user is chef or admin
pub async fn require_chef_or_admin(
    req: &HttpRequest,
    pool: &actix_web::web::Data<DbPool>,
) -> Result<(), AppError> {
    require_roles(req, pool, &[Role::Chef, Role::Admin]).await
}

/// Helper function to check if a role has permission for an action
pub fn has_permission(role: Role, action: &str) -> bool {
    match role {
        Role::Admin => true, // Admins can do everything
        Role::Mod => {
            // Moderators can manage content and users (except admins)
            matches!(action, "manage_content" | "manage_users" | "view_reports")
        }
        Role::Chef => {
            // Chefs can manage their own content
            matches!(action, "manage_own_chef_profile" | "manage_own_menus" | "manage_own_bookings")
        }
        Role::Diner => {
            // Diners can view and book
            matches!(action, "view_chefs" | "create_booking" | "view_own_bookings")
        }
    }
}

