use actix_web::{web, HttpResponse};
use uuid::Uuid;
use slug::slugify;

use crate::db::DbPool;
use crate::models::{Chef, CreateChef, UpdateChef, ChefPublicProfile, MenuItem, MenuItemPublic};
use crate::errors::AppError;

pub async fn create_chef_profile(
    pool: web::Data<DbPool>,
    user_id: web::Path<Uuid>,
    data: web::Json<CreateChef>,
) -> Result<HttpResponse, AppError> {
    // Check if chef profile already exists
    let existing = sqlx::query_as::<_, Chef>(
        "SELECT * FROM chefs WHERE user_id = $1"
    )
    .bind(*user_id)
    .fetch_optional(pool.get_ref())
    .await?;

    if existing.is_some() {
        return Err(AppError::ValidationError("Chef profile already exists".to_string()));
    }

    // Generate slug from chef name
    let slug = Some(slugify(&data.chef_name));

    let chef = sqlx::query_as::<_, Chef>(
        r#"
        INSERT INTO chefs (
            user_id, business_name, chef_name, bio, cuisine_types, location,
            phone, email, website, profile_image_url, cover_image_url,
            hourly_rate, minimum_hours, travel_radius, slug, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, NOW(), NOW())
        RETURNING *
        "#
    )
    .bind(*user_id)
    .bind(&data.business_name)
    .bind(&data.chef_name)
    .bind(&data.bio)
    .bind(&data.cuisine_types)
    .bind(&data.location)
    .bind(&data.phone)
    .bind(&data.email)
    .bind(&data.website)
    .bind(&data.profile_image_url)
    .bind(&data.cover_image_url)
    .bind(&data.hourly_rate)
    .bind(data.minimum_hours.unwrap_or(2))
    .bind(&data.travel_radius)
    .bind(&slug)
    .fetch_one(pool.get_ref())
    .await?;

    Ok(HttpResponse::Created().json(chef))
}

pub async fn get_chef_profile(
    pool: web::Data<DbPool>,
    user_id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let chef = sqlx::query_as::<_, Chef>(
        "SELECT * FROM chefs WHERE user_id = $1"
    )
    .bind(*user_id)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Chef profile not found".to_string()))?;

    Ok(HttpResponse::Ok().json(chef))
}

pub async fn update_chef_profile(
    pool: web::Data<DbPool>,
    user_id: web::Path<Uuid>,
    data: web::Json<UpdateChef>,
) -> Result<HttpResponse, AppError> {
    let mut update_fields = Vec::new();
    let mut bind_index = 1;

    if data.chef_name.is_some() {
        update_fields.push(format!("chef_name = ${}", bind_index));
        bind_index += 1;
    }
    if data.business_name.is_some() {
        update_fields.push(format!("business_name = ${}", bind_index));
        bind_index += 1;
    }
    if data.bio.is_some() {
        update_fields.push(format!("bio = ${}", bind_index));
        bind_index += 1;
    }
    if data.cuisine_types.is_some() {
        update_fields.push(format!("cuisine_types = ${}", bind_index));
        bind_index += 1;
    }
    if data.location.is_some() {
        update_fields.push(format!("location = ${}", bind_index));
        bind_index += 1;
    }
    if data.hourly_rate.is_some() {
        update_fields.push(format!("hourly_rate = ${}", bind_index));
        bind_index += 1;
    }
    if data.is_active.is_some() {
        update_fields.push(format!("is_active = ${}", bind_index));
        bind_index += 1;
    }

    if update_fields.is_empty() {
        return Err(AppError::ValidationError("No fields to update".to_string()));
    }

    update_fields.push(format!("updated_at = NOW()"));

    let query = format!(
        "UPDATE chefs SET {} WHERE user_id = ${} RETURNING *",
        update_fields.join(", "),
        bind_index
    );

    let mut query_builder = sqlx::query_as::<_, Chef>(&query);
    
    if let Some(ref name) = data.chef_name {
        query_builder = query_builder.bind(name);
    }
    if let Some(ref business) = data.business_name {
        query_builder = query_builder.bind(business);
    }
    if let Some(ref bio) = data.bio {
        query_builder = query_builder.bind(bio);
    }
    if let Some(ref cuisines) = data.cuisine_types {
        query_builder = query_builder.bind(cuisines);
    }
    if let Some(ref location) = data.location {
        query_builder = query_builder.bind(location);
    }
    if let Some(ref rate) = data.hourly_rate {
        query_builder = query_builder.bind(rate);
    }
    if let Some(ref active) = data.is_active {
        query_builder = query_builder.bind(active);
    }
    
    query_builder = query_builder.bind(*user_id);

    let chef = query_builder
        .fetch_optional(pool.get_ref())
        .await?
        .ok_or_else(|| AppError::NotFound("Chef profile not found".to_string()))?;

    Ok(HttpResponse::Ok().json(chef))
}

pub async fn get_public_chef_profile(
    pool: web::Data<DbPool>,
    slug: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let chef = sqlx::query_as::<_, Chef>(
        "SELECT * FROM chefs WHERE slug = $1 AND is_active = true"
    )
    .bind(slug.as_str())
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Chef profile not found".to_string()))?;

    // Get featured menu items
    let featured_items = sqlx::query_as::<_, MenuItem>(
        r#"
        SELECT mi.* FROM menu_items mi
        INNER JOIN menus m ON mi.menu_id = m.id
        WHERE m.chef_id = $1 AND mi.is_featured = true AND m.is_active = true
        ORDER BY mi.display_order ASC
        LIMIT 6
        "#
    )
    .bind(chef.id)
    .fetch_all(pool.get_ref())
    .await?;

    let mut public_profile: ChefPublicProfile = chef.into();
    public_profile.featured_menu_items = featured_items
        .into_iter()
        .map(|item| MenuItemPublic {
            id: item.id,
            name: item.name,
            description: item.description,
            course_type: item.course_type,
            image_url: item.image_url,
        })
        .collect();

    Ok(HttpResponse::Ok().json(public_profile))
}

