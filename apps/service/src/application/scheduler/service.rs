use anyhow::Result;
use chrono::{DateTime, Utc};
use cron::Schedule;
use sqlx::PgPool;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, error, warn};
use uuid::Uuid;

use crate::config::settings::Scheduler as SchedulerConfig;
use crate::domain::job::Job;
use crate::domain::scraper_config::ScraperConfig;
use crate::infrastructure::queue::redis_queue::{JobQueue, RedisJobQueue};
use crate::utils::error::AppError;

pub struct SchedulerService {
    db_pool: PgPool,
    job_queue: Arc<RedisJobQueue>,
    config: SchedulerConfig,
    running: bool,
}

impl SchedulerService {
    pub fn new(
        db_pool: PgPool,
        job_queue: Arc<RedisJobQueue>,
        config: SchedulerConfig,
    ) -> Self {
        Self {
            db_pool,
            job_queue,
            config,
            running: false,
        }
    }
    
    pub async fn start(&mut self) -> Result<()> {
        if !self.config.enabled {
            info!("Scheduler is disabled");
            return Ok(());
        }
        
        if self.running {
            return Ok(());
        }
        
        self.running = true;
        info!("Starting scheduler");
        
        while self.running {
            // Check for configs that need to be scheduled
            if let Err(e) = self.check_schedules().await {
                error!("Error checking schedules: {}", e);
            }
            
            // Wait for the next check interval
            sleep(Duration::from_secs(self.config.check_interval_seconds)).await;
        }
        
        info!("Scheduler stopped");
        Ok(())
    }
    
    pub fn stop(&mut self) {
        info!("Stopping scheduler");
        self.running = false;
    }
    
    async fn check_schedules(&self) -> Result<()> {
        // Get all active configs with schedules
        let configs = sqlx::query_as!(
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
            WHERE active = true AND schedule IS NOT NULL
            "#
        )
        .fetch_all(&self.db_pool)
        .await?;
        
        let now = Utc::now();
        
        for config in configs {
            if let Some(schedule_str) = &config.schedule {
                // Parse the cron schedule
                match Schedule::from_str(schedule_str) {
                    Ok(schedule) => {
                        // Get the next run time after the last run
                        let last_job = self.get_last_job_for_config(config.id).await?;
                        
                        let last_run = last_job
                            .as_ref()
                            .and_then(|job| job.completed_at)
                            .unwrap_or_else(|| config.created_at);
                        
                        // Check if there's a scheduled time between last run and now
                        let should_run = schedule
                            .after(&last_run)
                            .take(1)
                            .any(|next_run| next_run <= now);
                        
                        if should_run {
                            info!("Scheduling job for config: {}", config.name);
                            self.create_job(config.id).await?;
                        }
                    }
                    Err(e) => {
                        warn!("Invalid schedule for config {}: {}", config.name, e);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    async fn get_last_job_for_config(&self, config_id: Uuid) -> Result<Option<Job>> {
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
            WHERE config_id = $1
            ORDER BY created_at DESC
            LIMIT 1
            "#,
            config_id
        )
        .fetch_optional(&self.db_pool)
        .await?;
        
        Ok(job)
    }
    
    async fn create_job(&self, config_id: Uuid) -> Result<Job> {
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
} 