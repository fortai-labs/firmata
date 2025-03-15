use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::utils::error::AppError;

#[derive(Debug, Deserialize)]
pub struct JobStatsQuery {
    config_id: Option<Uuid>,
    start_date: Option<DateTime<Utc>>,
    end_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct JobStats {
    job_id: Uuid,
    config_id: Uuid,
    start_time: DateTime<Utc>,
    end_time: Option<DateTime<Utc>>,
    status: String,
    total_pages: i64,
    successful_pages: i64,
    failed_pages: i64,
    avg_page_time_ms: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct ConfigStats {
    config_id: Uuid,
    name: String,
    total_jobs: i64,
    total_pages: i64,
    successful_pages: i64,
    failed_pages: i64,
    avg_job_time_seconds: Option<f64>,
    last_job_time: Option<DateTime<Utc>>,
}

pub async fn get_job_stats(
    State(db_pool): State<PgPool>,
    Query(params): Query<JobStatsQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut query = "
        SELECT 
            j.id as job_id,
            j.config_id,
            j.started_at as start_time,
            j.completed_at as end_time,
            j.status,
            COUNT(p.id) as total_pages,
            COUNT(CASE WHEN p.error_message IS NULL THEN 1 END) as successful_pages,
            COUNT(CASE WHEN p.error_message IS NOT NULL THEN 1 END) as failed_pages,
            AVG(EXTRACT(EPOCH FROM (p.crawled_at - j.started_at)) * 1000) as avg_page_time_ms
        FROM jobs j
        LEFT JOIN pages p ON j.id = p.job_id
    ".to_string();
    
    let mut conditions = Vec::new();
    
    if let Some(config_id) = params.config_id {
        conditions.push(format!("j.config_id = '{}'", config_id));
    }
    
    if let Some(start_date) = params.start_date {
        conditions.push(format!("j.started_at >= '{}'", start_date));
    }
    
    if let Some(end_date) = params.end_date {
        conditions.push(format!("j.started_at <= '{}'", end_date));
    }
    
    if !conditions.is_empty() {
        query.push_str(" WHERE ");
        query.push_str(&conditions.join(" AND "));
    }
    
    query.push_str(" GROUP BY j.id ORDER BY j.started_at DESC");
    
    let job_stats = sqlx::query_as!(
        JobStats,
        &query
    )
    .fetch_all(&db_pool)
    .await
    .map_err(AppError::from)?;
    
    let response = serde_json::json!({
        "job_stats": job_stats,
        "_links": {
            "self": { "href": "/api/analytics/jobs" }
        }
    });
    
    Ok(Json(response))
}

pub async fn get_config_stats(
    State(db_pool): State<PgPool>,
) -> Result<Json<serde_json::Value>, AppError> {
    let config_stats = sqlx::query_as!(
        ConfigStats,
        r#"
        SELECT 
            c.id as config_id,
            c.name,
            COUNT(DISTINCT j.id) as total_jobs,
            COUNT(p.id) as total_pages,
            COUNT(CASE WHEN p.error_message IS NULL THEN 1 END) as successful_pages,
            COUNT(CASE WHEN p.error_message IS NOT NULL THEN 1 END) as failed_pages,
            AVG(EXTRACT(EPOCH FROM (j.completed_at - j.started_at))) as avg_job_time_seconds,
            MAX(j.started_at) as last_job_time
        FROM configs c
        LEFT JOIN jobs j ON c.id = j.config_id
        LEFT JOIN pages p ON j.id = p.job_id
        GROUP BY c.id, c.name
        ORDER BY last_job_time DESC NULLS LAST
        "#
    )
    .fetch_all(&db_pool)
    .await
    .map_err(AppError::from)?;
    
    let response = serde_json::json!({
        "config_stats": config_stats,
        "_links": {
            "self": { "href": "/api/analytics/configs" }
        }
    });
    
    Ok(Json(response))
}

pub async fn get_job_timeline(
    State(db_pool): State<PgPool>,
    Path(job_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Check if the job exists
    let job_exists = sqlx::query!("SELECT id FROM jobs WHERE id = $1", job_id)
        .fetch_optional(&db_pool)
        .await
        .map_err(AppError::from)?
        .is_some();
    
    if !job_exists {
        return Err(AppError::NotFound(format!("Job not found: {}", job_id)));
    }
    
    // Get the timeline data
    let timeline_data = sqlx::query!(
        r#"
        SELECT 
            crawled_at as timestamp,
            url,
            http_status,
            error_message,
            depth
        FROM pages
        WHERE job_id = $1
        ORDER BY crawled_at ASC
        "#,
        job_id
    )
    .fetch_all(&db_pool)
    .await
    .map_err(AppError::from)?;
    
    // Get job details
    let job = sqlx::query!(
        r#"
        SELECT 
            id,
            config_id,
            status,
            started_at,
            completed_at
        FROM jobs
        WHERE id = $1
        "#,
        job_id
    )
    .fetch_one(&db_pool)
    .await
    .map_err(AppError::from)?;
    
    let response = serde_json::json!({
        "job": {
            "id": job.id,
            "config_id": job.config_id,
            "status": job.status,
            "started_at": job.started_at,
            "completed_at": job.completed_at
        },
        "timeline": timeline_data.iter().map(|entry| {
            serde_json::json!({
                "timestamp": entry.timestamp,
                "url": entry.url,
                "http_status": entry.http_status,
                "error_message": entry.error_message,
                "depth": entry.depth
            })
        }).collect::<Vec<_>>(),
        "_links": {
            "self": { "href": format!("/api/analytics/jobs/{}/timeline", job_id) },
            "job": { "href": format!("/api/jobs/{}", job_id) }
        }
    });
    
    Ok(Json(response))
} 