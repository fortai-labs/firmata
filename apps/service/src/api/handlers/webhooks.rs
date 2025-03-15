use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::webhook::Webhook;
use crate::utils::error::AppError;

#[derive(Debug, Deserialize)]
pub struct ListWebhooksQuery {
    limit: Option<i64>,
    offset: Option<i64>,
    config_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct CreateWebhookRequest {
    config_id: Uuid,
    url: String,
    events: Vec<String>,
    headers: Option<serde_json::Value>,
    description: Option<String>,
    is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWebhookRequest {
    url: Option<String>,
    events: Option<Vec<String>>,
    headers: Option<serde_json::Value>,
    description: Option<String>,
    is_active: Option<bool>,
}

pub async fn list_webhooks(
    State(db_pool): State<PgPool>,
    Query(params): Query<ListWebhooksQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let limit = params.limit.unwrap_or(10);
    let offset = params.offset.unwrap_or(0);
    
    // Build the query based on parameters
    let mut query = "SELECT id, config_id, url, events, headers as \"headers: serde_json::Value\", 
                    description, is_active, created_at, updated_at 
                    FROM webhooks".to_string();
    
    if let Some(config_id) = params.config_id {
        query.push_str(&format!(" WHERE config_id = '{}'", config_id));
    }
    
    query.push_str(" ORDER BY created_at DESC LIMIT $1 OFFSET $2");
    
    let webhooks = sqlx::query_as::<_, Webhook>(&query)
        .bind(limit)
        .bind(offset)
        .fetch_all(&db_pool)
        .await
        .map_err(AppError::from)?;
    
    let response = serde_json::json!({
        "webhooks": webhooks,
        "_links": {
            "self": { "href": "/api/webhooks" }
        }
    });
    
    Ok(Json(response))
}

pub async fn get_webhook(
    State(db_pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let webhook = sqlx::query_as!(
        Webhook,
        r#"
        SELECT 
            id, config_id, url, events, headers as "headers: serde_json::Value",
            description, is_active, created_at, updated_at
        FROM webhooks
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(&db_pool)
    .await
    .map_err(AppError::from)?
    .ok_or_else(|| AppError::NotFound(format!("Webhook not found: {}", id)))?;
    
    let response = serde_json::json!({
        "webhook": webhook,
        "_links": {
            "self": { "href": format!("/api/webhooks/{}", webhook.id) },
            "config": { "href": format!("/api/configs/{}", webhook.config_id) }
        }
    });
    
    Ok(Json(response))
}

pub async fn create_webhook(
    State(db_pool): State<PgPool>,
    Json(payload): Json<CreateWebhookRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Validate that the config exists
    let config_exists = sqlx::query!("SELECT id FROM configs WHERE id = $1", payload.config_id)
        .fetch_optional(&db_pool)
        .await
        .map_err(AppError::from)?
        .is_some();
    
    if !config_exists {
        return Err(AppError::NotFound(format!("Config not found: {}", payload.config_id)));
    }
    
    // Create the webhook
    let webhook = sqlx::query_as!(
        Webhook,
        r#"
        INSERT INTO webhooks (config_id, url, events, headers, description, is_active)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, config_id, url, events, headers as "headers: serde_json::Value",
                  description, is_active, created_at, updated_at
        "#,
        payload.config_id,
        payload.url,
        &payload.events as &[String],
        payload.headers,
        payload.description,
        payload.is_active.unwrap_or(true)
    )
    .fetch_one(&db_pool)
    .await
    .map_err(AppError::from)?;
    
    let response = serde_json::json!({
        "webhook": webhook,
        "_links": {
            "self": { "href": format!("/api/webhooks/{}", webhook.id) },
            "config": { "href": format!("/api/configs/{}", webhook.config_id) }
        }
    });
    
    Ok(Json(response))
}

pub async fn update_webhook(
    State(db_pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateWebhookRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Get the current webhook
    let webhook = sqlx::query_as!(
        Webhook,
        r#"
        SELECT 
            id, config_id, url, events, headers as "headers: serde_json::Value",
            description, is_active, created_at, updated_at
        FROM webhooks
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(&db_pool)
    .await
    .map_err(AppError::from)?
    .ok_or_else(|| AppError::NotFound(format!("Webhook not found: {}", id)))?;
    
    // Update the webhook
    let updated_webhook = sqlx::query_as!(
        Webhook,
        r#"
        UPDATE webhooks
        SET url = COALESCE($1, url),
            events = COALESCE($2, events),
            headers = COALESCE($3, headers),
            description = COALESCE($4, description),
            is_active = COALESCE($5, is_active),
            updated_at = NOW()
        WHERE id = $6
        RETURNING id, config_id, url, events, headers as "headers: serde_json::Value",
                  description, is_active, created_at, updated_at
        "#,
        payload.url,
        payload.events.as_deref(),
        payload.headers,
        payload.description,
        payload.is_active,
        id
    )
    .fetch_one(&db_pool)
    .await
    .map_err(AppError::from)?;
    
    let response = serde_json::json!({
        "webhook": updated_webhook,
        "_links": {
            "self": { "href": format!("/api/webhooks/{}", updated_webhook.id) },
            "config": { "href": format!("/api/configs/{}", updated_webhook.config_id) }
        }
    });
    
    Ok(Json(response))
}

pub async fn delete_webhook(
    State(db_pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    // Check if the webhook exists
    let webhook_exists = sqlx::query!("SELECT id FROM webhooks WHERE id = $1", id)
        .fetch_optional(&db_pool)
        .await
        .map_err(AppError::from)?
        .is_some();
    
    if !webhook_exists {
        return Err(AppError::NotFound(format!("Webhook not found: {}", id)));
    }
    
    // Delete the webhook
    sqlx::query!("DELETE FROM webhooks WHERE id = $1", id)
        .execute(&db_pool)
        .await
        .map_err(AppError::from)?;
    
    Ok(StatusCode::NO_CONTENT)
} 