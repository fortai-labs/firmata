use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

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
    
    // Build the query based on parameters
    let mut query = "SELECT id, job_id, url, normalized_url, content_hash, 
                    http_status, http_headers as \"http_headers: serde_json::Value\", 
                    crawled_at, html_storage_path, markdown_storage_path, 
                    title, metadata as \"metadata: serde_json::Value\", 
                    error_message, depth, parent_url 
                    FROM pages".to_string();
    
    let mut conditions = Vec::new();
    
    if let Some(job_id) = params.job_id {
        conditions.push(format!("job_id = '{}'", job_id));
    }
    
    if let Some(url) = &params.url {
        conditions.push(format!("url LIKE '%{}%'", url.replace('\'', "''")));
    }
    
    if !conditions.is_empty() {
        query.push_str(" WHERE ");
        query.push_str(&conditions.join(" AND "));
    }
    
    query.push_str(&format!(" ORDER BY crawled_at DESC LIMIT {} OFFSET {}", limit, offset));
    
    let pages = sqlx::query_as::<_, Page>(&query)
        .fetch_all(&state.db_pool)
        .await
        .map_err(AppError::from)?;
    
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
    let page = sqlx::query_as::<_, Page>(
        r#"
        SELECT id, job_id, url, normalized_url, content_hash, 
               http_status, http_headers as "http_headers: serde_json::Value", 
               crawled_at, html_storage_path, markdown_storage_path, 
               title, metadata as "metadata: serde_json::Value", 
               error_message, depth, parent_url 
        FROM pages
        WHERE id = $1
        "#
    )
    .bind(id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(AppError::from)?
    .ok_or_else(|| AppError::NotFound(format!("Page not found: {}", id)))?;
    
    let response = serde_json::json!({
        "page": page,
        "_links": {
            "self": { "href": format!("/api/pages/{}", page.id) },
            "job": { "href": format!("/api/jobs/{}", page.job_id) },
            "html": { "href": format!("/api/pages/{}/html", page.id) },
            "markdown": { "href": format!("/api/pages/{}/markdown", page.id) }
        }
    });
    
    Ok(Json(response))
}

pub async fn get_page_html(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Get the page from the database
    let page = sqlx::query_as::<_, Page>(
        r#"
        SELECT id, job_id, url, normalized_url, content_hash, 
               http_status, http_headers as "http_headers: serde_json::Value", 
               crawled_at, html_storage_path, markdown_storage_path, 
               title, metadata as "metadata: serde_json::Value", 
               error_message, depth, parent_url 
        FROM pages
        WHERE id = $1
        "#
    )
    .bind(id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(AppError::from)?
    .ok_or_else(|| AppError::NotFound(format!("Page not found: {}", id)))?;
    
    // Check if HTML is available
    if page.html_storage_path.is_none() {
        return Err(AppError::NotFound("HTML content not available for this page".to_string()));
    }
    
    // Get the HTML content from S3
    let html_content = state.storage_client.get_object(page.html_storage_path.unwrap().as_str()).await
        .map_err(|e| AppError::Internal(format!("Failed to retrieve HTML content: {}", e)))?;
    
    let response = serde_json::json!({
        "content": html_content,
        "page_id": page.id,
        "url": page.url,
        "crawled_at": page.crawled_at,
        "_links": {
            "self": { "href": format!("/api/pages/{}/html", page.id) },
            "page": { "href": format!("/api/pages/{}", page.id) },
            "markdown": { "href": format!("/api/pages/{}/markdown", page.id) }
        }
    });
    
    Ok(Json(response))
}

pub async fn get_page_markdown(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Get the page from the database
    let page = sqlx::query_as::<_, Page>(
        r#"
        SELECT id, job_id, url, normalized_url, content_hash, 
               http_status, http_headers as "http_headers: serde_json::Value", 
               crawled_at, html_storage_path, markdown_storage_path, 
               title, metadata as "metadata: serde_json::Value", 
               error_message, depth, parent_url 
        FROM pages
        WHERE id = $1
        "#
    )
    .bind(id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(AppError::from)?
    .ok_or_else(|| AppError::NotFound(format!("Page not found: {}", id)))?;
    
    // Check if Markdown is available
    if page.markdown_storage_path.is_none() {
        return Err(AppError::NotFound("Markdown content not available for this page".to_string()));
    }
    
    // Get the Markdown content from S3
    let markdown_content = state.storage_client.get_object(page.markdown_storage_path.unwrap().as_str()).await
        .map_err(|e| AppError::Internal(format!("Failed to retrieve Markdown content: {}", e)))?;
    
    let response = serde_json::json!({
        "content": markdown_content,
        "page_id": page.id,
        "url": page.url,
        "crawled_at": page.crawled_at,
        "_links": {
            "self": { "href": format!("/api/pages/{}/markdown", page.id) },
            "page": { "href": format!("/api/pages/{}", page.id) },
            "html": { "href": format!("/api/pages/{}/html", page.id) }
        }
    });
    
    Ok(Json(response))
} 