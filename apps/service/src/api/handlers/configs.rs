use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;
use tracing::{info, error, debug, instrument};

use crate::application::scraper::service::ScraperService;
use crate::domain::scraper_config::ScraperConfig;
use crate::utils::error::AppError;
use crate::api::routes::AppState;

#[derive(Debug, Deserialize)]
pub struct CreateConfigRequest {
    name: String,
    description: Option<String>,
    base_url: String,
    include_patterns: Vec<String>,
    exclude_patterns: Vec<String>,
    max_depth: i32,
    max_pages_per_job: Option<i32>,
    respect_robots_txt: Option<bool>,
    user_agent: Option<String>,
    request_delay_ms: Option<i32>,
    max_concurrent_requests: Option<i32>,
    schedule: Option<String>,
    headers: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct ConfigResponse {
    config: ScraperConfig,
    _links: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct ListConfigsQuery {
    limit: Option<i64>,
    offset: Option<i64>,
}

#[instrument(skip(state, payload), fields(config_name = %payload.name, base_url = %payload.base_url))]
pub async fn create_config(
    State(state): State<AppState>,
    Json(payload): Json<CreateConfigRequest>,
) -> Result<(StatusCode, Json<ConfigResponse>), AppError> {
    info!("Creating new scraper config: {}", payload.name);
    
    // Create a new config from the request
    let mut config = ScraperConfig::new(
        payload.name,
        payload.base_url,
        payload.include_patterns,
        payload.exclude_patterns,
        payload.max_depth,
    );
    
    // Set optional fields
    config.description = payload.description;
    config.max_pages_per_job = Some(payload.max_pages_per_job.unwrap_or(1000));
    config.respect_robots_txt = payload.respect_robots_txt.unwrap_or(true);
    config.user_agent = payload.user_agent.unwrap_or_else(|| "FortaiBot/1.0".to_string());
    config.request_delay_ms = payload.request_delay_ms.unwrap_or(1000);
    config.max_concurrent_requests = payload.max_concurrent_requests.unwrap_or(5);
    config.schedule = payload.schedule;
    config.headers = payload.headers.unwrap_or_else(|| serde_json::json!({}));
    config.active = true;
    
    debug!("Inserting config into database with id: {}", config.id);
    
    // Insert into database
    let config_id = sqlx::query!(
        r#"
        INSERT INTO scraper_configs (
            id, name, description, base_url, include_patterns, exclude_patterns,
            max_depth, max_pages_per_job, respect_robots_txt, user_agent,
            request_delay_ms, max_concurrent_requests, schedule, headers,
            created_at, updated_at, active
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17
        )
        RETURNING id
        "#,
        config.id,
        config.name,
        config.description,
        config.base_url,
        &config.include_patterns as _,
        &config.exclude_patterns as _,
        config.max_depth,
        config.max_pages_per_job,
        config.respect_robots_txt,
        config.user_agent,
        config.request_delay_ms,
        config.max_concurrent_requests,
        config.schedule,
        config.headers,
        config.created_at,
        config.updated_at,
        config.active
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Failed to insert config: {}", e);
        AppError::from(e)
    })?
    .id;
    
    info!("Successfully created config with id: {}", config_id);
    
    // Create HATEOAS links
    let links = serde_json::json!({
        "self": { "href": format!("/api/configs/{}", config_id) },
        "jobs": { "href": format!("/api/configs/{}/jobs", config_id) },
        "start": { "href": format!("/api/configs/{}/start", config_id) }
    });
    
    Ok((StatusCode::CREATED, Json(ConfigResponse {
        config,
        _links: links,
    })))
}

#[instrument(skip(state), fields(limit = %params.limit.unwrap_or(10), offset = %params.offset.unwrap_or(0)))]
pub async fn list_configs(
    State(state): State<AppState>,
    Query(params): Query<ListConfigsQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let limit = params.limit.unwrap_or(10);
    let offset = params.offset.unwrap_or(0);
    
    debug!("Listing configs with limit: {}, offset: {}", limit, offset);
    
    let configs = sqlx::query_as!(
        ScraperConfig,
        r#"
        SELECT 
            id, name, description, base_url, include_patterns, exclude_patterns,
            max_depth, max_pages_per_job, respect_robots_txt, user_agent,
            request_delay_ms, max_concurrent_requests, schedule, 
            headers as "headers: serde_json::Value",
            created_at, updated_at, active
        FROM scraper_configs
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
        "#,
        limit,
        offset
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Failed to fetch configs: {}", e);
        AppError::from(e)
    })?;
    
    info!("Retrieved {} configs", configs.len());
    
    let response = serde_json::json!({
        "configs": configs,
        "_links": {
            "self": { "href": "/api/configs" }
        }
    });
    
    Ok(Json(response))
}

#[instrument(skip(state), fields(config_id = %id))]
pub async fn get_config(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ConfigResponse>, AppError> {
    debug!("Fetching config with id: {}", id);
    
    let config = sqlx::query_as!(
        ScraperConfig,
        r#"
        SELECT 
            id, name, description, base_url, include_patterns, exclude_patterns,
            max_depth, max_pages_per_job, respect_robots_txt, user_agent,
            request_delay_ms, max_concurrent_requests, schedule, 
            headers as "headers: serde_json::Value",
            created_at, updated_at, active
        FROM scraper_configs
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Database error when fetching config {}: {}", id, e);
        AppError::from(e)
    })?
    .ok_or_else(|| {
        error!("Config not found: {}", id);
        AppError::NotFound(format!("Config not found: {}", id))
    })?;
    
    debug!("Found config: {} ({})", config.name, config.id);
    
    // Create HATEOAS links
    let links = serde_json::json!({
        "self": { "href": format!("/api/configs/{}", config.id) },
        "jobs": { "href": format!("/api/configs/{}/jobs", config.id) },
        "start": { "href": format!("/api/configs/{}/start", config.id) }
    });
    
    Ok(Json(ConfigResponse {
        config,
        _links: links,
    }))
}

#[instrument(skip(state, payload), fields(config_id = %id, config_name = %payload.name))]
pub async fn update_config(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CreateConfigRequest>,
) -> Result<Json<ConfigResponse>, AppError> {
    info!("Updating config: {}", id);
    
    // Get the existing config
    let mut config = sqlx::query_as!(
        ScraperConfig,
        r#"
        SELECT 
            id, name, description, base_url, include_patterns, exclude_patterns,
            max_depth, max_pages_per_job, respect_robots_txt, user_agent,
            request_delay_ms, max_concurrent_requests, schedule, 
            headers as "headers: serde_json::Value",
            created_at, updated_at, active
        FROM scraper_configs
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Database error when fetching config for update {}: {}", id, e);
        AppError::from(e)
    })?
    .ok_or_else(|| {
        error!("Config not found for update: {}", id);
        AppError::NotFound(format!("Config not found: {}", id))
    })?;
    
    debug!("Found config to update: {} ({})", config.name, config.id);
    
    // Update fields
    config.name = payload.name;
    config.description = payload.description;
    config.base_url = payload.base_url;
    config.include_patterns = payload.include_patterns;
    config.exclude_patterns = payload.exclude_patterns;
    config.max_depth = payload.max_depth;
    config.max_pages_per_job = Some(payload.max_pages_per_job.unwrap_or(config.max_pages_per_job.unwrap_or(1000)));
    config.respect_robots_txt = payload.respect_robots_txt.unwrap_or(config.respect_robots_txt);
    config.user_agent = payload.user_agent.unwrap_or(config.user_agent);
    config.request_delay_ms = payload.request_delay_ms.unwrap_or(config.request_delay_ms);
    config.max_concurrent_requests = payload.max_concurrent_requests.unwrap_or(config.max_concurrent_requests);
    config.schedule = payload.schedule.or(config.schedule);
    config.headers = payload.headers.unwrap_or(config.headers);
    config.updated_at = chrono::Utc::now();
    config.active = true;
    
    debug!("Updating config in database: {}", config.id);
    
    // Update in database
    sqlx::query!(
        r#"
        UPDATE scraper_configs
        SET 
            name = $1, description = $2, base_url = $3,
            include_patterns = $4, exclude_patterns = $5, max_depth = $6,
            max_pages_per_job = $7, respect_robots_txt = $8, user_agent = $9,
            request_delay_ms = $10, max_concurrent_requests = $11, schedule = $12,
            headers = $13, updated_at = $14, active = $15
        WHERE id = $16
        "#,
        config.name,
        config.description,
        config.base_url,
        &config.include_patterns as _,
        &config.exclude_patterns as _,
        config.max_depth,
        config.max_pages_per_job,
        config.respect_robots_txt,
        config.user_agent,
        config.request_delay_ms,
        config.max_concurrent_requests,
        config.schedule,
        config.headers,
        config.updated_at,
        config.active,
        config.id
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Failed to update config {}: {}", id, e);
        AppError::from(e)
    })?;
    
    info!("Successfully updated config: {}", config.id);
    
    // Create HATEOAS links
    let links = serde_json::json!({
        "self": { "href": format!("/api/configs/{}", config.id) },
        "jobs": { "href": format!("/api/configs/{}/jobs", config.id) },
        "start": { "href": format!("/api/configs/{}/start", config.id) }
    });
    
    Ok(Json(ConfigResponse {
        config,
        _links: links,
    }))
}

#[instrument(skip(state), fields(config_id = %id))]
pub async fn start_job(
    State(state): State<crate::api::routes::AppState>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    info!("Starting job for config: {}", id);
    
    let job = state.scraper_service.create_job(id).await.map_err(|e| {
        error!("Failed to create job for config {}: {:?}", id, e);
        match e.downcast::<AppError>() {
            Ok(app_error) => app_error,
            Err(other) => AppError::Internal(other.to_string()),
        }
    })?;
    
    info!("Successfully created job {} for config {}", job.id, id);
    
    let response = serde_json::json!({
        "job_id": job.id,
        "status": job.status.to_string(),
        "_links": {
            "self": { "href": format!("/api/jobs/{}", job.id) },
            "config": { "href": format!("/api/configs/{}", job.config_id) }
        }
    });
    
    Ok((StatusCode::CREATED, Json(response)))
} 