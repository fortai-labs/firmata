use anyhow::Result;
use reqwest::Client as HttpClient;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio::time::sleep;
use tracing::{info, error, warn};
use url::Url;
use uuid::Uuid;

use crate::config::settings::Scraper as ScraperConfig;
use crate::domain::job::{Job, JobStatus};
use crate::domain::page::Page;
use crate::domain::scraper_config::ScraperConfig as ScrapeConfig;
use crate::infrastructure::grpc::markdown_client::MarkdownClient;
use crate::infrastructure::queue::redis_queue::{JobQueue, RedisJobQueue};
use crate::infrastructure::storage::s3_client::{StorageClient, S3StorageClient};
use crate::utils::error::AppError;

pub struct ScraperWorker {
    db_pool: PgPool,
    job_queue: Arc<RedisJobQueue>,
    storage_client: Arc<S3StorageClient>,
    markdown_client: Arc<MarkdownClient>,
    config: ScraperConfig,
    http_client: HttpClient,
    worker_id: String,
    running: bool,
}

impl ScraperWorker {
    pub fn new(
        db_pool: PgPool,
        job_queue: Arc<RedisJobQueue>,
        storage_client: Arc<S3StorageClient>,
        markdown_client: Arc<MarkdownClient>,
        config: ScraperConfig,
    ) -> Self {
        // Create HTTP client with appropriate timeouts
        let http_client = HttpClient::builder()
            .timeout(Duration::from_secs(30))
            .user_agent(&config.default_user_agent)
            .build()
            .expect("Failed to create HTTP client");
        
        // Generate a unique worker ID
        let worker_id = format!("worker-{}", Uuid::new_v4());
        
        Self {
            db_pool,
            job_queue,
            storage_client,
            markdown_client,
            config,
            http_client,
            worker_id,
            running: false,
        }
    }
    
    pub async fn start(&mut self) -> Result<()> {
        if self.running {
            return Ok(());
        }
        
        self.running = true;
        info!("Starting scraper worker: {}", self.worker_id);
        
        while self.running {
            // Try to get a job from the queue
            match self.job_queue.dequeue::<Uuid>("scraper_jobs").await {
                Ok(Some((queue_job_id, job_id))) => {
                    info!("Processing job: {}", job_id);
                    
                    // Process the job
                    if let Err(e) = self.process_job(job_id).await {
                        error!("Error processing job {}: {}", job_id, e);
                        
                        // Mark the job as failed in the database
                        if let Err(db_err) = self.mark_job_failed(job_id, &e.to_string()).await {
                            error!("Error marking job as failed: {}", db_err);
                        }
                        
                        // Mark the job as failed in the queue
                        if let Err(queue_err) = self.job_queue.fail("scraper_jobs", &queue_job_id, &e.to_string()).await {
                            error!("Error marking job as failed in queue: {}", queue_err);
                        }
                    } else {
                        // Complete the job in the queue
                        if let Err(queue_err) = self.job_queue.complete("scraper_jobs", &queue_job_id).await {
                            error!("Error completing job in queue: {}", queue_err);
                        }
                    }
                }
                Ok(None) => {
                    // No jobs available, wait a bit
                    sleep(Duration::from_secs(1)).await;
                }
                Err(e) => {
                    error!("Error dequeueing job: {}", e);
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
        
        // Get the scraper config
        let config = self.get_scraper_config(job.config_id).await?;
        
        // Mark the job as running
        self.mark_job_running(&mut job).await?;
        
        // Create a semaphore to limit concurrent requests
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_requests as usize));
        
        // Start with the base URL
        let base_url = Url::parse(&config.base_url)
            .map_err(|e| AppError::InvalidInput(format!("Invalid base URL: {}", e)))?;
        
        // Create a queue of URLs to crawl
        let mut to_crawl = vec![(base_url.to_string(), 0, None)];
        let mut crawled = HashMap::new();
        
        // Process URLs until the queue is empty or we hit the limit
        while let Some((url, depth, parent_url)) = to_crawl.pop() {
            // Check if we've already crawled this URL
            if crawled.contains_key(&url) {
                continue;
            }
            
            // Check if we've reached the maximum depth
            if depth > config.max_depth {
                continue;
            }
            
            // Check if we've reached the maximum pages per job
            if let Some(max_pages) = config.max_pages_per_job {
                if crawled.len() >= max_pages as usize {
                    break;
                }
            }
            
            // Acquire a permit from the semaphore
            let permit = semaphore.clone().acquire_owned().await?;
            
            // Clone necessary data for the task
            let job_id = job.id;
            let http_client = self.http_client.clone();
            let storage_client = self.storage_client.clone();
            let markdown_client = self.markdown_client.clone();
            let db_pool = self.db_pool.clone();
            let url_clone = url.clone();
            let parent_url_clone = parent_url.clone();
            let include_patterns = config.include_patterns.clone();
            let exclude_patterns = config.exclude_patterns.clone();
            let user_agent = config.user_agent.clone();
            let delay = config.request_delay_ms;
            
            // Spawn a task to crawl the URL
            tokio::spawn(async move {
                // Respect the delay
                sleep(Duration::from_millis(delay as u64)).await;
                
                // Crawl the URL
                match Self::crawl_url(
                    &http_client,
                    &storage_client,
                    &mut markdown_client.clone(),
                    &db_pool,
                    job_id,
                    &url_clone,
                    depth,
                    parent_url_clone,
                    &include_patterns,
                    &exclude_patterns,
                    &user_agent,
                ).await {
                    Ok((page, new_urls)) => {
                        // Update the job stats
                        Self::update_job_stats(&db_pool, job_id, true, false, false).await;
                        
                        // Add new URLs to the queue
                        for new_url in new_urls {
                            to_crawl.push((new_url, depth + 1, Some(url_clone.clone())));
                        }
                    }
                    Err(e) => {
                        error!("Error crawling URL {}: {}", url_clone, e);
                        
                        // Update the job stats
                        Self::update_job_stats(&db_pool, job_id, false, true, false).await;
                    }
                }
                
                // Mark the URL as crawled
                crawled.insert(url_clone, true);
                
                // Release the permit
                drop(permit);
            });
        }
        
        // Wait for all tasks to complete
        while semaphore.available_permits() < config.max_concurrent_requests as usize {
            sleep(Duration::from_millis(100)).await;
        }
        
        // Mark the job as completed
        self.mark_job_completed(&mut job).await?;
        
        Ok(())
    }
    
    async fn crawl_url(
        http_client: &HttpClient,
        storage_client: &S3StorageClient,
        markdown_client: &mut MarkdownClient,
        db_pool: &PgPool,
        job_id: Uuid,
        url: &str,
        depth: i32,
        parent_url: Option<String>,
        include_patterns: &[String],
        exclude_patterns: &[String],
        user_agent: &str,
    ) -> Result<(Page, Vec<String>)> {
        // Check if the URL matches include/exclude patterns
        if !Self::should_crawl_url(url, include_patterns, exclude_patterns) {
            return Err(AppError::InvalidInput(format!("URL does not match patterns: {}", url)).into());
        }
        
        // Normalize the URL
        let normalized_url = Self::normalize_url(url)?;
        
        // Fetch the page
        let response = http_client
            .get(url)
            .header("User-Agent", user_agent)
            .send()
            .await
            .map_err(|e| AppError::Scraper(format!("Failed to fetch URL {}: {}", url, e)))?;
        
        let status = response.status().as_u16() as i32;
        let headers = serde_json::to_value(response.headers().clone())
            .map_err(|e| AppError::Scraper(format!("Failed to serialize headers: {}", e)))?;
        
        // Get the HTML content
        let html = response.text().await
            .map_err(|e| AppError::Scraper(format!("Failed to get response text: {}", e)))?;
        
        // Calculate content hash
        let content_hash = format!("{:x}", md5::compute(&html));
        
        // Create a page record
        let mut page = Page::new(
            job_id,
            url.to_string(),
            normalized_url,
            status,
            headers,
            content_hash,
            depth,
            parent_url,
        );
        
        // Extract title
        if let Some(title) = Self::extract_title(&html) {
            page.set_title(title);
        }
        
        // Upload HTML to storage
        let html_path = storage_client.upload_html(&job_id, url, &html).await?;
        page.set_html_storage_path(html_path);
        
        // Convert to Markdown
        let metadata = HashMap::new();
        match markdown_client.convert_html_to_markdown(&html, url, metadata).await {
            Ok((markdown, extracted_links, _)) => {
                // Upload Markdown to storage
                let markdown_path = storage_client.upload_markdown(&job_id, url, &markdown).await?;
                page.set_markdown_storage_path(markdown_path);
                
                // Save the page to the database
                Self::save_page(db_pool, &page).await?;
                
                // Return the page and extracted links
                Ok((page, extracted_links))
            }
            Err(e) => {
                error!("Failed to convert HTML to Markdown: {}", e);
                
                // Still save the page to the database
                Self::save_page(db_pool, &page).await?;
                
                // Return the page with no links
                Ok((page, vec![]))
            }
        }
    }
    
    fn should_crawl_url(url: &str, include_patterns: &[String], exclude_patterns: &[String]) -> bool {
        // Check exclude patterns first
        for pattern in exclude_patterns {
            if url.contains(pattern) {
                return false;
            }
        }
        
        // If no include patterns, allow all
        if include_patterns.is_empty() {
            return true;
        }
        
        // Check include patterns
        for pattern in include_patterns {
            if url.contains(pattern) {
                return true;
            }
        }
        
        false
    }
    
    fn normalize_url(url: &str) -> Result<String> {
        let parsed = Url::parse(url)
            .map_err(|e| AppError::InvalidInput(format!("Invalid URL: {}", e)))?;
        
        // Remove fragments
        let mut normalized = parsed.clone();
        normalized.set_fragment(None);
        
        Ok(normalized.to_string())
    }
    
    fn extract_title(html: &str) -> Option<String> {
        // Simple regex-based title extraction
        let re = regex::Regex::new(r"<title[^>]*>(.*?)</title>").ok()?;
        re.captures(html)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
    }
    
    async fn save_page(db_pool: &PgPool, page: &Page) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO pages (
                id, job_id, url, normalized_url, content_hash,
                http_status, http_headers, crawled_at,
                html_storage_path, markdown_storage_path,
                title, metadata, error_message, depth, parent_url
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            "#,
            page.id,
            page.job_id,
            page.url,
            page.normalized_url,
            page.content_hash,
            page.http_status,
            page.http_headers,
            page.crawled_at,
            page.html_storage_path,
            page.markdown_storage_path,
            page.title,
            page.metadata,
            page.error_message,
            page.depth,
            page.parent_url
        )
        .execute(db_pool)
        .await?;
        
        Ok(())
    }
    
    async fn update_job_stats(db_pool: &PgPool, job_id: Uuid, crawled: bool, failed: bool, skipped: bool) -> Result<()> {
        let mut query = "UPDATE jobs SET updated_at = NOW()".to_string();
        
        if crawled {
            query.push_str(", pages_crawled = pages_crawled + 1");
        }
        
        if failed {
            query.push_str(", pages_failed = pages_failed + 1");
        }
        
        if skipped {
            query.push_str(", pages_skipped = pages_skipped + 1");
        }
        
        query.push_str(" WHERE id = $1");
        
        sqlx::query(&query)
            .bind(job_id)
            .execute(db_pool)
            .await?;
        
        Ok(())
    }
    
    async fn get_job(&self, job_id: Uuid) -> Result<Job> {
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
    
    async fn get_scraper_config(&self, config_id: Uuid) -> Result<ScrapeConfig> {
        let config = sqlx::query_as!(
            ScrapeConfig,
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
    
    async fn mark_job_running(&self, job: &mut Job) -> Result<()> {
        job.start(self.worker_id.clone());
        
        sqlx::query!(
            r#"
            UPDATE jobs
            SET status = $1, updated_at = $2, started_at = $3, worker_id = $4
            WHERE id = $5
            "#,
            job.status.to_string(),
            job.updated_at,
            job.started_at,
            job.worker_id,
            job.id
        )
        .execute(&self.db_pool)
        .await?;
        
        Ok(())
    }
    
    async fn mark_job_completed(&self, job: &mut Job) -> Result<()> {
        job.complete();
        
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
        
        Ok(())
    }
    
    async fn mark_job_failed(&self, job_id: Uuid, error_message: &str) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE jobs
            SET status = $1, updated_at = NOW(), completed_at = NOW(), error_message = $2
            WHERE id = $3
            "#,
            JobStatus::Failed.to_string(),
            error_message,
            job_id
        )
        .execute(&self.db_pool)
        .await?;
        
        Ok(())
    }
} 