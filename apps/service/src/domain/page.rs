use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(sqlx::FromRow,Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    pub id: Uuid,
    pub job_id: Uuid,
    pub url: String,
    pub normalized_url: String,
    pub content_hash: String,
    pub http_status: i32,
    pub http_headers: serde_json::Value,
    pub crawled_at: DateTime<Utc>,
    pub html_storage_path: Option<String>,
    pub markdown_storage_path: Option<String>,
    pub title: Option<String>,
    pub metadata: serde_json::Value,
    pub error_message: Option<String>,
    pub depth: i32,
    pub parent_url: Option<String>,
    
    // Temporary field to hold HTML content, not stored in the database
    #[sqlx(skip)]
    #[serde(skip)]
    pub html_content: Option<String>,
}

impl Page {
    pub fn new(
        job_id: Uuid,
        url: String,
        normalized_url: String,
        http_status: i32,
        http_headers: serde_json::Value,
        content_hash: String,
        depth: i32,
        parent_url: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            job_id,
            url,
            normalized_url,
            content_hash,
            http_status,
            http_headers,
            crawled_at: Utc::now(),
            html_storage_path: None,
            markdown_storage_path: None,
            title: None,
            metadata: serde_json::json!({}),
            error_message: None,
            depth,
            parent_url,
            html_content: None,
        }
    }

    pub fn with_error(
        job_id: Uuid,
        url: String,
        normalized_url: String,
        error_message: String,
        depth: i32,
        parent_url: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            job_id,
            url,
            normalized_url,
            content_hash: String::new(),
            http_status: 0,
            http_headers: serde_json::json!({}),
            crawled_at: Utc::now(),
            html_storage_path: None,
            markdown_storage_path: None,
            title: None,
            metadata: serde_json::json!({}),
            error_message: Some(error_message),
            depth,
            parent_url,
            html_content: None,
        }
    }

    pub fn set_html_storage_path(&mut self, path: String) {
        self.html_storage_path = Some(path);
    }

    pub fn set_markdown_storage_path(&mut self, path: String) {
        self.markdown_storage_path = Some(path);
    }

    pub fn set_title(&mut self, title: String) {
        self.title = Some(title);
    }

    pub fn add_metadata(&mut self, key: &str, value: serde_json::Value) {
        if let serde_json::Value::Object(ref mut map) = self.metadata {
            map.insert(key.to_string(), value);
        }
    }
} 