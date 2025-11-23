use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// User roles in the system
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Admin,
    Mod,
    Chef,
    Diner,
}

// Custom serialization to use lowercase strings
impl Serialize for Role {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

// Custom deserialization to handle case-insensitive strings
impl<'de> Deserialize<'de> for Role {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

// Implement sqlx::Type for Role to work with VARCHAR
impl sqlx::Type<sqlx::Postgres> for Role {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        // Use VARCHAR type info instead of TEXT
        sqlx::postgres::PgTypeInfo::with_name("VARCHAR")
    }
}

// Implement sqlx::Decode for Role
impl<'r> sqlx::Decode<'r, sqlx::Postgres> for Role {
    fn decode(
        value: sqlx::postgres::PgValueRef<'r>,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <&str as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        // Handle case-insensitive parsing and default to diner if invalid
        match s.to_lowercase().trim() {
            "admin" => Ok(Role::Admin),
            "mod" => Ok(Role::Mod),
            "chef" => Ok(Role::Chef),
            "diner" => Ok(Role::Diner),
            _ => {
                // Log warning but default to diner for safety
                tracing::warn!("Invalid role value in database: '{}', defaulting to diner", s);
                Ok(Role::Diner)
            }
        }
    }
}

// Implement sqlx::Encode for Role
impl<'q> sqlx::Encode<'q, sqlx::Postgres> for Role {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> sqlx::encode::IsNull {
        let s = self.to_string();
        <&str as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&s.as_str(), buf)
    }
}

impl Default for Role {
    fn default() -> Self {
        Role::Diner
    }
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::Admin => write!(f, "admin"),
            Role::Mod => write!(f, "mod"),
            Role::Chef => write!(f, "chef"),
            Role::Diner => write!(f, "diner"),
        }
    }
}

impl std::str::FromStr for Role {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "admin" => Ok(Role::Admin),
            "mod" => Ok(Role::Mod),
            "chef" => Ok(Role::Chef),
            "diner" => Ok(Role::Diner),
            _ => Err(format!("Invalid role: {}", s)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub role: Role,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateUser {
    pub email: String,
    pub password: String,
    #[serde(default)]
    pub role: Option<Role>,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub role: Role,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        UserResponse {
            id: user.id,
            email: user.email,
            role: user.role,
            created_at: user.created_at,
        }
    }
}

