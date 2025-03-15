use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::Serialize;
use sqlx::PgPool;

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    status: String,
    version: String,
    database: String,
}

pub async fn health_check(
    State(db_pool): State<PgPool>,
) -> Result<Json<HealthResponse>, StatusCode> {
    // Check database connection
    let db_status = match sqlx::query("SELECT 1").execute(&db_pool).await {
        Ok(_) => "up",
        Err(_) => "down",
    };
    
    // Get version from Cargo.toml
    let version = env!("CARGO_PKG_VERSION");
    
    let response = HealthResponse {
        status: "ok".to_string(),
        version: version.to_string(),
        database: db_status.to_string(),
    };
    
    Ok(Json(response))
} 