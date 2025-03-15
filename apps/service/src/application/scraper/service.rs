use anyhow::Result;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::job::{Job, JobStatus};
use crate::domain::scraper_config::ScraperConfig;
use crate::infrastructure::queue::redis_queue::{JobQueue, RedisJobQueue};
use crate::utils::error::AppError;

pub struct ScraperService {
    db_pool: PgPool,
    job_queue: Arc<RedisJobQueue>,
}

impl ScraperService {
    pub fn new(db_pool: PgPool, job_queue: Arc<RedisJobQueue>) -> Self {
        Self { db_pool, job_queue }
    }
    
    pub async fn get_config(&self, config_id: Uuid) -> Result<ScraperConfig> {
        let config = sqlx::query_as!(
            ScraperConfig,
            r#"
            SELECT 
                id, name, description, base_url, 
                include_patterns, exclude_patterns, max_depth, 
                max_pages_per_job, respect_robots_txt, user_agent, 
                request_delay_ms, max_concurrent_requests, schedule, 
                headers as "headers: serde_json::Value", 
                created_at, updated_at, active
            FROM scraper_configs
            WHERE id = $1
            "#,
            config_id
        )
        .fetch_optional(&self.db_pool)
        .await?;
        
        config.ok_or_else(|| AppError::NotFound(format!("Config not found: {}", config_id)).into())
    }
    
    pub async fn create_job(&self, config_id: Uuid) -> Result<Job> {
        // Verify config exists
        let config = self.get_config(config_id).await?;
        
        if !config.active {
            return Err(AppError::InvalidInput(format!("Config is not active: {}", config_id)).into());
        }
        
        // Create job
        let job = Job::new(config_id);
        
        // Save to database
        sqlx::query!(
            r#"
            INSERT INTO jobs (
                id, config_id, status, created_at, updated_at, 
                started_at, completed_at, error_message, 
                pages_crawled, pages_failed, pages_skipped, 
                next_run_at, worker_id, metadata
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            "#,
            job.id,
            job.config_id,
            job.status.to_string(),
            job.created_at,
            job.updated_at,
            job.started_at,
            job.completed_at,
            job.error_message,
            job.pages_crawled,
            job.pages_failed,
            job.pages_skipped,
            job.next_run_at,
            job.worker_id,
            job.metadata
        )
        .execute(&self.db_pool)
        .await?;
        
        // Enqueue job
        self.job_queue.enqueue("scraper_jobs", &job.id).await?;
        
        Ok(job)
    }
    
    pub async fn get_job(&self, job_id: Uuid) -> Result<Job> {
        let job = sqlx::query_as!(
            Job,
            r#"
            SELECT 
                id, config_id, 
                status as "status: JobStatus", 
                created_at, updated_at, started_at, completed_at, 
                error_message, pages_crawled, pages_failed, pages_skipped, 
                next_run_at, worker_id, 
                metadata as "metadata: serde_json::Value"
            FROM jobs
            WHERE id = $1
            "#,
            job_id
        )
        .fetch_optional(&self.db_pool)
        .await?;
        
        job.ok_or_else(|| AppError::NotFound(format!("Job not found: {}", job_id)).into())
    }
    
    pub async fn cancel_job(&self, job_id: Uuid) -> Result<Job> {
        // Get job
        let mut job = self.get_job(job_id).await?;
        
        // Check if job can be cancelled
        if job.status != JobStatus::Pending && job.status != JobStatus::Running {
            return Err(AppError::InvalidInput(format!("Job cannot be cancelled: {}", job_id)).into());
        }
        
        // Update job status
        job.cancel();
        
        // Update database
        sqlx::query!(
            r#"
            UPDATE jobs
            SET status = $1, updated_at = $2, completed_at = $3
            WHERE id = $4
            "#,
            job.status.to_string(),
            job.updated_at,
            job.completed_at,
            job.id
        )
        .execute(&self.db_pool)
        .await?;
        
        Ok(job)
    }
    
    pub async fn list_jobs(&self, limit: i64, offset: i64) -> Result<Vec<Job>> {
        let rows = sqlx::query!(
            r#"
            SELECT 
                id, config_id, 
                status, 
                created_at, updated_at, started_at, completed_at, 
                error_message, pages_crawled, pages_failed, pages_skipped, 
                next_run_at, worker_id, 
                metadata as "metadata: serde_json::Value"
            FROM jobs
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(&self.db_pool)
        .await?;
        
        let jobs = rows.into_iter().map(|row| {
            let status = match row.status.as_str() {
                "pending" => JobStatus::Pending,
                "running" => JobStatus::Running,
                "completed" => JobStatus::Completed,
                "failed" => JobStatus::Failed,
                "cancelled" => JobStatus::Cancelled,
                _ => JobStatus::Unknown,
            };
            
            Job {
                id: row.id,
                config_id: row.config_id,
                status,
                created_at: row.created_at,
                updated_at: row.updated_at,
                started_at: row.started_at,
                completed_at: row.completed_at,
                error_message: row.error_message,
                pages_crawled: row.pages_crawled,
                pages_failed: row.pages_failed,
                pages_skipped: row.pages_skipped,
                next_run_at: row.next_run_at,
                worker_id: row.worker_id,
                metadata: row.metadata,
            }
        }).collect();
        
        Ok(jobs)
    }
    
    pub async fn list_jobs_by_config(&self, config_id: Uuid, limit: i64, offset: i64) -> Result<Vec<Job>> {
        let jobs = sqlx::query_as!(
            Job,
            r#"
            SELECT 
                id, config_id, 
                status as "status: JobStatus", 
                created_at, updated_at, started_at, completed_at, 
                error_message, pages_crawled, pages_failed, pages_skipped, 
                next_run_at, worker_id, 
                metadata as "metadata: serde_json::Value"
            FROM jobs
            WHERE config_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            config_id,
            limit,
            offset
        )
        .fetch_all(&self.db_pool)
        .await?;
        
        Ok(jobs)
    }
} 