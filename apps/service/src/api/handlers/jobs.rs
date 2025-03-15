use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::application::scraper::service::ScraperService;
use crate::utils::error::AppError;

#[derive(Debug, Deserialize)]
pub struct ListJobsQuery {
    limit: Option<i64>,
    offset: Option<i64>,
    config_id: Option<Uuid>,
}

pub async fn list_jobs(
    State(scraper_service): State<Arc<ScraperService>>,
    Query(params): Query<ListJobsQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let limit = params.limit.unwrap_or(10);
    let offset = params.offset.unwrap_or(0);
    
    let jobs = if let Some(config_id) = params.config_id {
        scraper_service.list_jobs_by_config(config_id, limit, offset).await?
    } else {
        scraper_service.list_jobs(limit, offset).await?
    };
    
    let response = serde_json::json!({
        "jobs": jobs,
        "_links": {
            "self": { "href": "/api/jobs" }
        }
    });
    
    Ok(Json(response))
}

pub async fn get_job(
    State(scraper_service): State<Arc<ScraperService>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let job = scraper_service.get_job(id).await?;
    
    let response = serde_json::json!({
        "job": job,
        "_links": {
            "self": { "href": format!("/api/jobs/{}", job.id) },
            "config": { "href": format!("/api/configs/{}", job.config_id) }
        }
    });
    
    Ok(Json(response))
}

pub async fn cancel_job(
    State(scraper_service): State<Arc<ScraperService>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let job = scraper_service.cancel_job(id).await?;
    
    let response = serde_json::json!({
        "job": job,
        "_links": {
            "self": { "href": format!("/api/jobs/{}", job.id) },
            "config": { "href": format!("/api/configs/{}", job.config_id) }
        }
    });
    
    Ok(Json(response))
} 