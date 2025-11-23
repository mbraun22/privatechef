use actix_web::{web, HttpRequest};
use actix_web_httpauth::middleware::HttpAuthentication;
use uuid::Uuid;

use crate::handlers::{auth, chef, menu, menu_item, booking};
use crate::middleware::auth::{validator, extract_user_id};
use crate::middleware::roles::require_chef_or_admin;
use crate::config::Config;
use crate::db::DbPool;
use crate::errors::AppError;

pub fn configure_api(cfg: &mut web::ServiceConfig, pool: web::Data<DbPool>, config: web::Data<Config>) {
    let auth_middleware = HttpAuthentication::bearer(validator);

    cfg.service(
        web::scope("/api")
            .app_data(pool.clone())
            .app_data(config.clone())
            // Public routes
            .service(
                web::scope("/chefs")
                    .route("/{slug}", web::get().to(chef::get_public_chef_profile))
            )
            .service(
                web::scope("/chefs/{chef_id}")
                    .route("/availability", web::get().to(booking::get_chef_availability))
                    .route("/bookings", web::post().to(booking::create_booking))
            )
            // Protected routes
            .service(
                web::scope("/users")
                    .wrap(auth_middleware.clone())
                    .route("/me", web::get().to(get_me_wrapper))
            )
            .service(
                web::scope("/chefs")
                    .wrap(auth_middleware.clone())
                    .route("", web::post().to(create_chef_wrapper))
                    .route("/profile", web::get().to(get_chef_profile_wrapper))
                    .route("/profile", web::put().to(update_chef_profile_wrapper))
            )
            .service(
                web::scope("/menus")
                    .wrap(auth_middleware.clone())
                    .route("", web::get().to(get_menus_wrapper))
                    .route("", web::post().to(create_menu_wrapper))
                    .route("/{menu_id}", web::put().to(update_menu_wrapper))
                    .route("/{menu_id}", web::delete().to(delete_menu_wrapper))
            )
            .service(
                web::scope("/menus/{menu_id}/items")
                    .wrap(auth_middleware.clone())
                    .route("", web::get().to(get_menu_items_wrapper))
                    .route("", web::post().to(create_menu_item_wrapper))
                    .route("/{item_id}", web::put().to(update_menu_item_wrapper))
                    .route("/{item_id}", web::delete().to(delete_menu_item_wrapper))
            )
            .service(
                web::scope("/chefs/{chef_id}/bookings")
                    .wrap(auth_middleware.clone())
                    .route("", web::get().to(get_chef_bookings_wrapper))
            )
            .service(
                web::scope("/bookings/{booking_id}")
                    .wrap(auth_middleware.clone())
                    .route("", web::put().to(update_booking_wrapper))
            )
    );
}

async fn get_me_wrapper(
    req: HttpRequest,
    pool: web::Data<DbPool>,
) -> Result<actix_web::HttpResponse, AppError> {
    let user_id = extract_user_id(&req)?;
    auth::get_me(pool, web::Path::from(user_id)).await
}

async fn create_chef_wrapper(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    data: web::Json<crate::models::CreateChef>,
) -> Result<actix_web::HttpResponse, AppError> {
    // Check if user has chef or admin role
    require_chef_or_admin(&req, &pool).await?;
    let user_id = extract_user_id(&req)?;
    chef::create_chef_profile(pool, web::Path::from(user_id), data).await
}

async fn get_chef_profile_wrapper(
    req: HttpRequest,
    pool: web::Data<DbPool>,
) -> Result<actix_web::HttpResponse, AppError> {
    // Check if user has chef or admin role
    require_chef_or_admin(&req, &pool).await?;
    let user_id = extract_user_id(&req)?;
    chef::get_chef_profile(pool, web::Path::from(user_id)).await
}

async fn update_chef_profile_wrapper(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    data: web::Json<crate::models::UpdateChef>,
) -> Result<actix_web::HttpResponse, AppError> {
    // Check if user has chef or admin role
    require_chef_or_admin(&req, &pool).await?;
    let user_id = extract_user_id(&req)?;
    chef::update_chef_profile(pool, web::Path::from(user_id), data).await
}

async fn get_menus_wrapper(
    req: HttpRequest,
    pool: web::Data<DbPool>,
) -> Result<actix_web::HttpResponse, AppError> {
    // Check if user has chef or admin role
    require_chef_or_admin(&req, &pool).await?;
    menu::get_chef_menus(req, pool).await
}

async fn create_menu_wrapper(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    data: web::Json<crate::models::CreateMenu>,
) -> Result<actix_web::HttpResponse, AppError> {
    // Check if user has chef or admin role
    require_chef_or_admin(&req, &pool).await?;
    menu::create_menu(req, pool, data).await
}

async fn update_menu_wrapper(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    path: web::Path<Uuid>,
    data: web::Json<crate::models::UpdateMenu>,
) -> Result<actix_web::HttpResponse, AppError> {
    // Check if user has chef or admin role
    require_chef_or_admin(&req, &pool).await?;
    menu::update_menu(req, pool, path, data).await
}

async fn delete_menu_wrapper(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    path: web::Path<Uuid>,
) -> Result<actix_web::HttpResponse, AppError> {
    // Check if user has chef or admin role
    require_chef_or_admin(&req, &pool).await?;
    menu::delete_menu(req, pool, path).await
}

async fn get_menu_items_wrapper(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    path: web::Path<Uuid>,
) -> Result<actix_web::HttpResponse, AppError> {
    // Check if user has chef or admin role
    require_chef_or_admin(&req, &pool).await?;
    menu_item::get_menu_items(pool, path).await
}

async fn create_menu_item_wrapper(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    path: web::Path<Uuid>,
    data: web::Json<crate::models::CreateMenuItem>,
) -> Result<actix_web::HttpResponse, AppError> {
    // Check if user has chef or admin role
    require_chef_or_admin(&req, &pool).await?;
    menu_item::create_menu_item(req, pool, path, data).await
}

async fn update_menu_item_wrapper(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    path: web::Path<(Uuid, Uuid)>,
    data: web::Json<crate::models::UpdateMenuItem>,
) -> Result<actix_web::HttpResponse, AppError> {
    // Check if user has chef or admin role
    require_chef_or_admin(&req, &pool).await?;
    menu_item::update_menu_item(req, pool, path, data).await
}

async fn delete_menu_item_wrapper(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    path: web::Path<(Uuid, Uuid)>,
) -> Result<actix_web::HttpResponse, AppError> {
    // Check if user has chef or admin role
    require_chef_or_admin(&req, &pool).await?;
    menu_item::delete_menu_item(req, pool, path).await
}

async fn get_chef_bookings_wrapper(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    path: web::Path<Uuid>,
) -> Result<actix_web::HttpResponse, AppError> {
    booking::get_chef_bookings(req, pool, path).await
}

async fn update_booking_wrapper(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    path: web::Path<Uuid>,
    data: web::Json<crate::models::UpdateBooking>,
) -> Result<actix_web::HttpResponse, AppError> {
    booking::update_booking(req, pool, path, data).await
}
