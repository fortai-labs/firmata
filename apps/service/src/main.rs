mod api;
mod application;
mod config;
mod domain;
mod infrastructure;
mod utils;

use std::sync::Arc;
use tokio::signal;
use tokio::sync::Mutex;
use tracing::{info, error};

use crate::application::scheduler::service::SchedulerService;
use crate::application::scraper::service::ScraperService;
use crate::application::scraper::worker::ScraperWorker;
use crate::application::scraper::crawler::CrawlerConfig;
use crate::config::settings::AppConfig;
use crate::infrastructure::queue::redis_queue::RedisJobQueue;
use crate::infrastructure::queue::redis_client::RedisClient;
use crate::infrastructure::storage::s3_client::S3StorageClient;
use crate::infrastructure::grpc::markdown_client::MarkdownClient;
use crate::utils::logging;



#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    utils::logging::init_tracing();
    
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
        &config.storage
    ).await?);
    info!("S3 storage client initialized");
    
    // Initialize Redis job queue
    let job_queue = Arc::new(RedisJobQueue::new(
        &config.redis
    ).await?);
    info!("Redis job queue initialized");
    
    // Initialize Redis client
    let redis_client = Arc::new(RedisClient::new(
        &config.redis.url,
    ).await?);
    info!("Redis client initialized");
    
    // Initialize Markdown client
    let markdown_client = Arc::new(MarkdownClient::new(
        &config.grpc
    ).await?);
    info!("Markdown client initialized");
    
    // Create crawler configuration
    let crawler_config = CrawlerConfig {
        max_concurrent_requests: config.scraper.max_concurrent_requests as usize,
        delay_between_requests_ms: config.scraper.request_delay_ms,
        max_retries: config.scraper.max_retries as usize,
        user_agent: config.scraper.default_user_agent.clone(),
        request_timeout_secs: config.scraper.request_timeout_secs,
        respect_robots_txt: config.scraper.respect_robots_txt,
        max_page_size_bytes: config.scraper.max_page_size_bytes,
    };
    
    // Create worker and scheduler with interior mutability
    let scraper_worker = Arc::new(Mutex::new(match ScraperWorker::new(
        db_pool.clone(),
        job_queue.clone(),
        storage_client.clone(),
        markdown_client.clone(),
        crawler_config,
    ) {
        Ok(worker) => worker,
        Err(e) => {
            error!("Failed to initialize scraper worker: {}", e);
            return Err(e);
        }
    }));
    info!("Scraper worker initialized");
    
    // Initialize scheduler service
    let scheduler_service = Arc::new(Mutex::new(SchedulerService::new(
        db_pool.clone(),
        job_queue.clone(),
        config.scheduler.clone(),
    )));
    info!("Scheduler service initialized");
    
    // Start the API server
    let api_handle = tokio::spawn(api::routes::serve(
        config.server.port,
        db_pool.clone(),
        job_queue.clone(),
        storage_client.clone(),
        Arc::new(ScraperWorker::new(
            db_pool.clone(),
            job_queue.clone(),
            storage_client.clone(),
            markdown_client.clone(),
            CrawlerConfig {
                max_concurrent_requests: config.scraper.max_concurrent_requests as usize,
                delay_between_requests_ms: config.scraper.request_delay_ms,
                max_retries: config.scraper.max_retries as usize,
                user_agent: config.scraper.default_user_agent.clone(),
                request_timeout_secs: config.scraper.request_timeout_secs,
                respect_robots_txt: config.scraper.respect_robots_txt,
                max_page_size_bytes: config.scraper.max_page_size_bytes,
            }
        ).unwrap()),
        Arc::new(SchedulerService::new(
            db_pool.clone(),
            job_queue.clone(),
            config.scheduler.clone(),
        )),
        redis_client.clone(),
    ));
    info!("API server started on {}", config.server.address);
    
    // Start the worker process
    let worker_clone = scraper_worker.clone();
    let worker_handle = tokio::spawn(async move {
        if let Err(e) = worker_clone.lock().await.start().await {
            error!("Worker process error: {}", e);
        }
    });
    info!("Worker process started");
    
    // Start the scheduler
    let scheduler_clone = scheduler_service.clone();
    let scheduler_handle = tokio::spawn(async move {
        if let Err(e) = scheduler_clone.lock().await.start().await {
            error!("Scheduler error: {}", e);
        }
    });
    
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