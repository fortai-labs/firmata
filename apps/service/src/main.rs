mod api;
mod application;
mod config;
mod domain;
mod infrastructure;
mod utils;

use std::sync::Arc;
use tokio::signal;
use tracing::{info, error};

use crate::application::scheduler::service::SchedulerService;
use crate::application::scraper::service::ScraperService;
use crate::application::scraper::worker::ScraperWorker;
use crate::config::AppConfig;
use crate::infrastructure::queue::redis_queue::RedisJobQueue;
use crate::infrastructure::queue::redis_client::RedisClient;
use crate::infrastructure::storage::s3_client::S3StorageClient;
use crate::utils::logging;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    logging::init()?;
    
    info!("Starting legal website scraper service");
    
    // Load configuration
    let config = AppConfig::load()?;
    info!("Configuration loaded");
    
    // Initialize database connection
    let db_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(config.database.max_connections)
        .connect(&config.database.url)
        .await?;
    info!("Database connection established");
    
    // Initialize S3 storage client
    let storage_client = Arc::new(S3StorageClient::new(
        &config.storage.endpoint,
        &config.storage.region,
        &config.storage.bucket,
        &config.storage.access_key,
        &config.storage.secret_key,
    ).await?);
    info!("S3 storage client initialized");
    
    // Initialize Redis job queue
    let job_queue = Arc::new(RedisJobQueue::new(
        &config.redis.url,
        &config.redis.job_queue_name,
    ).await?);
    info!("Redis job queue initialized");
    
    // Initialize Redis client
    let redis_client = Arc::new(RedisClient::new(
        &config.redis.url,
    ).await?);
    info!("Redis client initialized");
    
    // Initialize scraper service
    let scraper_service = Arc::new(ScraperService::new(
        db_pool.clone(),
        job_queue.clone(),
    ));
    info!("Scraper service initialized");
    
    // Initialize scraper worker
    let scraper_worker = Arc::new(ScraperWorker::new(
        db_pool.clone(),
        job_queue.clone(),
        storage_client.clone(),
    ));
    info!("Scraper worker initialized");
    
    // Initialize scheduler service
    let scheduler_service = Arc::new(SchedulerService::new(
        db_pool.clone(),
        scraper_service.clone(),
    ));
    info!("Scheduler service initialized");
    
    // Start the API server
    let api_handle = tokio::spawn(api::routes::serve(
        config.server.clone(),
        db_pool.clone(),
        storage_client.clone(),
        job_queue.clone(),
        scraper_service.clone(),
        scraper_worker.clone(),
        scheduler_service.clone(),
        redis_client.clone(),
    ));
    info!("API server started on {}", config.server.address);
    
    // Start the worker process
    let worker_handle = tokio::spawn(async move {
        if let Err(e) = scraper_worker.start().await {
            error!("Worker process error: {}", e);
        }
    });
    info!("Worker process started");
    
    // Start the scheduler
    let scheduler_handle = tokio::spawn(async move {
        if let Err(e) = scheduler_service.start().await {
            error!("Scheduler error: {}", e);
        }
    });
    info!("Scheduler started");
    
    // Wait for shutdown signal
    match signal::ctrl_c().await {
        Ok(()) => {
            info!("Shutdown signal received, stopping services");
            
            // Cancel all tasks
            api_handle.abort();
            worker_handle.abort();
            scheduler_handle.abort();
            
            info!("All services stopped");
        }
        Err(err) => {
            error!("Error waiting for shutdown signal: {}", err);
        }
    }
    
    Ok(())
} 