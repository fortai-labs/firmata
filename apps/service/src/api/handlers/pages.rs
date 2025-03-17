use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::domain::page::Page;
use crate::infrastructure::storage::s3_client::S3StorageClient;
use crate::infrastructure::storage::s3_client::StorageClient;
use crate::utils::error::AppError;
use crate::api::routes::AppState;

#[derive(Debug, Deserialize)]
pub struct ListPagesQuery {
    limit: Option<i64>,
    offset: Option<i64>,
    job_id: Option<Uuid>,
    url: Option<String>,
}

pub async fn list_pages(
    State(state): State<AppState>,
    Query(params): Query<ListPagesQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let limit = params.limit.unwrap_or(10);
    let offset = params.offset.unwrap_or(0);
    
    // Use a simpler query that doesn't rely on specific columns
    let mut query_builder = sqlx::QueryBuilder::new("SELECT * FROM pages");
    
    if let Some(job_id) = params.job_id {
        query_builder.push(" WHERE job_id = ");
        query_builder.push_bind(job_id);
    } else if let Some(url) = &params.url {
        query_builder.push(" WHERE url LIKE ");
        query_builder.push_bind(format!("%{}%", url));
    }
    
    query_builder.push(" ORDER BY crawled_at DESC LIMIT ");
    query_builder.push_bind(limit);
    query_builder.push(" OFFSET ");
    query_builder.push_bind(offset);
    
    let query = query_builder.build();
    
    // Fetch the rows as JSON directly
    let rows = query
        .fetch_all(&state.db_pool)
        .await
        .map_err(AppError::from)?;
    
    // Convert rows to a simplified format
    let pages: Vec<serde_json::Value> = rows.iter().map(|row| {
        let id: Uuid = row.get("id");
        let job_id: Uuid = row.get("job_id");
        let url: String = row.get("url");
        let normalized_url: String = row.get("normalized_url");
        let content_hash: String = row.get("content_hash");
        let http_status: i32 = row.get("http_status");
        let crawled_at: DateTime<Utc> = row.get("crawled_at");
        let html_storage_path: Option<String> = row.get("html_storage_path");
        let markdown_storage_path: Option<String> = row.get("markdown_storage_path");
        let title: Option<String> = row.get("title");
        let error_message: Option<String> = row.get("error_message");
        let depth: i32 = row.get("depth");
        let parent_url: Option<String> = row.get("parent_url");
        
        serde_json::json!({
            "id": id,
            "job_id": job_id,
            "url": url,
            "normalized_url": normalized_url,
            "content_hash": content_hash,
            "http_status": http_status,
            "crawled_at": crawled_at,
            "html_storage_path": html_storage_path,
            "markdown_storage_path": markdown_storage_path,
            "title": title,
            "error_message": error_message,
            "depth": depth,
            "parent_url": parent_url,
            "http_headers": serde_json::json!({}),
            "metadata": serde_json::json!({})
        })
    }).collect();
    
    let response = serde_json::json!({
        "pages": pages,
        "_links": {
            "self": { "href": "/api/pages" }
        }
    });
    
    Ok(Json(response))
}

pub async fn get_page(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Use a simpler query approach that doesn't rely on specific columns
    let row = sqlx::query("SELECT * FROM pages WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db_pool)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(format!("Page not found: {}", id)))?;
    
    // Extract fields manually
    let page_id: Uuid = row.get("id");
    let job_id: Uuid = row.get("job_id");
    let url: String = row.get("url");
    let normalized_url: String = row.get("normalized_url");
    let content_hash: String = row.get("content_hash");
    let http_status: i32 = row.get("http_status");
    let crawled_at: DateTime<Utc> = row.get("crawled_at");
    let html_storage_path: Option<String> = row.get("html_storage_path");
    let markdown_storage_path: Option<String> = row.get("markdown_storage_path");
    let title: Option<String> = row.get("title");
    let error_message: Option<String> = row.get("error_message");
    let depth: i32 = row.get("depth");
    let parent_url: Option<String> = row.get("parent_url");
    
    // Create a page object manually
    let page = serde_json::json!({
        "id": page_id,
        "job_id": job_id,
        "url": url,
        "normalized_url": normalized_url,
        "content_hash": content_hash,
        "http_status": http_status,
        "crawled_at": crawled_at,
        "html_storage_path": html_storage_path,
        "markdown_storage_path": markdown_storage_path,
        "title": title,
        "error_message": error_message,
        "depth": depth,
        "parent_url": parent_url,
        "http_headers": serde_json::json!({}),
        "metadata": serde_json::json!({})
    });
    
    let response = serde_json::json!({
        "page": page,
        "_links": {
            "self": { "href": format!("/api/pages/{}", page_id) },
            "job": { "href": format!("/api/jobs/{}", job_id) },
            "html": { "href": format!("/api/pages/{}/html", page_id) },
            "markdown": { "href": format!("/api/pages/{}/markdown", page_id) }
        }
    });
    
    Ok(Json(response))
}

pub async fn get_page_html(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Use a simpler query approach that doesn't rely on specific columns
    let row = sqlx::query("SELECT * FROM pages WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db_pool)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(format!("Page not found: {}", id)))?;
    
    // Extract fields manually
    let page_id: Uuid = row.get("id");
    let url: String = row.get("url");
    let crawled_at: DateTime<Utc> = row.get("crawled_at");
    let html_storage_path: Option<String> = row.get("html_storage_path");
    
    // Check if HTML is available
    if html_storage_path.is_none() {
        return Err(AppError::NotFound("HTML content not available for this page".to_string()));
    }
    
    // Get the HTML content from S3
    let html_content = state.storage_client.get_object(html_storage_path.unwrap().as_str()).await
        .map_err(|e| AppError::Internal(format!("Failed to retrieve HTML content: {}", e)))?;
    
    let response = serde_json::json!({
        "content": html_content,
        "page_id": page_id,
        "url": url,
        "crawled_at": crawled_at,
        "_links": {
            "self": { "href": format!("/api/pages/{}/html", page_id) },
            "page": { "href": format!("/api/pages/{}", page_id) },
            "markdown": { "href": format!("/api/pages/{}/markdown", page_id) }
        }
    });
    
    Ok(Json(response))
}

pub async fn get_page_markdown(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Use a simpler query approach that doesn't rely on specific columns
    let row = sqlx::query("SELECT * FROM pages WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db_pool)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(format!("Page not found: {}", id)))?;
    
    // Extract fields manually
    let page_id: Uuid = row.get("id");
    let url: String = row.get("url");
    let crawled_at: DateTime<Utc> = row.get("crawled_at");
    let markdown_storage_path: Option<String> = row.get("markdown_storage_path");
    
    // Check if Markdown is available
    if markdown_storage_path.is_none() {
        return Err(AppError::NotFound("Markdown content not available for this page".to_string()));
    }
    
    // Get the Markdown content from S3
    let markdown_content = state.storage_client.get_object(markdown_storage_path.unwrap().as_str()).await
        .map_err(|e| AppError::Internal(format!("Failed to retrieve Markdown content: {}", e)))?;
    
    let response = serde_json::json!({
        "content": markdown_content,
        "page_id": page_id,
        "url": url,
        "crawled_at": crawled_at,
        "_links": {
            "self": { "href": format!("/api/pages/{}/markdown", page_id) },
            "page": { "href": format!("/api/pages/{}", page_id) },
            "html": { "href": format!("/api/pages/{}/html", page_id) }
        }
    });
    
    Ok(Json(response))
} 