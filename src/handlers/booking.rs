use actix_web::{web, HttpResponse};
use uuid::Uuid;
use chrono::{NaiveDate, NaiveTime};

use crate::db::DbPool;
use crate::models::{Booking, CreateBooking, UpdateBooking, BookingAvailability};
use crate::errors::AppError;
use crate::middleware::auth::extract_user_id;
use actix_web::HttpRequest;

pub async fn create_booking(
    pool: web::Data<DbPool>,
    chef_id: web::Path<Uuid>,
    data: web::Json<CreateBooking>,
) -> Result<HttpResponse, AppError> {
    // Get chef info for pricing
    let chef: Option<(Option<f64>, i32)> = sqlx::query_as(
        "SELECT hourly_rate, minimum_hours FROM chefs WHERE id = $1 AND is_active = true"
    )
    .bind(*chef_id)
    .fetch_optional(pool.get_ref())
    .await?;
    
    let (hourly_rate, minimum_hours) = chef
        .ok_or_else(|| AppError::NotFound("Chef not found".to_string()))?;

    let duration_hours = data.duration_hours as i32;
    let duration = minimum_hours.max(duration_hours);
    let hourly_rate = hourly_rate.unwrap_or(100.0);
    let total_price = hourly_rate * duration as f64 * data.number_of_guests as f64;

    // Check for conflicts - simplified check
    let conflict: Option<(uuid::Uuid,)> = sqlx::query_as(
        r#"
        SELECT id FROM bookings
        WHERE chef_id = $1
        AND event_date = $2
        AND event_time = $3
        AND status IN ('pending', 'confirmed')
        "#
    )
    .bind(*chef_id)
    .bind(data.event_date)
    .bind(data.event_time)
    .fetch_optional(pool.get_ref())
    .await?;

    if conflict.is_some() {
        return Err(AppError::ValidationError("Time slot is already booked".to_string()));
    }

    let booking = sqlx::query_as::<_, Booking>(
        r#"
        INSERT INTO bookings (
            chef_id, customer_id, menu_id, customer_name, customer_email,
            customer_phone, event_date, event_time, duration_hours,
            number_of_guests, location_address, special_requests,
            total_price, status, payment_status, created_at, updated_at
        )
        VALUES ($1, NULL, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, 'pending', 'pending', NOW(), NOW())
        RETURNING *
        "#
    )
    .bind(*chef_id)
    .bind(&data.menu_id)
    .bind(&data.customer_name)
    .bind(&data.customer_email)
    .bind(&data.customer_phone)
    .bind(data.event_date)
    .bind(data.event_time)
    .bind(data.duration_hours)
    .bind(data.number_of_guests)
    .bind(&data.location_address)
    .bind(&data.special_requests)
    .bind(total_price)
    .fetch_one(pool.get_ref())
    .await?;

    Ok(HttpResponse::Created().json(booking))
}

pub async fn get_chef_availability(
    pool: web::Data<DbPool>,
    chef_id: web::Path<Uuid>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> Result<HttpResponse, AppError> {
    let start_date = query.get("start_date")
        .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
        .unwrap_or_else(|| chrono::Utc::now().date_naive());
    
    let end_date = query.get("end_date")
        .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
        .unwrap_or_else(|| start_date + chrono::Duration::days(30));

    // Get existing bookings
    #[derive(sqlx::FromRow)]
    struct BookingTime {
        event_date: chrono::NaiveDate,
        event_time: chrono::NaiveTime,
        duration_hours: f64,
    }
    
    let bookings: Vec<BookingTime> = sqlx::query_as(
        r#"
        SELECT event_date, event_time, duration_hours
        FROM bookings
        WHERE chef_id = $1
        AND event_date BETWEEN $2 AND $3
        AND status IN ('pending', 'confirmed')
        "#
    )
    .bind(*chef_id)
    .bind(start_date)
    .bind(end_date)
    .fetch_all(pool.get_ref())
    .await?;

    // Generate availability for each date
    let mut availability = Vec::new();
    let mut current_date = start_date;
    
    while current_date <= end_date {
        let booked_times: Vec<NaiveTime> = bookings
            .iter()
            .filter(|b| b.event_date == current_date)
            .map(|b| b.event_time)
            .collect();

        // Default available times (10 AM, 2 PM, 6 PM)
        let default_times = vec!["10:00", "14:00", "18:00"];
        let available_times: Vec<String> = default_times
            .into_iter()
            .filter(|time_str| {
                if let Ok(time) = NaiveTime::parse_from_str(time_str, "%H:%M") {
                    !booked_times.contains(&time)
                } else {
                    false
                }
            })
            .map(|s| s.to_string())
            .collect();

        availability.push(BookingAvailability {
            date: current_date,
            available: !available_times.is_empty(),
            available_times,
        });

        current_date = current_date.succ_opt().unwrap_or(current_date);
    }

    Ok(HttpResponse::Ok().json(availability))
}

pub async fn get_chef_bookings(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    chef_id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let user_id = extract_user_id(&req)?;

    // Verify ownership
    let _chef: Option<(uuid::Uuid,)> = sqlx::query_as(
        "SELECT id FROM chefs WHERE id = $1 AND user_id = $2"
    )
    .bind(*chef_id)
    .bind(user_id)
    .fetch_optional(pool.get_ref())
    .await?;
    
    if _chef.is_none() {
        return Err(AppError::Unauthorized("Not authorized".to_string()));
    }

    let bookings = sqlx::query_as::<_, Booking>(
        "SELECT * FROM bookings WHERE chef_id = $1 ORDER BY event_date DESC, event_time DESC"
    )
    .bind(*chef_id)
    .fetch_all(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(bookings))
}

pub async fn update_booking(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    booking_id: web::Path<Uuid>,
    data: web::Json<UpdateBooking>,
) -> Result<HttpResponse, AppError> {
    let user_id = extract_user_id(&req)?;

    // Verify ownership
    let _chef: Option<(uuid::Uuid,)> = sqlx::query_as(
        "SELECT c.id FROM chefs c
         INNER JOIN bookings b ON b.chef_id = c.id
         WHERE c.user_id = $1 AND b.id = $2"
    )
    .bind(user_id)
    .bind(*booking_id)
    .fetch_optional(pool.get_ref())
    .await?;
    
    if _chef.is_none() {
        return Err(AppError::Unauthorized("Not authorized".to_string()));
    }

    let booking = sqlx::query_as::<_, Booking>(
        r#"
        UPDATE bookings
        SET status = COALESCE($1, status),
            payment_status = COALESCE($2, payment_status),
            updated_at = NOW()
        WHERE id = $3
        RETURNING *
        "#
    )
    .bind(&data.status)
    .bind(&data.payment_status)
    .bind(*booking_id)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Booking not found".to_string()))?;

    Ok(HttpResponse::Ok().json(booking))
}

