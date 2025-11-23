// Simple script to make a user an admin
// Run with: cargo run --bin make_admin -- mattbraun@example.com

use sqlx::PgPool;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    
    let email_pattern = env::args()
        .nth(1)
        .unwrap_or_else(|| "mattbraun".to_string());
    
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    
    let pool = PgPool::connect(&database_url).await?;
    
    // Update user role to admin
    let result = sqlx::query(
        "UPDATE users SET role = 'admin' WHERE email LIKE $1 OR email ILIKE $2"
    )
    .bind(format!("%{}%", email_pattern))
    .bind(format!("%{}%", email_pattern))
    .execute(&pool)
    .await?;
    
    println!("Updated {} user(s) to admin role", result.rows_affected());
    
    // Verify the update
    let users = sqlx::query_as::<_, (uuid::Uuid, String, String)>(
        "SELECT id, email, role FROM users WHERE role = 'admin' ORDER BY email"
    )
    .fetch_all(&pool)
    .await?;
    
    println!("\nCurrent admin users:");
    for (id, email, role) in users {
        println!("  {} - {} ({})", email, role, id);
    }
    
    Ok(())
}

