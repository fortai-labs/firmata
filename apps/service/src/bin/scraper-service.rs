use anyhow::Result;
use std::sync::Arc;
use tracing::{info, error};

use fortai_scraper_service::application::scraper::service::ScraperService;
use fortai_scraper_service::application::scraper::worker::ScraperWorker;
use fortai_scraper_service::application::scheduler::service::SchedulerService;
use fortai_scraper_service::application::scraper::crawler::CrawlerConfig;
use fortai_scraper_service::config::settings::Settings;
use fortai_scraper_service::infrastructure::db::postgres::init_db_pool;
use fortai_scraper_service::infrastructure::grpc::markdown_client::MarkdownClient;
use fortai_scraper_service::infrastructure::queue::redis_queue::RedisJobQueue;
use fortai_scraper_service::infrastructure::storage::s3_client::S3StorageClient;
use fortai_scraper_service::api::server::start_server;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    info!("Starting Fortai Scraper Service");
    
    // Load configuration
    let settings = Settings::new()?;
    info!("Configuration loaded");
    
    // Initialize database connection
    let db_pool = init_db_pool(&settings.database).await?;
    info!("Database connection established");
    
    // Initialize S3 storage client
    let storage_client = Arc::new(S3StorageClient::new(&settings.storage).await?);
    info!("S3 storage client initialized");
    
    // Initialize Redis job queue
    let job_queue = Arc::new(RedisJobQueue::new(&settings.redis).await?);
    info!("Redis job queue initialized");
    
    // Initialize Markdown conversion client
    let markdown_client = Arc::new(MarkdownClient::new(&settings.markdown).await?);
    info!("Markdown conversion client initialized");
    
    // Create crawler configuration
    let crawler_config = CrawlerConfig {
        user_agent: settings.crawler.user_agent.clone(),
        request_timeout: settings.crawler.request_timeout,
        max_concurrent_requests: settings.crawler.max_concurrent_requests,
        request_delay: settings.crawler.request_delay,
        respect_robots_txt: settings.crawler.respect_robots_txt,
    };
    
    // Create scraper service
    let scraper_service = Arc::new(ScraperService::new(
        db_pool.clone(),
        job_queue.clone(),
        storage_client.clone(),
    )?);
    info!("Scraper service initialized");
    
    // Create scheduler service
    let scheduler_service = Arc::new(SchedulerService::new(
        db_pool.clone(),
        job_queue.clone(),
    )?);
    info!("Scheduler service initialized");
    
    // Create scraper worker
    let mut worker = ScraperWorker::new(
        db_pool.clone(),
        job_queue.clone(),
        storage_client.clone(),
        markdown_client.clone(),
        crawler_config,
    )?;
    info!("Scraper worker initialized");
    
    // Start the API server
    let server_handle = tokio::spawn(async move {
        if let Err(e) = start_server(
            settings.server.host.clone(),
            settings.server.port,
            db_pool,
            scraper_service,
            scheduler_service,
        ).await {
            error!("Error starting API server: {}", e);
        }
    });
    info!("API server started on {}:{}", settings.server.host, settings.server.port);
    
    // Start the worker
    let worker_handle = tokio::spawn(async move {
        if let Err(e) = worker.start().await {
            error!("Error starting worker: {}", e);
        }
    });
    info!("Worker process started");
    
    // Wait for both tasks to complete
    let _ = tokio::try_join!(
        async { server_handle.await.map_err(|e| anyhow::anyhow!(e)) },
        async { worker_handle.await.map_err(|e| anyhow::anyhow!(e)) },
    );
    
    Ok(())
} 