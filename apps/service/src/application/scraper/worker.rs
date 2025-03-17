use anyhow::Result;
use sqlx::PgPool;
use sqlx::Row;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, error, warn, debug};
use uuid::Uuid;
use serde_json;
use chrono::Utc;
use redis::AsyncCommands;
use reqwest;

use crate::domain::job::{Job, JobStatus};
use crate::domain::page::Page;
use crate::domain::scraper_config::ScraperConfig;
use crate::infrastructure::grpc::markdown_client::MarkdownClient;
use crate::infrastructure::queue::redis_queue::{JobQueue, RedisJobQueue};
use crate::infrastructure::storage::s3_client::{StorageClient, S3StorageClient};
use crate::utils::error::AppError;
use crate::application::scraper::crawler::{Crawler, CrawlerConfig};

pub struct ScraperWorker {
    db_pool: PgPool,
    job_queue: Arc<RedisJobQueue>,
    storage_client: Arc<S3StorageClient>,
    markdown_client: Arc<MarkdownClient>,
    crawler: Crawler,
    worker_id: String,
    running: bool,
}

impl ScraperWorker {
    pub fn new(
        db_pool: PgPool,
        job_queue: Arc<RedisJobQueue>,
        storage_client: Arc<S3StorageClient>,
        markdown_client: Arc<MarkdownClient>,
        config: CrawlerConfig,
    ) -> Result<Self> {
        // Create the crawler
        let crawler = Crawler::new(config)?;
        
        // Generate a unique worker ID
        let worker_id = format!("worker-{}", Uuid::new_v4());
        
        Ok(Self {
            db_pool,
            job_queue,
            storage_client,
            markdown_client,
            crawler,
            worker_id,
            running: false,
        })
    }
    
    pub async fn start(&mut self) -> Result<()> {
        if self.running {
            return Ok(());
        }
        
        self.running = true;
        info!("Starting scraper worker: {}", self.worker_id);
        
        while self.running {
            // Try to get a job from the queue
            match self.job_queue.dequeue::<String>("scraper_jobs").await {
                Ok(Some((job_id_str, job_data))) => {
                    info!("Processing job: {}", job_id_str);
                    
                    // Parse the job ID string to UUID
                    let job_id = match Uuid::parse_str(&job_id_str) {
                        Ok(uuid) => uuid,
                        Err(e) => {
                            error!("Invalid job ID format: {}", e);
                            continue;
                        }
                    };
                    
                    // Process the job
                    if let Err(e) = self.process_job(job_id).await {
                        error!("Error processing job {}: {}", job_id, e);
                        
                        // Mark the job as failed
                        if let Err(mark_err) = self.mark_job_failed(job_id, &e.to_string()).await {
                            error!("Error marking job {} as failed: {}", job_id, mark_err);
                        }
                    }
                },
                Ok(None) => {
                    // No jobs in the queue, wait a bit before checking again
                    sleep(Duration::from_secs(1)).await;
                },
                Err(e) => {
                    error!("Error dequeuing job: {}", e);
                    sleep(Duration::from_secs(5)).await;
                }
            }
        }
        
        info!("Scraper worker stopped: {}", self.worker_id);
        Ok(())
    }
    
    pub fn stop(&mut self) {
        info!("Stopping scraper worker: {}", self.worker_id);
        self.running = false;
    }
    
    async fn process_job(&self, job_id: Uuid) -> Result<()> {
        // Get the job from the database
        let mut job = self.get_job(job_id).await?;
        
        // Check if the job is already completed or failed
        if job.status == JobStatus::Completed || job.status == JobStatus::Failed {
            warn!("Job {} is already in terminal state: {:?}", job_id, job.status);
            
            // Complete the job in Redis to remove it from the processing list
            if let Err(e) = self.job_queue.complete("scraper_jobs", &job_id.to_string()).await {
                error!("Error completing job {} in Redis: {}", job_id, e);
            }
            
            return Ok(());
        }
        
        // Mark the job as running
        self.mark_job_running(&mut job).await?;
        
        // Get the scraper configuration
        let config = self.get_scraper_config(job.config_id).await?;
        
        // Parse the include and exclude patterns
        let include_patterns = &config.include_patterns;
        let exclude_patterns = &config.exclude_patterns;
        
        // Create a queue of URLs to crawl
        let mut url_queue = vec![(config.base_url.clone(), 0, None)];
        let mut crawled_urls = HashMap::new();
        
        // Process URLs until the queue is empty or we reach the max pages
        while let Some((url, depth, parent_url)) = url_queue.pop() {
            // Check if we've reached the max pages
            if let Some(max_pages) = config.max_pages_per_job {
                if crawled_urls.len() >= max_pages as usize {
                    info!("Reached max pages ({}) for job {}", max_pages, job_id);
                    break;
                }
            }
            
            // Check if we've reached the max depth
            if depth >= config.max_depth {
                debug!("Reached max depth ({}) for URL: {}", config.max_depth, url);
                continue;
            }
            
            // Check if we've already crawled this URL
            if crawled_urls.contains_key(&url) {
                continue;
            }
            
            // Crawl the URL
            match self.crawler.crawl_url(&url, depth, parent_url.clone(), &include_patterns, &exclude_patterns).await {
                Ok((mut page, discovered_urls)) => {
                    // Set the job ID
                    page.job_id = job_id;
                    
                    // Store the HTML content
                    if page.html_content.is_some() {
                        if let Ok(html_path) = self.store_content(&page, "html").await {
                            page.html_storage_path = Some(html_path);
                            
                            // Convert HTML to Markdown and store it
                            if page.error_message.is_none() {
                                // Use the HTML content directly from the page object
                                if let Some(html_content) = &page.html_content {
                                    if let Ok(markdown) = self.convert_html_to_markdown(html_content, &page.url).await {
                                        if let Ok(markdown_path) = self.store_markdown(&page, &markdown).await {
                                            page.markdown_storage_path = Some(markdown_path);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    
                    // Clear the HTML content before saving to the database
                    page.html_content = None;
                    
                    // Save the page to the database
                    if let Err(e) = self.save_page(&page).await {
                        error!("Error saving page {}: {}", page.url, e);
                    } else {
                        // Update job stats
                        if let Err(e) = self.update_job_stats(&job_id, true, page.error_message.is_some(), false).await {
                            error!("Error updating job stats for {}: {}", job_id, e);
                        }
                        
                        // Mark the URL as crawled
                        crawled_urls.insert(url.to_string(), page.id);
                        
                        // Add discovered URLs to the queue if they match the patterns
                        for discovered_url in discovered_urls {
                            if !crawled_urls.contains_key(&discovered_url) {
                                url_queue.push((discovered_url, depth + 1, Some(url.to_string())));
                            }
                        }
                    }
                },
                Err(e) => {
                    warn!("Error crawling URL {}: {}", url, e);
                    
                    // Create a page with error information
                    let page = Page {
                        id: Uuid::new_v4(),
                        job_id,
                        url: url.to_string(),
                        normalized_url: url.to_string(),
                        content_hash: String::new(),
                        http_status: 0,
                        http_headers: serde_json::Value::Null,
                        crawled_at: chrono::Utc::now(),
                        html_storage_path: None,
                        markdown_storage_path: None,
                        title: None,
                        metadata: serde_json::Value::Null,
                        error_message: Some(e.to_string()),
                        depth,
                        parent_url,
                        html_content: None,
                    };
                    
                    // Save the page to the database
                    if let Err(save_err) = self.save_page(&page).await {
                        error!("Error saving error page {}: {}", page.url, save_err);
                    } else {
                        // Update job stats
                        if let Err(stats_err) = self.update_job_stats(&job_id, true, true, false).await {
                            error!("Error updating job stats for {}: {}", job_id, stats_err);
                        }
                        
                        // Mark the URL as crawled
                        crawled_urls.insert(url.to_string(), page.id);
                    }
                }
            }
        }
        
        // Mark the job as completed
        self.mark_job_completed(&mut job).await?;
        
        // Complete the job in Redis to remove it from the processing list
        debug!("Attempting to complete job {} in Redis queue", job_id);
        match self.job_queue.complete("scraper_jobs", &job_id.to_string()).await {
            Ok(_) => {
                debug!("Successfully completed job {} in Redis queue", job_id);
            },
            Err(e) => {
                error!("Error completing job {} in Redis: {}", job_id, e);
                
                // Try to verify if the job is still in the processing list
                if let Ok(mut conn) = self.job_queue.get_connection().await {
                    let processing_key = format!("processing:{}", "scraper_jobs");
                    if let Ok(items) = conn.lrange::<_, Vec<String>>(&processing_key, 0, -1).await {
                        debug!("Current items in processing list: {:?}", items);
                        let job_id_str = job_id.to_string();
                        if items.contains(&job_id_str) {
                            debug!("Job {} is still in the processing list after completion attempt", job_id);
                            // Try one more time with explicit LREM command
                            let mut pipe = redis::pipe();
                            pipe.cmd("LREM").arg(&processing_key).arg(0).arg(&job_id_str);
                            
                            if let Err(e) = pipe.query_async::<()>(&mut conn).await {
                                error!("Failed to manually remove job from processing list: {}", e);
                            } else {
                                debug!("Manually removed job {} from processing list", job_id);
                            }
                        }
                    }
                }
            }
        }
        
        info!("Job {} completed, crawled {} pages", job_id, crawled_urls.len());
        Ok(())
    }
    
    async fn store_content(&self, page: &Page, content_type: &str) -> Result<String> {
        if content_type == "html" {
            // Check if there was an error crawling the page
            if let Some(error) = &page.error_message {
                // If there was an error, we can't store the content
                debug!("Cannot store content for page with error: {}", error);
                return Err(AppError::InvalidInput(format!("Cannot store content for page with error: {}", error)).into());
            }
            
            // Get the HTML content from the Page object
            if let Some(html_content) = &page.html_content {
                // Store the HTML content in S3 and return the path from the S3 client
                debug!("Attempting to upload HTML content for URL: {}", page.url);
                match self.storage_client.upload_html(&page.job_id, &page.url, html_content).await {
                    Ok(path) => {
                        debug!("Successfully uploaded HTML content to path: {}", path);
                        Ok(path)
                    },
                    Err(e) => {
                        error!("Failed to upload HTML content: {}", e);
                        Err(e)
                    }
                }
            } else {
                // If there's no HTML content, we can't store it
                debug!("Page has no HTML content: {}", page.url);
                Err(AppError::InvalidInput("Page has no HTML content".to_string()).into())
            }
        } else {
            // For other content types, we'll just return a placeholder path
            debug!("Unsupported content type: {}", content_type);
            Err(AppError::InvalidInput(format!("Unsupported content type: {}", content_type)).into())
        }
    }
    
    async fn convert_html_to_markdown(&self, html_content: &str, url: &str) -> Result<String> {
        let metadata = HashMap::new();
        // Clone the Arc to get a new reference to the client
        let client = Arc::clone(&self.markdown_client);
        let (markdown, _, _) = client.convert_html_to_markdown(html_content, url, metadata).await?;
        Ok(markdown)
    }
    
    async fn convert_to_markdown(&self, page: &Page) -> Result<String> {
        // If the page has an HTML storage path, we need to get the HTML content from storage
        if let Some(html_path) = &page.html_storage_path {
            // Get the HTML content from storage
            let html_content = self.storage_client.get_object(html_path).await?;
            
            // Convert HTML to Markdown
            self.convert_html_to_markdown(&html_content, &page.url).await
        } else {
            // If there's no HTML storage path, we can't convert to Markdown
            Err(AppError::InvalidInput("Page has no HTML storage path".to_string()).into())
        }
    }
    
    async fn store_markdown(&self, page: &Page, markdown: &str) -> Result<String> {
        // Store the content and return the path from the S3 client
        debug!("Attempting to upload Markdown content for URL: {}", page.url);
        match self.storage_client.upload_markdown(&page.job_id, &page.url, &markdown).await {
            Ok(path) => {
                debug!("Successfully uploaded Markdown content to path: {}", path);
                Ok(path)
            },
            Err(e) => {
                error!("Failed to upload Markdown content: {}", e);
                Err(e)
            }
        }
    }
    
    async fn save_page(&self, page: &Page) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO pages (
                id, job_id, url, normalized_url, content_hash, http_status, http_headers,
                crawled_at, html_storage_path, markdown_storage_path, title, metadata,
                error_message, depth, parent_url
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15
            )
            "#
        )
        .bind(page.id)
        .bind(page.job_id)
        .bind(&page.url)
        .bind(&page.normalized_url)
        .bind(&page.content_hash)
        .bind(page.http_status)
        .bind(&page.http_headers)
        .bind(page.crawled_at)
        .bind(&page.html_storage_path)
        .bind(&page.markdown_storage_path)
        .bind(&page.title)
        .bind(&page.metadata)
        .bind(&page.error_message)
        .bind(page.depth)
        .bind(&page.parent_url)
        .execute(&self.db_pool)
        .await?;
        
        Ok(())
    }
    
    async fn update_job_stats(&self, job_id: &Uuid, crawled: bool, failed: bool, skipped: bool) -> Result<()> {
        // Get the current job stats
        let row = sqlx::query(
            r#"
            SELECT pages_crawled, pages_failed, pages_skipped
            FROM jobs
            WHERE id = $1
            "#
        )
        .bind(job_id)
        .fetch_one(&self.db_pool)
        .await?;
        
        // Calculate the new stats
        let pages_crawled: i32 = row.get("pages_crawled");
        let pages_failed: i32 = row.get("pages_failed");
        let pages_skipped: i32 = row.get("pages_skipped");
        
        let new_crawled = if crawled { pages_crawled + 1 } else { pages_crawled };
        let new_failed = if failed { pages_failed + 1 } else { pages_failed };
        let new_skipped = if skipped { pages_skipped + 1 } else { pages_skipped };
        
        // Update the job stats
        sqlx::query(
            r#"
            UPDATE jobs
            SET pages_crawled = $1, pages_failed = $2, pages_skipped = $3, updated_at = $4
            WHERE id = $5
            "#
        )
        .bind(new_crawled)
        .bind(new_failed)
        .bind(new_skipped)
        .bind(Utc::now())
        .bind(job_id)
        .execute(&self.db_pool)
        .await?;
        
        Ok(())
    }
    
    async fn get_job(&self, job_id: Uuid) -> Result<Job> {
        let row = sqlx::query(
            r#"
            SELECT 
                id, config_id, status, started_at, completed_at, error_message,
                pages_crawled, pages_failed, pages_skipped, created_at, updated_at,
                next_run_at, worker_id, metadata
            FROM jobs
            WHERE id = $1
            "#
        )
        .bind(job_id)
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Job with ID {} not found", job_id)))?;

        // Parse the status string to JobStatus enum
        let status_str: String = row.get("status");
        let status = match status_str.to_lowercase().as_str() {
            "pending" => JobStatus::Pending,
            "running" => JobStatus::Running,
            "completed" => JobStatus::Completed,
            "failed" => JobStatus::Failed,
            "cancelled" => JobStatus::Cancelled,
            _ => JobStatus::Unknown,
        };

        Ok(Job {
            id: row.get("id"),
            config_id: row.get("config_id"),
            status,
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            started_at: row.get("started_at"),
            completed_at: row.get("completed_at"),
            error_message: row.get("error_message"),
            pages_crawled: row.get("pages_crawled"),
            pages_failed: row.get("pages_failed"),
            pages_skipped: row.get("pages_skipped"),
            next_run_at: row.get("next_run_at"),
            worker_id: row.get("worker_id"),
            metadata: row.get("metadata"),
        })
    }
    
    async fn get_scraper_config(&self, config_id: Uuid) -> Result<ScraperConfig> {
        let row = sqlx::query(
            r#"
            SELECT 
                id, name, description, base_url, include_patterns,
                exclude_patterns, max_depth, max_pages_per_job, respect_robots_txt,
                user_agent, request_delay_ms, max_concurrent_requests, schedule,
                headers, created_at, updated_at, active
            FROM scraper_configs
            WHERE id = $1
            "#
        )
        .bind(config_id)
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Scraper config with ID {} not found", config_id)))?;

        Ok(ScraperConfig {
            id: row.get("id"),
            name: row.get("name"),
            description: row.get("description"),
            base_url: row.get("base_url"),
            include_patterns: row.get("include_patterns"),
            exclude_patterns: row.get("exclude_patterns"),
            max_depth: row.get("max_depth"),
            max_pages_per_job: row.get("max_pages_per_job"),
            respect_robots_txt: row.get("respect_robots_txt"),
            user_agent: row.get("user_agent"),
            request_delay_ms: row.get("request_delay_ms"),
            max_concurrent_requests: row.get("max_concurrent_requests"),
            schedule: row.get("schedule"),
            headers: row.get("headers"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            active: row.get("active"),
        })
    }
    
    async fn mark_job_running(&self, job: &mut Job) -> Result<()> {
        // Update the job status in the database
        let worker_id = self.worker_id.clone();
        let now = Utc::now();
        
        sqlx::query(
            r#"
            UPDATE jobs
            SET status = $1, started_at = $2, updated_at = $3, worker_id = $4
            WHERE id = $5
            "#
        )
        .bind("running")
        .bind(now)
        .bind(now)
        .bind(worker_id)
        .bind(job.id)
        .execute(&self.db_pool)
        .await?;
        
        // Update the job object
        job.status = JobStatus::Running;
        job.started_at = Some(now);
        job.updated_at = now;
        job.worker_id = Some(self.worker_id.clone());
        
        Ok(())
    }
    
    async fn mark_job_completed(&self, job: &mut Job) -> Result<()> {
        // Update the job status in the database
        let now = Utc::now();
        
        sqlx::query(
            r#"
            UPDATE jobs
            SET status = $1, completed_at = $2, updated_at = $3
            WHERE id = $4
            "#
        )
        .bind("completed")
        .bind(now)
        .bind(now)
        .bind(job.id)
        .execute(&self.db_pool)
        .await?;
        
        // Update the job object
        job.status = JobStatus::Completed;
        job.completed_at = Some(now);
        job.updated_at = now;
        
        Ok(())
    }
    
    async fn mark_job_failed(&self, job_id: Uuid, error_message: &str) -> Result<()> {
        // Update the job status in the database
        let now = Utc::now();
        
        sqlx::query(
            r#"
            UPDATE jobs
            SET status = $1, error_message = $2, completed_at = $3, updated_at = $4
            WHERE id = $5
            "#
        )
        .bind("failed")
        .bind(error_message)
        .bind(now)
        .bind(now)
        .bind(job_id)
        .execute(&self.db_pool)
        .await?;
        
        Ok(())
    }
} 