use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Menu {
    pub id: Uuid,
    pub chef_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub price_per_person: Option<f64>,
    pub minimum_guests: i32,
    pub cuisine_type: Option<String>,
    pub dietary_options: Option<Vec<String>>,
    pub duration_hours: Option<f64>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateMenu {
    pub name: String,
    pub description: Option<String>,
    pub price_per_person: Option<f64>,
    pub minimum_guests: Option<i32>,
    pub cuisine_type: Option<String>,
    pub dietary_options: Option<Vec<String>>,
    pub duration_hours: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMenu {
    pub name: Option<String>,
    pub description: Option<String>,
    pub price_per_person: Option<f64>,
    pub minimum_guests: Option<i32>,
    pub cuisine_type: Option<String>,
    pub dietary_options: Option<Vec<String>>,
    pub duration_hours: Option<f64>,
    pub is_active: Option<bool>,
}

