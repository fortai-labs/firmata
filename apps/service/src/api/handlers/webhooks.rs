use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::domain::webhook::{Webhook, WebhookEventType};
use crate::utils::error::AppError;
use crate::api::routes::AppState;

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
    State(state): State<AppState>,
    Query(params): Query<ListWebhooksQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let limit = params.limit.unwrap_or(10);
    let offset = params.offset.unwrap_or(0);
    
    // Build the query based on parameters
    let mut query = "SELECT id, name, url, event_types, secret, active, created_at, updated_at, headers 
                    FROM webhooks".to_string();
    
    if let Some(config_id) = params.config_id {
        query.push_str(&format!(" WHERE id = '{}'", config_id));
    }
    
    query.push_str(" ORDER BY created_at DESC LIMIT $1 OFFSET $2");
    
    let webhooks_raw = sqlx::query(&query)
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.db_pool)
        .await
        .map_err(AppError::from)?;
    
    // Convert raw webhooks to Webhook objects
    let webhooks = webhooks_raw.iter().map(|row| {
        let id: Uuid = row.get("id");
        let name: String = row.get("name");
        let url: String = row.get("url");
        let event_types_raw: Vec<String> = row.get("event_types");
        let secret: Option<String> = row.get("secret");
        let active: bool = row.get("active");
        let created_at: DateTime<Utc> = row.get("created_at");
        let updated_at: DateTime<Utc> = row.get("updated_at");
        let headers: serde_json::Value = row.get("headers");
        
        // Convert event_types from text[] to Vec<WebhookEventType>
        let event_types = event_types_raw.iter()
            .map(|s| match s.as_str() {
                "job.created" => WebhookEventType::JobCreated,
                "job.started" => WebhookEventType::JobStarted,
                "job.completed" => WebhookEventType::JobCompleted,
                "job.failed" => WebhookEventType::JobFailed,
                "job.cancelled" => WebhookEventType::JobCancelled,
                "page.crawled" => WebhookEventType::PageCrawled,
                "page.failed" => WebhookEventType::PageFailed,
                "content.changed" => WebhookEventType::ContentChanged,
                _ => WebhookEventType::JobCreated, // Default
            })
            .collect::<Vec<_>>();
        
        Webhook {
            id,
            name,
            url,
            event_types,
            secret,
            active,
            created_at,
            updated_at,
            headers,
        }
    }).collect::<Vec<_>>();
    
    let response = serde_json::json!({
        "webhooks": webhooks,
        "_links": {
            "self": { "href": "/api/webhooks" }
        }
    });
    
    Ok(Json(response))
}

pub async fn get_webhook(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let webhook = sqlx::query!(
        r#"
        SELECT 
            id, name, url, event_types, 
            secret, active, created_at, updated_at, headers as "headers: serde_json::Value"
        FROM webhooks
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(AppError::from)?
    .ok_or_else(|| AppError::NotFound(format!("Webhook not found: {}", id)))?;
    
    // Convert event_types from text[] to Vec<WebhookEventType>
    let event_types: Vec<WebhookEventType> = webhook.event_types
        .iter()
        .map(|s| match s.as_str() {
            "job.created" => WebhookEventType::JobCreated,
            "job.started" => WebhookEventType::JobStarted,
            "job.completed" => WebhookEventType::JobCompleted,
            "job.failed" => WebhookEventType::JobFailed,
            "job.cancelled" => WebhookEventType::JobCancelled,
            "page.crawled" => WebhookEventType::PageCrawled,
            "page.failed" => WebhookEventType::PageFailed,
            "content.changed" => WebhookEventType::ContentChanged,
            _ => WebhookEventType::JobCreated, // Default
        })
        .collect();
    
    // Construct a Webhook manually
    let webhook_obj = Webhook {
        id: webhook.id,
        name: webhook.name,
        url: webhook.url,
        event_types,
        secret: webhook.secret,
        active: webhook.active,
        created_at: webhook.created_at,
        updated_at: webhook.updated_at,
        headers: webhook.headers,
    };
    
    let response = serde_json::json!({
        "webhook": webhook_obj,
        "_links": {
            "self": { "href": format!("/api/webhooks/{}", webhook_obj.id) }
        }
    });
    
    Ok(Json(response))
}

pub async fn create_webhook(
    State(state): State<AppState>,
    Json(payload): Json<CreateWebhookRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Convert events to strings
    let event_types: Vec<String> = payload.events.iter()
        .map(|e| e.to_string())
        .collect();
    
    // Create the webhook
    let webhook_id = Uuid::new_v4();
    let webhook_raw = sqlx::query!(
        r#"
        INSERT INTO webhooks (id, name, url, event_types, headers, secret, active, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW())
        RETURNING id, name, url, event_types, 
                  secret, active, created_at, updated_at, headers as "headers: serde_json::Value"
        "#,
        webhook_id,
        payload.config_id.to_string(),
        payload.url,
        &event_types as &[String],
        payload.headers.unwrap_or(serde_json::json!({})),
        payload.description,
        payload.is_active.unwrap_or(true)
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(AppError::from)?;
    
    // Convert event_types from text[] to Vec<WebhookEventType>
    let event_types: Vec<WebhookEventType> = webhook_raw.event_types
        .iter()
        .map(|s| match s.as_str() {
            "job.created" => WebhookEventType::JobCreated,
            "job.started" => WebhookEventType::JobStarted,
            "job.completed" => WebhookEventType::JobCompleted,
            "job.failed" => WebhookEventType::JobFailed,
            "job.cancelled" => WebhookEventType::JobCancelled,
            "page.crawled" => WebhookEventType::PageCrawled,
            "page.failed" => WebhookEventType::PageFailed,
            "content.changed" => WebhookEventType::ContentChanged,
            _ => WebhookEventType::JobCreated, // Default
        })
        .collect();
    
    // Construct a Webhook manually
    let webhook = Webhook {
        id: webhook_raw.id,
        name: webhook_raw.name,
        url: webhook_raw.url,
        event_types,
        secret: webhook_raw.secret,
        active: webhook_raw.active,
        created_at: webhook_raw.created_at,
        updated_at: webhook_raw.updated_at,
        headers: webhook_raw.headers,
    };
    
    let response = serde_json::json!({
        "webhook": webhook,
        "_links": {
            "self": { "href": format!("/api/webhooks/{}", webhook.id) }
        }
    });
    
    Ok(Json(response))
}

pub async fn update_webhook(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateWebhookRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Get the current webhook
    let webhook_exists = sqlx::query!("SELECT id FROM webhooks WHERE id = $1", id)
        .fetch_optional(&state.db_pool)
        .await
        .map_err(AppError::from)?
        .is_some();
    
    if !webhook_exists {
        return Err(AppError::NotFound(format!("Webhook not found: {}", id)));
    }
    
    // Convert events to strings if provided
    let event_types = payload.events.as_ref().map(|events| {
        events.iter().map(|e| e.to_string()).collect::<Vec<String>>()
    });
    
    // Update the webhook
    let updated_webhook_raw = sqlx::query!(
        r#"
        UPDATE webhooks
        SET url = COALESCE($1, url),
            event_types = COALESCE($2, event_types),
            headers = COALESCE($3, headers),
            secret = COALESCE($4, secret),
            active = COALESCE($5, active),
            updated_at = NOW()
        WHERE id = $6
        RETURNING id, name, url, event_types, 
                  secret, active, created_at, updated_at, headers as "headers: serde_json::Value"
        "#,
        payload.url,
        event_types.as_deref(),
        payload.headers,
        payload.description,
        payload.is_active,
        id
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(AppError::from)?;
    
    // Convert event_types from text[] to Vec<WebhookEventType>
    let event_types: Vec<WebhookEventType> = updated_webhook_raw.event_types
        .iter()
        .map(|s| match s.as_str() {
            "job.created" => WebhookEventType::JobCreated,
            "job.started" => WebhookEventType::JobStarted,
            "job.completed" => WebhookEventType::JobCompleted,
            "job.failed" => WebhookEventType::JobFailed,
            "job.cancelled" => WebhookEventType::JobCancelled,
            "page.crawled" => WebhookEventType::PageCrawled,
            "page.failed" => WebhookEventType::PageFailed,
            "content.changed" => WebhookEventType::ContentChanged,
            _ => WebhookEventType::JobCreated, // Default
        })
        .collect();
    
    // Construct a Webhook manually
    let updated_webhook = Webhook {
        id: updated_webhook_raw.id,
        name: updated_webhook_raw.name,
        url: updated_webhook_raw.url,
        event_types,
        secret: updated_webhook_raw.secret,
        active: updated_webhook_raw.active,
        created_at: updated_webhook_raw.created_at,
        updated_at: updated_webhook_raw.updated_at,
        headers: updated_webhook_raw.headers,
    };
    
    let response = serde_json::json!({
        "webhook": updated_webhook,
        "_links": {
            "self": { "href": format!("/api/webhooks/{}", updated_webhook.id) }
        }
    });
    
    Ok(Json(response))
}

pub async fn delete_webhook(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    // Check if the webhook exists
    let webhook_exists = sqlx::query!("SELECT id FROM webhooks WHERE id = $1", id)
        .fetch_optional(&state.db_pool)
        .await
        .map_err(AppError::from)?
        .is_some();
    
    if !webhook_exists {
        return Err(AppError::NotFound(format!("Webhook not found: {}", id)));
    }
    
    // Delete the webhook
    sqlx::query!("DELETE FROM webhooks WHERE id = $1", id)
        .execute(&state.db_pool)
        .await
        .map_err(AppError::from)?;
    
    Ok(StatusCode::NO_CONTENT)
} 