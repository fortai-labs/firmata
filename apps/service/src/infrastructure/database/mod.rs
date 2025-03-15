use sqlx::postgres::{PgPool, PgPoolOptions};
use anyhow::Result;

use crate::config::settings::Database;

pub async fn establish_connection(config: &Database) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .connect(&config.url)
        .await?;
    
    Ok(pool)
} 