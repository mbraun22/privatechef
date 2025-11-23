use actix_cors::Cors;

pub fn configure_cors() -> Cors {
    // For same-origin requests (web pages), CORS shouldn't interfere
    // But we need to allow credentials for cookies to work
    Cors::default()
        .allow_any_origin()
        .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
        .allowed_headers(vec![
            actix_web::http::header::AUTHORIZATION,
            actix_web::http::header::CONTENT_TYPE,
        ])
        .max_age(3600)
}

