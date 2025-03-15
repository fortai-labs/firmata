use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl ToString for JobStatus {
    fn to_string(&self) -> String {
        match self {
            JobStatus::Pending => "pending".to_string(),
            JobStatus::Running => "running".to_string(),
            JobStatus::Completed => "completed".to_string(),
            JobStatus::Failed => "failed".to_string(),
            JobStatus::Cancelled => "cancelled".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: Uuid,
    pub config_id: Uuid,
    pub status: JobStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub pages_crawled: i32,
    pub pages_failed: i32,
    pub pages_skipped: i32,
    pub next_run_at: Option<DateTime<Utc>>,
    pub worker_id: Option<String>,
    pub metadata: serde_json::Value,
}

impl Job {
    pub fn new(config_id: Uuid) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            config_id,
            status: JobStatus::Pending,
            created_at: now,
            updated_at: now,
            started_at: None,
            completed_at: None,
            error_message: None,
            pages_crawled: 0,
            pages_failed: 0,
            pages_skipped: 0,
            next_run_at: None,
            worker_id: None,
            metadata: serde_json::json!({}),
        }
    }

    pub fn start(&mut self, worker_id: String) {
        let now = Utc::now();
        self.status = JobStatus::Running;
        self.started_at = Some(now);
        self.updated_at = now;
        self.worker_id = Some(worker_id);
    }

    pub fn complete(&mut self) {
        let now = Utc::now();
        self.status = JobStatus::Completed;
        self.completed_at = Some(now);
        self.updated_at = now;
    }

    pub fn fail(&mut self, error_message: String) {
        let now = Utc::now();
        self.status = JobStatus::Failed;
        self.error_message = Some(error_message);
        self.completed_at = Some(now);
        self.updated_at = now;
    }

    pub fn cancel(&mut self) {
        let now = Utc::now();
        self.status = JobStatus::Cancelled;
        self.completed_at = Some(now);
        self.updated_at = now;
    }

    pub fn increment_crawled(&mut self) {
        self.pages_crawled += 1;
        self.updated_at = Utc::now();
    }

    pub fn increment_failed(&mut self) {
        self.pages_failed += 1;
        self.updated_at = Utc::now();
    }

    pub fn increment_skipped(&mut self) {
        self.pages_skipped += 1;
        self.updated_at = Utc::now();
    }
} 