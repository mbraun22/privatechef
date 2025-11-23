use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Chef {
    pub id: Uuid,
    pub user_id: Uuid,
    pub business_name: Option<String>,
    pub chef_name: String,
    pub bio: Option<String>,
    pub cuisine_types: Option<Vec<String>>,
    pub location: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
    pub profile_image_url: Option<String>,
    pub cover_image_url: Option<String>,
    pub hourly_rate: Option<f64>,
    pub minimum_hours: i32,
    pub travel_radius: Option<i32>,
    pub is_active: bool,
    pub slug: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateChef {
    pub business_name: Option<String>,
    pub chef_name: String,
    pub bio: Option<String>,
    pub cuisine_types: Option<Vec<String>>,
    pub location: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
    pub profile_image_url: Option<String>,
    pub cover_image_url: Option<String>,
    pub hourly_rate: Option<f64>,
    pub minimum_hours: Option<i32>,
    pub travel_radius: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateChef {
    pub business_name: Option<String>,
    pub chef_name: Option<String>,
    pub bio: Option<String>,
    pub cuisine_types: Option<Vec<String>>,
    pub location: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
    pub profile_image_url: Option<String>,
    pub cover_image_url: Option<String>,
    pub hourly_rate: Option<f64>,
    pub minimum_hours: Option<i32>,
    pub travel_radius: Option<i32>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct ChefPublicProfile {
    pub id: Uuid,
    pub business_name: Option<String>,
    pub chef_name: String,
    pub bio: Option<String>,
    pub cuisine_types: Option<Vec<String>>,
    pub location: Option<String>,
    pub profile_image_url: Option<String>,
    pub cover_image_url: Option<String>,
    pub hourly_rate: Option<f64>,
    pub minimum_hours: i32,
    pub slug: Option<String>,
    pub featured_menu_items: Vec<MenuItemPublic>,
}

#[derive(Debug, Serialize)]
pub struct MenuItemPublic {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub course_type: Option<String>,
    pub image_url: Option<String>,
}

impl From<Chef> for ChefPublicProfile {
    fn from(chef: Chef) -> Self {
        ChefPublicProfile {
            id: chef.id,
            business_name: chef.business_name,
            chef_name: chef.chef_name,
            bio: chef.bio,
            cuisine_types: chef.cuisine_types,
            location: chef.location,
            profile_image_url: chef.profile_image_url,
            cover_image_url: chef.cover_image_url,
            hourly_rate: chef.hourly_rate,
            minimum_hours: chef.minimum_hours,
            slug: chef.slug,
            featured_menu_items: vec![], // Will be populated separately
        }
    }
}

