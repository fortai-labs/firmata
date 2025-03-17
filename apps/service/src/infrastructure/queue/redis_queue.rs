use anyhow::Result;
use async_trait::async_trait;
use deadpool_redis::{Config, Pool, Runtime};
use redis::AsyncCommands;
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;
use uuid::Uuid;
use tracing::{debug, info, error, warn};

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
    
    pub async fn get_connection(&self) -> Result<deadpool_redis::Connection> {
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
        let queue_key = format!("queue:{}", queue);
        let processing_key = format!("processing:{}", queue);
        
        // Use BRPOPLPUSH to atomically move a job from the queue to a processing list
        // This ensures that jobs are not lost if the worker crashes
        let job_data: Option<String> = conn.brpoplpush(&queue_key, &processing_key, DEFAULT_POLL_INTERVAL as f64).await
            .map_err(|e| AppError::Redis(e.to_string()))?;
        
        if let Some(job_data) = job_data {
            debug!("Dequeued job data: {}", job_data);
            
            // Parse the job data to extract the job ID
            // The job data might be a JSON string or just a UUID string
            let job_id = if job_data.starts_with('"') && job_data.ends_with('"') {
                // It's a quoted string, remove the quotes
                job_data[1..job_data.len()-1].to_string()
            } else if job_data.starts_with('{') {
                // It's a JSON object
                match serde_json::from_str::<serde_json::Value>(&job_data) {
                    Ok(json) => {
                        if let Some(id) = json.get("job_id").and_then(|id| id.as_str()) {
                            id.to_string()
                        } else {
                            // Fallback to using the job data as the ID
                            job_data.clone()
                        }
                    },
                    Err(_) => {
                        // Fallback to using the job data as the ID
                        job_data.clone()
                    }
                }
            } else {
                // Use the job data as the ID
                job_data.clone()
            };
            
            debug!("Using job ID: {}", job_id);
            
            // Set the job data with an expiration time
            let job_key = format!("job:{}:{}", queue, job_id);
            conn.set_ex(&job_key, &job_data, self.visibility_timeout as u64)
                .await.map_err(|e| AppError::Redis(e.to_string()))?;
            
            // Deserialize the job data
            let job: T = serde_json::from_str(&job_data)
                .map_err(|e| AppError::Internal(format!("Serialization error: {}", e.to_string())))?;
            
            Ok(Some((job_id, job)))
        } else {
            Ok(None)
        }
    }
    
    async fn complete(&self, queue: &str, job_id: &str) -> Result<()> {
        let mut conn = self.get_connection().await?;
        
        debug!("Completing job {} in queue {}", job_id, queue);
        
        // Remove the job from the processing list
        let processing_key = format!("processing:{}", queue);
        let job_key = format!("job:{}:{}", queue, job_id);
        
        // Check if job exists in processing list before removing
        let exists: i32 = conn.exists(&processing_key).await
            .map_err(|e| AppError::Redis(format!("Failed to check if processing list exists: {}", e)))?;
        
        debug!("Processing list {} exists: {}", processing_key, exists > 0);
        
        if exists > 0 {
            // Check if job ID is in the processing list
            let count: i32 = conn.llen(&processing_key).await
                .map_err(|e| AppError::Redis(format!("Failed to get processing list length: {}", e)))?;
            
            debug!("Processing list {} has {} items", processing_key, count);
            
            let items: Vec<String> = conn.lrange(&processing_key, 0, -1).await
                .map_err(|e| AppError::Redis(format!("Failed to get processing list items: {}", e)))?;
            
            debug!("Processing list items: {:?}", items);
            
            let job_in_list = items.contains(&job_id.to_string());
            debug!("Job {} is in processing list: {}", job_id, job_in_list);
        }
        
        // Use a pipeline to execute both commands atomically
        let mut pipe = redis::pipe();
        pipe.lrem(&processing_key, 0, job_id).del(&job_key);
        
        let result: (i32, i32) = pipe.query_async(&mut conn).await
            .map_err(|e| AppError::Redis(format!("Failed to execute pipeline: {}", e)))?;
        
        debug!("Complete job result: removed {} items from processing list, deleted {} job keys", result.0, result.1);
        debug!("Completed job {} in queue {}", job_id, queue);
        
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