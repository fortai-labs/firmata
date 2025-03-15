use anyhow::Result;
use async_trait::async_trait;
use deadpool_redis::{Config, Pool, Runtime};
use redis::AsyncCommands;
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;
use uuid::Uuid;

use crate::config::settings::Redis as RedisConfig;
use crate::utils::error::AppError;

const DEFAULT_VISIBILITY_TIMEOUT: u64 = 300; // 5 minutes
const DEFAULT_POLL_INTERVAL: u64 = 5; // 5 seconds

#[async_trait]
pub trait JobQueue {
    async fn enqueue<T: Serialize + Send + Sync>(&self, queue: &str, job: &T) -> Result<String>;
    async fn dequeue<T: DeserializeOwned + Send + Sync>(&self, queue: &str) -> Result<Option<(String, T)>>;
    async fn complete(&self, queue: &str, job_id: &str) -> Result<()>;
    async fn fail(&self, queue: &str, job_id: &str, error: &str) -> Result<()>;
    async fn schedule<T: Serialize + Send + Sync>(&self, queue: &str, job: &T, delay_seconds: u64) -> Result<String>;
}

pub struct RedisJobQueue {
    pool: Pool,
    visibility_timeout: u64,
}

impl RedisJobQueue {
    pub async fn new(config: &RedisConfig) -> Result<Self> {
        let cfg = Config::from_url(&config.url);
        let pool = cfg.create_pool(Some(Runtime::Tokio1))?;
        
        // Test connection
        let mut conn = pool.get().await.map_err(|e| AppError::Redis(e.to_string()))?;
        redis::cmd("PING").query_async(&mut conn).await.map_err(|e| AppError::Redis(e.to_string()))?;
        
        Ok(Self {
            pool,
            visibility_timeout: DEFAULT_VISIBILITY_TIMEOUT,
        })
    }
    
    pub fn with_visibility_timeout(mut self, seconds: u64) -> Self {
        self.visibility_timeout = seconds;
        self
    }
    
    async fn get_connection(&self) -> Result<deadpool_redis::Connection> {
        self.pool.get().await.map_err(|e| AppError::Redis(e.to_string()).into())
    }
}

#[async_trait]
impl JobQueue for RedisJobQueue {
    async fn enqueue<T: Serialize + Send + Sync>(&self, queue: &str, job: &T) -> Result<String> {
        let job_id = Uuid::new_v4().to_string();
        let job_data = serde_json::to_string(job)?;
        
        let mut conn = self.get_connection().await?;
        
        // Add to the queue
        conn.lpush(format!("queue:{}", queue), &job_data).await
            .map_err(|e| AppError::Redis(e.to_string()))?;
        
        Ok(job_id)
    }
    
    async fn dequeue<T: DeserializeOwned + Send + Sync>(&self, queue: &str) -> Result<Option<(String, T)>> {
        let mut conn = self.get_connection().await?;
        
        // Move from queue to processing list with BRPOPLPUSH
        // This is an atomic operation that ensures job safety
        let result: Option<String> = conn.brpoplpush(
            format!("queue:{}", queue),
            format!("processing:{}", queue),
            DEFAULT_POLL_INTERVAL as usize
        ).await.map_err(|e| AppError::Redis(e.to_string()))?;
        
        if let Some(job_data) = result {
            // Generate a job ID for tracking
            let job_id = Uuid::new_v4().to_string();
            
            // Set expiration on the processing item
            conn.set_ex(
                format!("job:{}:{}", queue, job_id),
                &job_data,
                self.visibility_timeout as usize
            ).await.map_err(|e| AppError::Redis(e.to_string()))?;
            
            // Deserialize the job data
            let job: T = serde_json::from_str(&job_data)?;
            
            Ok(Some((job_id, job)))
        } else {
            Ok(None)
        }
    }
    
    async fn complete(&self, queue: &str, job_id: &str) -> Result<()> {
        let mut conn = self.get_connection().await?;
        
        // Remove from processing list and job key
        conn.del(format!("job:{}:{}", queue, job_id)).await
            .map_err(|e| AppError::Redis(e.to_string()))?;
        
        Ok(())
    }
    
    async fn fail(&self, queue: &str, job_id: &str, error: &str) -> Result<()> {
        let mut conn = self.get_connection().await?;
        
        // Get the job data
        let job_data: Option<String> = conn.get(format!("job:{}:{}", queue, job_id)).await
            .map_err(|e| AppError::Redis(e.to_string()))?;
        
        if let Some(job_data) = job_data {
            // Move to failed queue with error information
            let failed_data = format!("{{\"job\":{},\"error\":\"{}\"}}", job_data, error.replace("\"", "\\\""));
            
            conn.lpush(format!("failed:{}", queue), failed_data).await
                .map_err(|e| AppError::Redis(e.to_string()))?;
            
            // Remove from processing
            conn.del(format!("job:{}:{}", queue, job_id)).await
                .map_err(|e| AppError::Redis(e.to_string()))?;
        }
        
        Ok(())
    }
    
    async fn schedule<T: Serialize + Send + Sync>(&self, queue: &str, job: &T, delay_seconds: u64) -> Result<String> {
        let job_id = Uuid::new_v4().to_string();
        let job_data = serde_json::to_string(job)?;
        
        let mut conn = self.get_connection().await?;
        
        // Calculate the execution time
        let execute_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() + delay_seconds;
        
        // Add to the sorted set with score as execution time
        conn.zadd(
            format!("scheduled:{}", queue),
            job_data,
            execute_at as f64
        ).await.map_err(|e| AppError::Redis(e.to_string()))?;
        
        Ok(job_id)
    }
} 