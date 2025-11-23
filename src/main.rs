// Use the library crate
use privatechefspace_backend::*;

use actix_web::{web, App, HttpServer};
use tracing_subscriber;

use routes::{configure_auth, configure_api, configure_web};
use middleware::cors::configure_cors;
use actix_files::Files;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Load configuration
    let config = Config::from_env()
        .expect("Failed to load configuration");

    // Create database pool
    let pool = create_pool(&config.database)
        .await
        .expect("Failed to create database pool");

    // Create Redis client
    let redis_client = RedisClient::new(&config.redis)
        .await
        .expect("Failed to create Redis client");

    tracing::info!(
        "Starting server on {}:{}",
        config.server.host,
        config.server.port
    );

    // Start HTTP server
    let host = config.server.host.clone();
    let port = config.server.port;
    let pool_data = web::Data::new(pool);
    let config_data = web::Data::new(config);
    let redis_data = web::Data::new(redis_client);
    
    HttpServer::new(move || {
        App::new()
            .app_data(pool_data.clone())
            .app_data(config_data.clone())
            .app_data(redis_data.clone())
            .wrap(configure_cors())
            // Serve static files - must be before other routes
            .service(
                Files::new("/static", "./static")
                    .show_files_listing()
                    .prefer_utf8(true)
            )
            // Configure routes
            .configure(|cfg| {
                configure_web(cfg, pool_data.clone(), config_data.clone(), redis_data.clone());
                configure_auth(cfg, pool_data.clone(), config_data.clone());
                configure_api(cfg, pool_data.clone(), config_data.clone());
            })
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}

