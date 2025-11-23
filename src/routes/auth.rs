use actix_web::web;
use crate::handlers::auth;
use crate::config::Config;
use crate::db::DbPool;

pub fn configure_auth(cfg: &mut web::ServiceConfig, pool: web::Data<DbPool>, config: web::Data<Config>) {
    cfg.service(
        web::scope("/api/auth")
            .app_data(pool.clone())
            .app_data(config.clone())
            .route("/register", web::post().to(auth::register))
            .route("/login", web::post().to(auth::login))
    );
}

