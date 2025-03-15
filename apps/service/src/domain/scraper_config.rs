use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScraperConfig {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub base_url: String,
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub max_depth: i32,
    pub max_pages_per_job: Option<i32>,
    pub respect_robots_txt: bool,
    pub user_agent: String,
    pub request_delay_ms: i32,
    pub max_concurrent_requests: i32,
    pub schedule: Option<String>, // Cron expression
    pub headers: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub active: bool,
}

impl ScraperConfig {
    pub fn new(
        name: String,
        base_url: String,
        include_patterns: Vec<String>,
        exclude_patterns: Vec<String>,
        max_depth: i32,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            description: None,
            base_url,
            include_patterns,
            exclude_patterns,
            max_depth,
            max_pages_per_job: None,
            respect_robots_txt: true,
            user_agent: "FortaiBot/1.0".to_string(),
            request_delay_ms: 1000,
            max_concurrent_requests: 5,
            schedule: None,
            headers: serde_json::json!({}),
            created_at: now,
            updated_at: now,
            active: true,
        }
    }
} 