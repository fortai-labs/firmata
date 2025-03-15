use anyhow::Result;
use sqlx::migrate::Migrator;
use sqlx::postgres::PgPoolOptions;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file if it exists
    dotenv::dotenv().ok();
    
    // Get database URL from environment
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    
    println!("Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    
    println!("Running migrations...");
    let migrations_path = Path::new("./migrations");
    Migrator::new(migrations_path)
        .await?
        .run(&pool)
        .await?;
    
    println!("Migrations completed successfully!");
    
    Ok(())
} 