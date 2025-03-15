use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WebhookEventType {
    JobCreated,
    JobStarted,
    JobCompleted,
    JobFailed,
    JobCancelled,
    PageCrawled,
    PageFailed,
    ContentChanged,
}

impl ToString for WebhookEventType {
    fn to_string(&self) -> String {
        match self {
            WebhookEventType::JobCreated => "job.created".to_string(),
            WebhookEventType::JobStarted => "job.started".to_string(),
            WebhookEventType::JobCompleted => "job.completed".to_string(),
            WebhookEventType::JobFailed => "job.failed".to_string(),
            WebhookEventType::JobCancelled => "job.cancelled".to_string(),
            WebhookEventType::PageCrawled => "page.crawled".to_string(),
            WebhookEventType::PageFailed => "page.failed".to_string(),
            WebhookEventType::ContentChanged => "content.changed".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WebhookDeliveryStatus {
    Pending,
    Delivered,
    Failed,
}

impl ToString for WebhookDeliveryStatus {
    fn to_string(&self) -> String {
        match self {
            WebhookDeliveryStatus::Pending => "pending".to_string(),
            WebhookDeliveryStatus::Delivered => "delivered".to_string(),
            WebhookDeliveryStatus::Failed => "failed".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Webhook {
    pub id: Uuid,
    pub name: String,
    pub url: String,
    pub event_types: Vec<WebhookEventType>,
    pub secret: Option<String>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub headers: serde_json::Value,
}

impl Webhook {
    pub fn new(name: String, url: String, event_types: Vec<WebhookEventType>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            url,
            event_types,
            secret: None,
            active: true,
            created_at: now,
            updated_at: now,
            headers: serde_json::json!({}),
        }
    }

    pub fn with_secret(
        name: String,
        url: String,
        event_types: Vec<WebhookEventType>,
        secret: String,
    ) -> Self {
        let mut webhook = Self::new(name, url, event_types);
        webhook.secret = Some(secret);
        webhook
    }

    pub fn is_subscribed_to(&self, event_type: &WebhookEventType) -> bool {
        self.event_types.contains(event_type)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookDelivery {
    pub id: Uuid,
    pub webhook_id: Uuid,
    pub event_type: WebhookEventType,
    pub payload: serde_json::Value,
    pub status: WebhookDeliveryStatus,
    pub response_status: Option<i32>,
    pub response_body: Option<String>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub retry_count: i32,
    pub next_retry_at: Option<DateTime<Utc>>,
}

impl WebhookDelivery {
    pub fn new(webhook_id: Uuid, event_type: WebhookEventType, payload: serde_json::Value) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            webhook_id,
            event_type,
            payload,
            status: WebhookDeliveryStatus::Pending,
            response_status: None,
            response_body: None,
            error_message: None,
            created_at: now,
            updated_at: now,
            delivered_at: None,
            retry_count: 0,
            next_retry_at: None,
        }
    }

    pub fn mark_delivered(&mut self, response_status: i32, response_body: Option<String>) {
        let now = Utc::now();
        self.status = WebhookDeliveryStatus::Delivered;
        self.response_status = Some(response_status);
        self.response_body = response_body;
        self.delivered_at = Some(now);
        self.updated_at = now;
    }

    pub fn mark_failed(&mut self, error_message: String, retry_after_seconds: Option<i64>) {
        let now = Utc::now();
        self.status = WebhookDeliveryStatus::Failed;
        self.error_message = Some(error_message);
        self.retry_count += 1;
        self.updated_at = now;

        if let Some(seconds) = retry_after_seconds {
            self.next_retry_at = Some(now + chrono::Duration::seconds(seconds));
        }
    }
} 