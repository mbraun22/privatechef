use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::db::DbPool;
use crate::models::{MenuItem, CreateMenuItem, UpdateMenuItem};
use crate::errors::AppError;
use crate::middleware::auth::extract_user_id;
use actix_web::HttpRequest;

/// Creates a new menu item for a specific menu.
/// 
/// # Authorization
/// Only the chef who owns the menu or an admin can create menu items.
/// 
/// # Arguments
/// * `req` - HTTP request with bearer token
/// * `pool` - Database connection pool
/// * `menu_id` - UUID of the menu to add item to
/// * `data` - CreateMenuItem struct with item details
/// 
/// # Returns
/// * `HttpResponse::Created` with the created MenuItem on success
/// * `AppError::Unauthorized` if user doesn't own the menu
pub async fn create_menu_item(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    menu_id: web::Path<Uuid>,
    data: web::Json<CreateMenuItem>,
) -> Result<HttpResponse, AppError> {
    let user_id = extract_user_id(&req)?;
    let menu_id = *menu_id;

    tracing::debug!(
        user_id = %user_id,
        menu_id = %menu_id,
        item_name = data.name,
        "Attempting to create menu item"
    );

    // Verify ownership of the menu
    let _chef: Option<(uuid::Uuid,)> = sqlx::query_as(
        "SELECT c.id FROM chefs c
         INNER JOIN menus m ON m.chef_id = c.id
         WHERE c.user_id = $1 AND m.id = $2"
    )
    .bind(user_id)
    .bind(menu_id)
    .fetch_optional(pool.get_ref())
    .await?;
    
    if _chef.is_none() {
        tracing::warn!(
            user_id = %user_id,
            menu_id = %menu_id,
            "User attempted to create menu item for unauthorized menu"
        );
        return Err(AppError::Unauthorized("Not authorized".to_string()));
    }

    tracing::debug!(
        menu_id = %menu_id,
        "Menu ownership verified, creating menu item"
    );

    let menu_item = sqlx::query_as::<_, MenuItem>(
        r#"
        INSERT INTO menu_items (
            menu_id, name, description, course_type, image_url,
            is_featured, display_order, quantity, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), NOW())
        RETURNING *
        "#
    )
    .bind(menu_id)
    .bind(&data.name)
    .bind(&data.description)
    .bind(&data.course_type)
    .bind(&data.image_url)
    .bind(data.is_featured.unwrap_or(false))
    .bind(data.display_order.unwrap_or(0))
    .bind(data.quantity)
    .fetch_one(pool.get_ref())
    .await?;

    tracing::info!(
        menu_id = %menu_id,
        item_id = %menu_item.id,
        item_name = menu_item.name,
        "Menu item created successfully"
    );

    Ok(HttpResponse::Created().json(menu_item))
}

/// Retrieves all menu items for a specific menu.
/// 
/// # Arguments
/// * `pool` - Database connection pool
/// * `menu_id` - UUID of the menu
/// 
/// # Returns
/// * `HttpResponse::Ok` with array of MenuItems
pub async fn get_menu_items(
    pool: web::Data<DbPool>,
    menu_id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let menu_id = *menu_id;

    tracing::debug!(
        menu_id = %menu_id,
        "Fetching menu items"
    );

    let items = sqlx::query_as::<_, MenuItem>(
        "SELECT * FROM menu_items WHERE menu_id = $1 ORDER BY display_order ASC, created_at ASC"
    )
    .bind(menu_id)
    .fetch_all(pool.get_ref())
    .await?;

    tracing::debug!(
        menu_id = %menu_id,
        item_count = items.len(),
        "Menu items retrieved successfully"
    );

    Ok(HttpResponse::Ok().json(items))
}

/// Updates an existing menu item.
/// 
/// # Authorization
/// Only the chef who owns the menu or an admin can update menu items.
/// 
/// # Arguments
/// * `req` - HTTP request with bearer token
/// * `pool` - Database connection pool
/// * `path` - Tuple of (menu_id, item_id)
/// * `data` - UpdateMenuItem struct with fields to update
/// 
/// # Returns
/// * `HttpResponse::Ok` with updated MenuItem on success
/// * `AppError::Unauthorized` if user doesn't own the menu
/// * `AppError::NotFound` if menu item doesn't exist
pub async fn update_menu_item(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    path: web::Path<(Uuid, Uuid)>,
    data: web::Json<UpdateMenuItem>,
) -> Result<HttpResponse, AppError> {
    let (menu_id, item_id) = path.into_inner();
    let user_id = extract_user_id(&req)?;

    tracing::debug!(
        user_id = %user_id,
        menu_id = %menu_id,
        item_id = %item_id,
        "Attempting to update menu item"
    );

    // Verify ownership of the menu
    let _chef: Option<(uuid::Uuid,)> = sqlx::query_as(
        "SELECT c.id FROM chefs c
         INNER JOIN menus m ON m.chef_id = c.id
         WHERE c.user_id = $1 AND m.id = $2"
    )
    .bind(user_id)
    .bind(menu_id)
    .fetch_optional(pool.get_ref())
    .await?;
    
    if _chef.is_none() {
        tracing::warn!(
            user_id = %user_id,
            menu_id = %menu_id,
            item_id = %item_id,
            "User attempted to update menu item for unauthorized menu"
        );
        return Err(AppError::Unauthorized("Not authorized".to_string()));
    }

    tracing::debug!(
        menu_id = %menu_id,
        "Menu ownership verified, updating item"
    );

    let item = sqlx::query_as::<_, MenuItem>(
        r#"
        UPDATE menu_items
        SET name = COALESCE($1, name),
            description = COALESCE($2, description),
            course_type = COALESCE($3, course_type),
            image_url = COALESCE($4, image_url),
            is_featured = COALESCE($5, is_featured),
            display_order = COALESCE($6, display_order),
            quantity = COALESCE($7, quantity),
            updated_at = NOW()
        WHERE id = $8
        RETURNING *
        "#
    )
    .bind(&data.name)
    .bind(&data.description)
    .bind(&data.course_type)
    .bind(&data.image_url)
    .bind(&data.is_featured)
    .bind(&data.display_order)
    .bind(&data.quantity)
    .bind(item_id)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| {
        tracing::warn!(
            item_id = %item_id,
            "Menu item not found for update"
        );
        AppError::NotFound("Menu item not found".to_string())
    })?;

    tracing::info!(
        item_id = %item.id,
        item_name = item.name,
        "Menu item updated successfully"
    );

    Ok(HttpResponse::Ok().json(item))
}

/// Deletes a menu item.
/// 
/// # Authorization
/// Only the chef who owns the menu or an admin can delete menu items.
/// 
/// # Arguments
/// * `req` - HTTP request with bearer token
/// * `pool` - Database connection pool
/// * `path` - Tuple of (menu_id, item_id)
/// 
/// # Returns
/// * `HttpResponse::NoContent` on success
/// * `AppError::Unauthorized` if user doesn't own the menu
pub async fn delete_menu_item(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    path: web::Path<(Uuid, Uuid)>,
) -> Result<HttpResponse, AppError> {
    let (menu_id, item_id) = path.into_inner();
    let user_id = extract_user_id(&req)?;

    tracing::debug!(
        user_id = %user_id,
        menu_id = %menu_id,
        item_id = %item_id,
        "Attempting to delete menu item"
    );

    // Verify ownership of the menu
    let _chef: Option<(uuid::Uuid,)> = sqlx::query_as(
        "SELECT c.id FROM chefs c
         INNER JOIN menus m ON m.chef_id = c.id
         WHERE c.user_id = $1 AND m.id = $2"
    )
    .bind(user_id)
    .bind(menu_id)
    .fetch_optional(pool.get_ref())
    .await?;
    
    if _chef.is_none() {
        tracing::warn!(
            user_id = %user_id,
            menu_id = %menu_id,
            item_id = %item_id,
            "User attempted to delete menu item from unauthorized menu"
        );
        return Err(AppError::Unauthorized("Not authorized".to_string()));
    }

    tracing::debug!(
        menu_id = %menu_id,
        item_id = %item_id,
        "Menu ownership verified, deleting item"
    );

    sqlx::query("DELETE FROM menu_items WHERE id = $1")
        .bind(item_id)
        .execute(pool.get_ref())
        .await?;

    tracing::info!(
        item_id = %item_id,
        "Menu item deleted successfully"
    );

    Ok(HttpResponse::NoContent().finish())
}

