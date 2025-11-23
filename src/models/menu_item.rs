use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct MenuItem {
    pub id: Uuid,
    pub menu_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub course_type: Option<String>,
    pub image_url: Option<String>,
    pub is_featured: bool,
    pub display_order: i32,
    pub quantity: Option<i32>, // Number of plates/servings for this item
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateMenuItem {
    pub name: String,
    pub description: Option<String>,
    pub course_type: Option<String>,
    pub image_url: Option<String>,
    pub is_featured: Option<bool>,
    pub display_order: Option<i32>,
    pub quantity: Option<i32>, // Number of plates/servings
}

#[derive(Debug, Deserialize)]
pub struct UpdateMenuItem {
    pub name: Option<String>,
    pub description: Option<String>,
    pub course_type: Option<String>,
    pub image_url: Option<String>,
    pub is_featured: Option<bool>,
    pub display_order: Option<i32>,
    pub quantity: Option<i32>, // Number of plates/servings
}

