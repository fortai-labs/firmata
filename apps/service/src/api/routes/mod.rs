use axum::{
    routing::{get, post, put, delete},
    Router,
};
use sqlx::PgPool;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::trace::TraceLayer;

use crate::api::handlers;
use crate::application::scraper::service::ScraperService;
use crate::application::scraper::worker::ScraperWorker;
use crate::application::scheduler::service::SchedulerService;
use crate::infrastructure::queue::redis_queue::RedisJobQueue;
use crate::infrastructure::storage::s3_client::S3StorageClient;
use crate::infrastructure::queue::redis_client::RedisClient;

pub mod health;

// Define a shared state struct
#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub scraper_service: Arc<ScraperService>,
    pub job_queue: Arc<RedisJobQueue>,
    pub storage_client: Arc<S3StorageClient>,
    pub scraper_worker: Arc<ScraperWorker>,
    pub scheduler: Arc<SchedulerService>,
    pub redis_client: Arc<RedisClient>,
}

pub async fn serve(
    port: u16,
    db_pool: PgPool,
    job_queue: Arc<RedisJobQueue>,
    storage_client: Arc<S3StorageClient>,
    scraper_worker: Arc<ScraperWorker>,
    scheduler: Arc<SchedulerService>,
    redis_client: Arc<RedisClient>,
) -> anyhow::Result<()> {
    // Create services
    let scraper_service = Arc::new(ScraperService::new(db_pool.clone(), job_queue.clone()));
    
    // Create shared state
    let state = AppState {
        db_pool,
        scraper_service,
        job_queue,
        storage_client,
        scraper_worker,
        scheduler,
        redis_client,
    };
    
    // Build the router with state
    let app = Router::new()
        // Health check routes
        .route("/health", get(health::health_check))
        
        // Config routes
        .route("/api/configs", get(handlers::configs::list_configs))
        .route("/api/configs", post(handlers::configs::create_config))
        .route("/api/configs/{id}", get(handlers::configs::get_config))
        .route("/api/configs/{id}", put(handlers::configs::update_config))
        .route("/api/configs/{id}/start", post(handlers::configs::start_job))
        
        // Job routes
        .route("/api/jobs", get(handlers::jobs::list_jobs))
        .route("/api/jobs/{id}", get(handlers::jobs::get_job))
        .route("/api/jobs/{id}/cancel", post(handlers::jobs::cancel_job))
        
        // Page routes
        .route("/api/pages", get(handlers::pages::list_pages))
        .route("/api/pages/{id}", get(handlers::pages::get_page))
        .route("/api/pages/{id}/html", get(handlers::pages::get_page_html))
        .route("/api/pages/{id}/markdown", get(handlers::pages::get_page_markdown))
        
        // Webhook routes
        .route("/api/webhooks", get(handlers::webhooks::list_webhooks))
        .route("/api/webhooks", post(handlers::webhooks::create_webhook))
        .route("/api/webhooks/{id}", get(handlers::webhooks::get_webhook))
        .route("/api/webhooks/{id}", put(handlers::webhooks::update_webhook))
        .route("/api/webhooks/{id}", delete(handlers::webhooks::delete_webhook))
        
        // Analytics routes
        .route("/api/analytics/jobs", get(handlers::analytics::get_job_stats))
        .route("/api/analytics/configs", get(handlers::analytics::get_config_stats))
        .route("/api/analytics/jobs/{id}/timeline", get(handlers::analytics::get_job_timeline))
        
        // Add middleware
        .layer(TraceLayer::new_for_http())
        
        // Add state
        .with_state(state);
    
    // Start the server
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
} 