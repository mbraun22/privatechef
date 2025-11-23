// Library crate for privatechefspace-backend
// This allows the code to be used in both the binary (main.rs) and tests

pub mod config;
pub mod errors;
pub mod utils;
pub mod db;
pub mod cache;
pub mod models;
pub mod handlers;
pub mod middleware;
pub mod routes;
pub mod templates;

pub use config::Config;
pub use errors::AppError;
pub use db::create_pool;
pub use cache::RedisClient;
pub use routes::{configure_auth, configure_api, configure_web};
pub use middleware::cors::configure_cors;

