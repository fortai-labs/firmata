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
use crate::utils::error::AppError;

#[derive(Debug, Deserialize)]
pub struct ListPagesQuery {
    limit: Option<i64>,
    offset: Option<i64>,
    job_id: Option<Uuid>,
    url: Option<String>,
}

pub async fn list_pages(
    State(db_pool): State<PgPool>,
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
    
    query.push_str(" ORDER BY crawled_at DESC LIMIT $1 OFFSET $2");
    
    let pages = sqlx::query_as::<_, Page>(&query)
        .bind(limit)
        .bind(offset)
        .fetch_all(&db_pool)
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
    State(db_pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let page = sqlx::query_as!(
        Page,
        r#"
        SELECT 
            id, job_id, url, normalized_url, content_hash,
            http_status, http_headers as "http_headers: serde_json::Value",
            crawled_at, html_storage_path, markdown_storage_path,
            title, metadata as "metadata: serde_json::Value",
            error_message, depth, parent_url
        FROM pages
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(&db_pool)
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
    State(db_pool): State<PgPool>,
    State(storage_client): State<Arc<S3StorageClient>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Get the page
    let page = sqlx::query_as!(
        Page,
        r#"
        SELECT 
            id, job_id, url, normalized_url, content_hash,
            http_status, http_headers as "http_headers: serde_json::Value",
            crawled_at, html_storage_path, markdown_storage_path,
            title, metadata as "metadata: serde_json::Value",
            error_message, depth, parent_url
        FROM pages
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(&db_pool)
    .await
    .map_err(AppError::from)?
    .ok_or_else(|| AppError::NotFound(format!("Page not found: {}", id)))?;
    
    // Get the HTML content
    let html_path = page.html_storage_path
        .ok_or_else(|| AppError::NotFound("HTML content not available".to_string()))?;
    
    let html_content = storage_client.get_object(&html_path).await?;
    
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
    State(db_pool): State<PgPool>,
    State(storage_client): State<Arc<S3StorageClient>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Get the page
    let page = sqlx::query_as!(
        Page,
        r#"
        SELECT 
            id, job_id, url, normalized_url, content_hash,
            http_status, http_headers as "http_headers: serde_json::Value",
            crawled_at, html_storage_path, markdown_storage_path,
            title, metadata as "metadata: serde_json::Value",
            error_message, depth, parent_url
        FROM pages
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(&db_pool)
    .await
    .map_err(AppError::from)?
    .ok_or_else(|| AppError::NotFound(format!("Page not found: {}", id)))?;
    
    // Get the Markdown content
    let markdown_path = page.markdown_storage_path
        .ok_or_else(|| AppError::NotFound("Markdown content not available".to_string()))?;
    
    let markdown_content = storage_client.get_object(&markdown_path).await?;
    
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