use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::db::DbPool;
use crate::models::{Menu, CreateMenu, UpdateMenu};
use crate::errors::AppError;
use crate::middleware::auth::extract_user_id;
use actix_web::HttpRequest;

pub async fn create_menu(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    data: web::Json<CreateMenu>,
) -> Result<HttpResponse, AppError> {
    let user_id = extract_user_id(&req)?;

    // Verify user is a chef
    let chef: Option<(uuid::Uuid,)> = sqlx::query_as(
        "SELECT id FROM chefs WHERE user_id = $1"
    )
    .bind(user_id)
    .fetch_optional(pool.get_ref())
    .await?;
    
    let chef_id = chef
        .map(|(id,)| id)
        .ok_or_else(|| AppError::Unauthorized("User is not a chef".to_string()))?;

    let menu = sqlx::query_as::<_, Menu>(
        r#"
        INSERT INTO menus (
            chef_id, name, description, price_per_person, minimum_guests,
            cuisine_type, dietary_options, duration_hours, is_active, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW(), NOW())
        RETURNING *
        "#
    )
    .bind(chef_id)
    .bind(&data.name)
    .bind(&data.description)
    .bind(&data.price_per_person)
    .bind(data.minimum_guests.unwrap_or(2))
    .bind(&data.cuisine_type)
    .bind(&data.dietary_options)
    .bind(&data.duration_hours)
    .bind(true)
    .fetch_one(pool.get_ref())
    .await?;

    Ok(HttpResponse::Created().json(menu))
}

pub async fn get_chef_menus(
    req: HttpRequest,
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, AppError> {
    let user_id = extract_user_id(&req)?;

    let chef: Option<(uuid::Uuid,)> = sqlx::query_as(
        "SELECT id FROM chefs WHERE user_id = $1"
    )
    .bind(user_id)
    .fetch_optional(pool.get_ref())
    .await?;
    
    let chef_id = chef
        .map(|(id,)| id)
        .ok_or_else(|| AppError::Unauthorized("User is not a chef".to_string()))?;

    let menus = sqlx::query_as::<_, Menu>(
        "SELECT * FROM menus WHERE chef_id = $1 ORDER BY created_at DESC"
    )
    .bind(chef_id)
    .fetch_all(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(menus))
}

pub async fn update_menu(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    menu_id: web::Path<Uuid>,
    data: web::Json<UpdateMenu>,
) -> Result<HttpResponse, AppError> {
    let user_id = extract_user_id(&req)?;

    // Verify ownership
    let chef: Option<(uuid::Uuid,)> = sqlx::query_as(
        "SELECT c.id FROM chefs c
         INNER JOIN menus m ON m.chef_id = c.id
         WHERE c.user_id = $1 AND m.id = $2"
    )
    .bind(user_id)
    .bind(*menu_id)
    .fetch_optional(pool.get_ref())
    .await?;
    
    if chef.is_none() {
        return Err(AppError::Unauthorized("Not authorized to update this menu".to_string()));
    }

    // Build update query dynamically
    let mut updates = Vec::new();
    if data.name.is_some() { updates.push("name = $1"); }
    if data.description.is_some() { updates.push("description = $2"); }
    if data.price_per_person.is_some() { updates.push("price_per_person = $3"); }
    if data.is_active.is_some() { updates.push("is_active = $4"); }
    updates.push("updated_at = NOW()");

    let menu = sqlx::query_as::<_, Menu>(
        &format!(
            "UPDATE menus SET {} WHERE id = $5 RETURNING *",
            updates.join(", ")
        )
    )
    .bind(&data.name)
    .bind(&data.description)
    .bind(&data.price_per_person)
    .bind(&data.is_active)
    .bind(*menu_id)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Menu not found".to_string()))?;

    Ok(HttpResponse::Ok().json(menu))
}

pub async fn delete_menu(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    menu_id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let user_id = extract_user_id(&req)?;

    // Verify ownership
    let chef: Option<(uuid::Uuid,)> = sqlx::query_as(
        "SELECT c.id FROM chefs c
         INNER JOIN menus m ON m.chef_id = c.id
         WHERE c.user_id = $1 AND m.id = $2"
    )
    .bind(user_id)
    .bind(*menu_id)
    .fetch_optional(pool.get_ref())
    .await?;
    
    if chef.is_none() {
        return Err(AppError::Unauthorized("Not authorized to delete this menu".to_string()));
    }

    sqlx::query("DELETE FROM menus WHERE id = $1")
        .bind(*menu_id)
        .execute(pool.get_ref())
        .await?;

    Ok(HttpResponse::NoContent().finish())
}

