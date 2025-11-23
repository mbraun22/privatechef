use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Example {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateExample {
    pub title: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateExample {
    pub title: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ExampleResponse {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Example> for ExampleResponse {
    fn from(example: Example) -> Self {
        ExampleResponse {
            id: example.id,
            title: example.title,
            description: example.description,
            user_id: example.user_id,
            created_at: example.created_at,
            updated_at: example.updated_at,
        }
    }
}

