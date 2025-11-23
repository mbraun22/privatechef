use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Booking {
    pub id: Uuid,
    pub chef_id: Uuid,
    pub customer_id: Option<Uuid>,
    pub menu_id: Option<Uuid>,
    pub customer_name: String,
    pub customer_email: String,
    pub customer_phone: Option<String>,
    pub event_date: NaiveDate,
    pub event_time: NaiveTime,
    pub duration_hours: f64,
    pub number_of_guests: i32,
    pub location_address: String,
    pub special_requests: Option<String>,
    pub total_price: f64,
    pub status: String,
    pub payment_status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateBooking {
    pub menu_id: Option<Uuid>,
    pub customer_name: String,
    pub customer_email: String,
    pub customer_phone: Option<String>,
    pub event_date: NaiveDate,
    pub event_time: NaiveTime,
    pub duration_hours: f64,
    pub number_of_guests: i32,
    pub location_address: String,
    pub special_requests: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateBooking {
    pub status: Option<String>,
    pub payment_status: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BookingAvailability {
    pub date: NaiveDate,
    pub available: bool,
    pub available_times: Vec<String>, // e.g., ["10:00", "14:00", "18:00"]
}

