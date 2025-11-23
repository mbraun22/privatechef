use actix_web::web;
use crate::handlers::web as web_handlers;
use crate::config::Config;
use crate::db::DbPool;
use crate::cache::RedisClient;

pub fn configure_web(
    cfg: &mut web::ServiceConfig, 
    _pool: web::Data<DbPool>, 
    _config: web::Data<Config>,
    _redis: web::Data<RedisClient>,
) {
    cfg
        // Public web routes
        .route("/", web::get().to(web_handlers::home))
        .route("/login", web::get().to(web_handlers::login_page))
        .route("/login", web::post().to(web_handlers::handle_login))
        .route("/register", web::post().to(web_handlers::handle_register))
        .route("/logout", web::get().to(web_handlers::handle_logout))
        // Protected web routes (session check is done in handlers)
        .route("/dashboard", web::get().to(web_handlers::dashboard_page))
        .route("/chef-dashboard", web::get().to(web_handlers::chef_dashboard_page))
        .route("/chef-dashboard/create-chef", web::post().to(web_handlers::handle_create_chef))
        .route("/chef-dashboard/create-menu", web::post().to(web_handlers::handle_create_menu))
        .route("/chef-dashboard/create-menu-item", web::post().to(web_handlers::handle_create_menu_item));
}

