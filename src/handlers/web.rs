use actix_web::{web, HttpResponse, HttpRequest};
use askama::Template;
use crate::db::DbPool;
use crate::config::Config;
use crate::cache::RedisClient;
use crate::cache::session::SessionData;
use crate::templates::{HomeTemplate, LoginTemplate, DashboardTemplate, ChefDashboardTemplate};
use crate::models::{UserResponse, User};
use crate::middleware::auth::extract_user_id_from_session;
use crate::errors::AppError;
use actix_web::cookie::{Cookie, SameSite, time::Duration as CookieDuration};
use bcrypt::verify;
use uuid::Uuid;
use slug::slugify;

/// Serve home page
pub async fn home(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    redis: web::Data<RedisClient>,
) -> Result<HttpResponse, AppError> {
    // Try to get user from session if available
    let user = get_user_from_request(&req, &pool, &redis).await.ok();
    
    let template = HomeTemplate { user };
    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(template.render().map_err(|e| AppError::InternalError(e.to_string()))?))
}

/// Serve login page
pub async fn login_page(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    redis: web::Data<RedisClient>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> Result<HttpResponse, AppError> {
    let is_register = query.get("register").map(|s| s == "true").unwrap_or(false);
    let user = get_user_from_request(&req, &pool, &redis).await.ok();
    
    let template = LoginTemplate {
        user,
        is_register,
        error: None,
        loading: false,
    };
    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(template.render().map_err(|e| AppError::InternalError(e.to_string()))?))
}

/// Serve dashboard page
pub async fn dashboard_page(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    redis: web::Data<RedisClient>,
) -> Result<HttpResponse, AppError> {
    // Log all cookies for debugging
    tracing::info!("Dashboard request - all cookies: {:?}", req.cookies());
    tracing::info!("Dashboard request - session_id cookie: {:?}", req.cookie("session_id"));
    
    let user_id = match extract_user_id_from_session(&req, &redis).await {
        Ok(id) => {
            tracing::info!("Successfully extracted user_id: {}", id);
            id
        }
        Err(e) => {
            tracing::warn!("Failed to extract user from session: {:?}", e);
            tracing::warn!("Redirecting to login page");
            // Redirect to login instead of returning 401
            return Ok(HttpResponse::SeeOther()
                .append_header(("Location", "/login"))
                .finish());
        }
    };
    
    let user = sqlx::query_as::<_, crate::models::User>(
        "SELECT * FROM users WHERE id = $1"
    )
    .bind(user_id)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    let template = DashboardTemplate {
        user: Some(UserResponse::from(user)),
    };
    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(template.render().map_err(|e| AppError::InternalError(e.to_string()))?))
}

/// Serve chef dashboard page
pub async fn chef_dashboard_page(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    redis: web::Data<RedisClient>,
) -> Result<HttpResponse, AppError> {
    // Log all cookies for debugging
    tracing::info!("Chef dashboard request - all cookies: {:?}", req.cookies());
    tracing::info!("Chef dashboard request - session_id cookie: {:?}", req.cookie("session_id"));
    
    // Extract user ID from session, redirect to login if not authenticated
    let user_id = match extract_user_id_from_session(&req, &redis).await {
        Ok(id) => {
            tracing::info!("Chef dashboard - Successfully extracted user_id: {}", id);
            id
        }
        Err(e) => {
            tracing::warn!("Chef dashboard - Failed to extract user from session: {:?}", e);
            tracing::warn!("Redirecting to login page");
            return Ok(HttpResponse::SeeOther()
                .append_header(("Location", "/login"))
                .finish());
        }
    };
    
    // Get user and check role
    let user = sqlx::query_as::<_, crate::models::User>(
        "SELECT * FROM users WHERE id = $1"
    )
    .bind(user_id)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
    
    // Check if user has chef or admin role
    if user.role != crate::models::Role::Chef && user.role != crate::models::Role::Admin {
        tracing::warn!("Chef dashboard - Access denied for user {} with role: {:?}", user_id, user.role);
        return Ok(HttpResponse::SeeOther()
            .append_header(("Location", "/login"))
            .finish());
    }

    // Get chef profile if exists
    let chef = sqlx::query_as::<_, crate::models::Chef>(
        "SELECT * FROM chefs WHERE user_id = $1"
    )
    .bind(user_id)
    .fetch_optional(pool.get_ref())
    .await?;

    // Get menus if chef exists
    let menus = if let Some(ref chef) = chef {
        sqlx::query_as::<_, crate::models::Menu>(
            "SELECT * FROM menus WHERE chef_id = $1 ORDER BY created_at DESC"
        )
        .bind(chef.id)
        .fetch_all(pool.get_ref())
        .await?
    } else {
        Vec::new()
    };

    // Get menu items for each menu and combine into MenuWithItems
    let mut menus_with_items = Vec::new();
    for menu in menus {
        let items = sqlx::query_as::<_, crate::models::MenuItem>(
            "SELECT * FROM menu_items WHERE menu_id = $1 ORDER BY display_order ASC, created_at ASC"
        )
        .bind(menu.id)
        .fetch_all(pool.get_ref())
        .await?;
        menus_with_items.push(crate::templates::MenuWithItems {
            menu,
            items,
        });
    }

    // Get error/success messages from query params
    let query = web::Query::<std::collections::HashMap<String, String>>::from_query(req.query_string()).ok();
    let error = query.as_ref().and_then(|q| q.get("error")).map(|s| {
        urlencoding::decode(s).map(|decoded| decoded.into_owned()).unwrap_or_else(|_| s.clone())
    });
    let success = query.as_ref().and_then(|q| q.get("success")).map(|s| {
        urlencoding::decode(s).map(|decoded| decoded.into_owned()).unwrap_or_else(|_| s.clone())
    });

    let template = ChefDashboardTemplate {
        user: Some(UserResponse::from(user)),
        chef,
        menus_with_items,
        error,
        success,
    };
    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(template.render().map_err(|e| AppError::InternalError(e.to_string()))?))
}

/// Helper to get user from request (from session)
async fn get_user_from_request(
    req: &HttpRequest,
    pool: &web::Data<DbPool>,
    redis: &web::Data<RedisClient>,
) -> Result<UserResponse, AppError> {
    // Try to get session ID from cookie
    if let Some(cookie) = req.cookie("session_id") {
        let session_id = cookie.value();
        
        // Get session data from Redis
        if let Some(session_data) = redis.get_session_data(session_id).await? {
            // Fetch user from database
            let user = sqlx::query_as::<_, User>(
                "SELECT * FROM users WHERE id = $1"
            )
            .bind(session_data.user_id)
            .fetch_optional(pool.get_ref())
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
            
            Ok(UserResponse::from(user))
        } else {
            Err(AppError::Unauthorized("Invalid or expired session".to_string()))
        }
    } else {
        Err(AppError::Unauthorized("Not authenticated".to_string()))
    }
}

/// Handle login form submission
pub async fn handle_login(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    redis: web::Data<RedisClient>,
    form: web::Form<std::collections::HashMap<String, String>>,
) -> Result<HttpResponse, AppError> {
    tracing::info!("Login attempt received");
    
    let email = form.get("email")
        .ok_or_else(|| AppError::ValidationError("Email is required".to_string()))?;
    let password = form.get("password")
        .ok_or_else(|| AppError::ValidationError("Password is required".to_string()))?;

    tracing::info!("Attempting login for email: {}", email);

    // Find user
    let user = match sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE email = $1"
    )
    .bind(email)
    .fetch_optional(pool.get_ref())
    .await? {
        Some(u) => {
            tracing::info!("User found: {}", u.id);
            u
        }
        None => {
            tracing::warn!("User not found for email: {}", email);
            return Err(AppError::Unauthorized("Invalid credentials".to_string()));
        }
    };

    // Verify password
    tracing::debug!("Verifying password");
    if !verify(password, &user.password_hash)? {
        tracing::warn!("Password verification failed for user: {}", user.id);
        return Err(AppError::Unauthorized("Invalid credentials".to_string()));
    }
    
    tracing::info!("Password verified successfully for user: {}", user.id);

    // Generate session ID
    let session_id = Uuid::new_v4().to_string();
    
    // Create session data
    let session_data = SessionData {
        user_id: user.id,
        email: user.email.clone(),
        role: user.role.to_string(),
    };
    
    // Store session in Redis (30 days expiration)
    let session_ttl = 30 * 24 * 60 * 60; // 30 days in seconds
    tracing::info!("Creating session for user {}: {}", user.id, session_id);
    redis.create_session(&session_id, &session_data, session_ttl).await?;
    tracing::info!("Session created successfully");

    // Create redirect response with session cookie
    // Use 303 See Other to ensure cookie is sent with redirect
    let cookie = Cookie::build("session_id", session_id.clone())
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(CookieDuration::seconds(session_ttl as i64))
        .finish();
    
    tracing::info!("Setting session cookie: {} in response", session_id);
    tracing::info!("Cookie value: {}, Cookie string: {}", cookie.value(), cookie.to_string());
    
    // Build response with cookie
    let mut response = HttpResponse::SeeOther();
    response.cookie(cookie);
    response.append_header(("Location", "/dashboard"));
    
    tracing::info!("Response built, cookie should be set");
    
    Ok(response.finish())
}

/// Handle register form submission
pub async fn handle_register(
    _req: HttpRequest,
    pool: web::Data<DbPool>,
    redis: web::Data<RedisClient>,
    _config: web::Data<Config>,
    form: web::Form<std::collections::HashMap<String, String>>,
) -> Result<HttpResponse, AppError> {
    let email = form.get("email")
        .ok_or_else(|| AppError::ValidationError("Email is required".to_string()))?;
    let password = form.get("password")
        .ok_or_else(|| AppError::ValidationError("Password is required".to_string()))?;
    let confirm_password = form.get("confirmPassword")
        .ok_or_else(|| AppError::ValidationError("Confirm password is required".to_string()))?;

    if password != confirm_password {
        return Err(AppError::ValidationError("Passwords don't match".to_string()));
    }

    // Create user directly (simpler than calling register handler)
    use crate::models::Role;
    use bcrypt::{hash, DEFAULT_COST};
    
    // Check if user already exists
    let existing_user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE email = $1"
    )
    .bind(&email)
    .fetch_optional(pool.get_ref())
    .await?;

    if existing_user.is_some() {
        return Err(AppError::ValidationError("User already exists".to_string()));
    }

    let role = Role::Diner;
    let password_hash = hash(password, DEFAULT_COST)?;
    let role_str = role.to_string();
    
    // Create user
    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (id, email, password_hash, role, created_at, updated_at)
        VALUES (gen_random_uuid(), $1, $2, $3::varchar, NOW(), NOW())
        RETURNING *
        "#
    )
    .bind(&email)
    .bind(&password_hash)
    .bind(&role_str)
    .fetch_one(pool.get_ref())
    .await?;

    // Generate session ID
    let session_id = Uuid::new_v4().to_string();
    
    // Create session data
    let session_data = SessionData {
        user_id: user.id,
        email: user.email.clone(),
        role: user.role.to_string(),
    };
    
    // Store session in Redis (30 days expiration)
    let session_ttl = 30 * 24 * 60 * 60; // 30 days in seconds
    tracing::info!("Creating session for user {}: {}", user.id, session_id);
    redis.create_session(&session_id, &session_data, session_ttl).await?;
    tracing::info!("Session created successfully");

    // Create redirect response with session cookie
    // Use 303 See Other to ensure cookie is sent with redirect
    let cookie = Cookie::build("session_id", session_id.clone())
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(CookieDuration::seconds(session_ttl as i64))
        .finish();
    
    tracing::info!("Setting session cookie: {} in response", session_id);
    
    // Build response with cookie
    let mut response = HttpResponse::SeeOther();
    response.cookie(cookie);
    response.append_header(("Location", "/dashboard"));
    
    // Log to verify cookie is set
    tracing::info!("Response built, cookie should be set");
    
    Ok(response.finish())
}

/// Handle logout
pub async fn handle_logout(
    req: HttpRequest,
    redis: web::Data<RedisClient>,
) -> Result<HttpResponse, AppError> {
    // Get session ID from cookie
    if let Some(cookie) = req.cookie("session_id") {
        let session_id = cookie.value();
        // Delete session from Redis
        redis.destroy_session(session_id).await?;
    }

    // Create redirect response that removes the session cookie
    let mut response = HttpResponse::SeeOther();
    response.append_header(("Location", "/"));
    
    // Remove session cookie by setting it with max_age 0
    response.cookie(
        Cookie::build("session_id", "")
            .path("/")
            .http_only(true)
            .same_site(SameSite::Lax)
            .max_age(CookieDuration::seconds(0))
            .finish(),
    );
    
    Ok(response.finish())
}

/// Handle create chef profile form submission
pub async fn handle_create_chef(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    redis: web::Data<RedisClient>,
    form: web::Form<std::collections::HashMap<String, String>>,
) -> Result<HttpResponse, AppError> {
    tracing::info!("Create chef profile form submission received");
    
    // Extract user ID from session
    let user_id = extract_user_id_from_session(&req, &redis).await?;
    
    // Get user and check role
    let user = sqlx::query_as::<_, crate::models::User>(
        "SELECT * FROM users WHERE id = $1"
    )
    .bind(user_id)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
    
    // Check if user has chef or admin role
    if user.role != crate::models::Role::Chef && user.role != crate::models::Role::Admin {
        return Err(AppError::Unauthorized("Chef or admin role required".to_string()));
    }
    
    // Check if chef profile already exists
    let existing = sqlx::query_as::<_, crate::models::Chef>(
        "SELECT * FROM chefs WHERE user_id = $1"
    )
    .bind(user_id)
    .fetch_optional(pool.get_ref())
    .await?;
    
    if existing.is_some() {
        let error_msg = urlencoding::encode("Chef profile already exists");
        return Ok(HttpResponse::SeeOther()
            .append_header(("Location", format!("/chef-dashboard?error={}", error_msg)))
            .finish());
    }
    
    // Extract form data
    let chef_name = match form.get("chef_name") {
        Some(name) if !name.trim().is_empty() => name.trim(),
        _ => {
            let error_msg = urlencoding::encode("Chef name is required");
            return Ok(HttpResponse::SeeOther()
                .append_header(("Location", format!("/chef-dashboard?error={}", error_msg)))
                .finish());
        }
    };
    let business_name = form.get("business_name").and_then(|s| {
        let trimmed = s.trim();
        if trimmed.is_empty() { None } else { Some(trimmed.to_string()) }
    });
    let bio = form.get("bio").and_then(|s| {
        let trimmed = s.trim();
        if trimmed.is_empty() { None } else { Some(trimmed.to_string()) }
    });
    let location = form.get("location").and_then(|s| {
        let trimmed = s.trim();
        if trimmed.is_empty() { None } else { Some(trimmed.to_string()) }
    });
    let hourly_rate = form.get("hourly_rate")
        .and_then(|s| s.trim().parse::<f64>().ok());
    let cuisine_types = form.get("cuisine_types")
        .map(|s| s.split(',').map(|t| t.trim().to_string()).filter(|t| !t.is_empty()).collect::<Vec<String>>())
        .filter(|v| !v.is_empty());
    
    // Generate slug from chef name
    let slug = Some(slugify(chef_name));
    
    // Create chef profile
    match sqlx::query_as::<_, crate::models::Chef>(
        r#"
        INSERT INTO chefs (
            user_id, business_name, chef_name, bio, cuisine_types, location,
            hourly_rate, minimum_hours, slug, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW(), NOW())
        RETURNING *
        "#
    )
    .bind(user_id)
    .bind(&business_name)
    .bind(chef_name)
    .bind(&bio)
    .bind(&cuisine_types)
    .bind(&location)
    .bind(&hourly_rate)
    .bind(2) // default minimum_hours
    .bind(&slug)
    .fetch_one(pool.get_ref())
    .await {
        Ok(_chef) => {
            tracing::info!("Chef profile created successfully for user: {}", user_id);
            let success_msg = urlencoding::encode("Chef profile created successfully!");
            Ok(HttpResponse::SeeOther()
                .append_header(("Location", format!("/chef-dashboard?success={}", success_msg)))
                .finish())
        }
        Err(e) => {
            tracing::error!("Failed to create chef profile: {:?}", e);
            let error_msg = urlencoding::encode("Failed to create chef profile. Please try again.");
            Ok(HttpResponse::SeeOther()
                .append_header(("Location", format!("/chef-dashboard?error={}", error_msg)))
                .finish())
        }
    }
}

/// Handle create menu form submission
pub async fn handle_create_menu(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    redis: web::Data<RedisClient>,
    form: web::Form<std::collections::HashMap<String, String>>,
) -> Result<HttpResponse, AppError> {
    tracing::info!("Create menu form submission received");
    
    // Extract user ID from session
    let user_id = extract_user_id_from_session(&req, &redis).await?;
    
    // Get user and check role
    let user = sqlx::query_as::<_, crate::models::User>(
        "SELECT * FROM users WHERE id = $1"
    )
    .bind(user_id)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
    
    // Check if user has chef or admin role
    if user.role != crate::models::Role::Chef && user.role != crate::models::Role::Admin {
        return Err(AppError::Unauthorized("Chef or admin role required".to_string()));
    }
    
    // Get chef profile
    let chef: Option<(uuid::Uuid,)> = sqlx::query_as(
        "SELECT id FROM chefs WHERE user_id = $1"
    )
    .bind(user_id)
    .fetch_optional(pool.get_ref())
    .await?;
    
    let chef_id = chef
        .map(|(id,)| id)
        .ok_or_else(|| AppError::Unauthorized("Chef profile not found. Please create a chef profile first.".to_string()))?;
    
    // Extract form data
    let name = match form.get("name") {
        Some(n) if !n.trim().is_empty() => n.trim(),
        _ => {
            let error_msg = urlencoding::encode("Menu name is required");
            return Ok(HttpResponse::SeeOther()
                .append_header(("Location", format!("/chef-dashboard?error={}", error_msg)))
                .finish());
        }
    };
    let description = form.get("description").and_then(|s| {
        let trimmed = s.trim();
        if trimmed.is_empty() { None } else { Some(trimmed.to_string()) }
    });
    let price_per_person = form.get("price_per_person")
        .and_then(|s| s.trim().parse::<f64>().ok());
    let minimum_guests = form.get("minimum_guests")
        .and_then(|s| s.trim().parse::<i32>().ok())
        .unwrap_or(2);
    let cuisine_type = form.get("cuisine_type").and_then(|s| {
        let trimmed = s.trim();
        if trimmed.is_empty() { None } else { Some(trimmed.to_string()) }
    });
    let duration_hours = form.get("duration_hours")
        .and_then(|s| s.trim().parse::<f64>().ok());
    
    // Create menu
    match sqlx::query_as::<_, crate::models::Menu>(
        r#"
        INSERT INTO menus (
            chef_id, name, description, price_per_person, minimum_guests,
            cuisine_type, duration_hours, is_active, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), NOW())
        RETURNING *
        "#
    )
    .bind(chef_id)
    .bind(name)
    .bind(&description)
    .bind(&price_per_person)
    .bind(minimum_guests)
    .bind(&cuisine_type)
    .bind(&duration_hours)
    .bind(true) // is_active
    .fetch_one(pool.get_ref())
    .await {
        Ok(_menu) => {
            tracing::info!("Menu created successfully for chef: {}", chef_id);
            let success_msg = urlencoding::encode("Menu created successfully!");
            Ok(HttpResponse::SeeOther()
                .append_header(("Location", format!("/chef-dashboard?success={}", success_msg)))
                .finish())
        }
        Err(e) => {
            tracing::error!("Failed to create menu: {:?}", e);
            let error_msg = urlencoding::encode("Failed to create menu. Please try again.");
            Ok(HttpResponse::SeeOther()
                .append_header(("Location", format!("/chef-dashboard?error={}", error_msg)))
                .finish())
        }
    }
}

/// Handle create menu item form submission
pub async fn handle_create_menu_item(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    redis: web::Data<RedisClient>,
    form: web::Form<std::collections::HashMap<String, String>>,
) -> Result<HttpResponse, AppError> {
    tracing::info!("Create menu item form submission received");
    
    // Extract user ID from session
    let user_id = extract_user_id_from_session(&req, &redis).await?;
    
    // Get user and check role
    let user = sqlx::query_as::<_, crate::models::User>(
        "SELECT * FROM users WHERE id = $1"
    )
    .bind(user_id)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
    
    // Check if user has chef or admin role
    if user.role != crate::models::Role::Chef && user.role != crate::models::Role::Admin {
        return Err(AppError::Unauthorized("Chef or admin role required".to_string()));
    }
    
    // Extract menu_id from form
    let menu_id_str = form.get("menu_id")
        .ok_or_else(|| AppError::ValidationError("Menu ID is required".to_string()))?;
    let menu_id = menu_id_str.parse::<uuid::Uuid>()
        .map_err(|_| AppError::ValidationError("Invalid menu ID".to_string()))?;
    
    // Verify ownership
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
        let error_msg = urlencoding::encode("Not authorized to add items to this menu");
        return Ok(HttpResponse::SeeOther()
            .append_header(("Location", format!("/chef-dashboard?error={}", error_msg)))
            .finish());
    }
    
    // Extract form data
    let name = match form.get("name") {
        Some(n) if !n.trim().is_empty() => n.trim(),
        _ => {
            let error_msg = urlencoding::encode("Item name is required");
            return Ok(HttpResponse::SeeOther()
                .append_header(("Location", format!("/chef-dashboard?error={}", error_msg)))
                .finish());
        }
    };
    let description = form.get("description").and_then(|s| {
        let trimmed = s.trim();
        if trimmed.is_empty() { None } else { Some(trimmed.to_string()) }
    });
    let course_type = form.get("course_type").and_then(|s| {
        let trimmed = s.trim();
        if trimmed.is_empty() { None } else { Some(trimmed.to_string()) }
    });
    let quantity = form.get("quantity")
        .and_then(|s| s.trim().parse::<i32>().ok())
        .filter(|&q| q > 0); // Only accept positive quantities
    let is_featured = form.get("is_featured").map(|s| s == "on" || s == "true").unwrap_or(false);
    
    // Create menu item
    match sqlx::query_as::<_, crate::models::MenuItem>(
        r#"
        INSERT INTO menu_items (
            menu_id, name, description, course_type, quantity,
            is_featured, display_order, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW())
        RETURNING *
        "#
    )
    .bind(menu_id)
    .bind(name)
    .bind(&description)
    .bind(&course_type)
    .bind(&quantity)
    .bind(is_featured)
    .bind(0) // display_order
    .fetch_one(pool.get_ref())
    .await {
        Ok(_item) => {
            tracing::info!("Menu item created successfully for menu: {}", menu_id);
            let success_msg = urlencoding::encode("Menu item added successfully!");
            Ok(HttpResponse::SeeOther()
                .append_header(("Location", format!("/chef-dashboard?success={}", success_msg)))
                .finish())
        }
        Err(e) => {
            tracing::error!("Failed to create menu item: {:?}", e);
            let error_msg = urlencoding::encode("Failed to create menu item. Please try again.");
            Ok(HttpResponse::SeeOther()
                .append_header(("Location", format!("/chef-dashboard?error={}", error_msg)))
                .finish())
        }
    }
}

