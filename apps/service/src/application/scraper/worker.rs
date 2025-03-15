use anyhow::Result;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, error, warn};
use uuid::Uuid;

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
            match self.job_queue.dequeue().await {
                Ok(Some(job_id)) => {
                    info!("Processing job: {}", job_id);
                    
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
        if job.status == JobStatus::Completed as i32 || job.status == JobStatus::Failed as i32 {
            warn!("Job {} is already in terminal state: {}", job_id, job.status);
            return Ok(());
        }
        
        // Mark the job as running
        self.mark_job_running(&mut job).await?;
        
        // Get the scraper configuration
        let config = self.get_scraper_config(job.config_id).await?;
        
        // Parse the include and exclude patterns
        let include_patterns = config.include_patterns.unwrap_or_default();
        let exclude_patterns = config.exclude_patterns.unwrap_or_default();
        
        // Create a queue of URLs to crawl
        let mut url_queue = vec![(config.start_url.clone(), 0, None)];
        let mut crawled_urls = HashMap::new();
        
        // Process URLs until the queue is empty or we reach the max pages
        while let Some((url, depth, parent_url)) = url_queue.pop() {
            // Check if we've reached the max pages
            if let Some(max_pages) = config.max_pages {
                if crawled_urls.len() >= max_pages as usize {
                    info!("Reached max pages ({}) for job {}", max_pages, job_id);
                    break;
                }
            }
            
            // Check if we've reached the max depth
            if let Some(max_depth) = config.max_depth {
                if depth > max_depth {
                    continue;
                }
            }
            
            // Check if we've already crawled this URL
            if crawled_urls.contains_key(&url) {
                continue;
            }
            
            // Crawl the URL
            match self.crawler.crawl_url(&url, depth, parent_url, &include_patterns, &exclude_patterns).await {
                Ok((mut page, discovered_urls)) => {
                    // Set the job ID
                    page.job_id = job_id;
                    
                    // Store the HTML content
                    if let Ok(html_path) = self.store_content(&page, "html").await {
                        page.html_storage_path = Some(html_path);
                    }
                    
                    // Convert HTML to Markdown and store it
                    if page.error_message.is_none() {
                        if let Ok(markdown) = self.convert_to_markdown(&page).await {
                            if let Ok(markdown_path) = self.store_markdown(&page, &markdown).await {
                                page.markdown_storage_path = Some(markdown_path);
                            }
                        }
                    }
                    
                    // Save the page to the database
                    if let Err(e) = self.save_page(&page).await {
                        error!("Error saving page {}: {}", page.url, e);
                    } else {
                        // Update job stats
                        if let Err(e) = self.update_job_stats(&job_id, true, page.error_message.is_some(), false).await {
                            error!("Error updating job stats for {}: {}", job_id, e);
                        }
                        
                        // Mark the URL as crawled
                        crawled_urls.insert(url.clone(), page.id);
                        
                        // Add discovered URLs to the queue if they match the patterns
                        for discovered_url in discovered_urls {
                            if !crawled_urls.contains_key(&discovered_url) {
                                url_queue.push((discovered_url, depth + 1, Some(url.clone())));
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
                        url: url.clone(),
                        normalized_url: url.clone(),
                        content_hash: None,
                        http_status: 0,
                        http_headers: None,
                        crawled_at: chrono::Utc::now(),
                        html_storage_path: None,
                        markdown_storage_path: None,
                        title: None,
                        metadata: None,
                        error_message: Some(e.to_string()),
                        depth,
                        parent_url,
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
                        crawled_urls.insert(url.clone(), page.id);
                    }
                }
            }
        }
        
        // Mark the job as completed
        self.mark_job_completed(&mut job).await?;
        
        info!("Job {} completed, crawled {} pages", job_id, crawled_urls.len());
        Ok(())
    }
    
    async fn store_content(&self, page: &Page, content_type: &str) -> Result<String> {
        // Get the HTML content from the page URL
        let content = match content_type {
            "html" => {
                // The HTML content is already fetched by the crawler
                // This would be passed from the crawler in a real implementation
                "".to_string()
            },
            _ => return Err(AppError::InvalidInput(format!("Invalid content type: {}", content_type)).into()),
        };
        
        // Generate a storage path
        let storage_path = format!("jobs/{}/pages/{}/{}.{}", 
                                  page.job_id, 
                                  page.id,
                                  content_type,
                                  content_type);
        
        // Store the content
        self.storage_client.put_object(&storage_path, content.as_bytes()).await?;
        
        Ok(storage_path)
    }
    
    async fn convert_to_markdown(&self, page: &Page) -> Result<String> {
        // Get the HTML content
        let html_path = page.html_storage_path.as_ref()
            .ok_or_else(|| AppError::InvalidInput("Page has no HTML storage path".to_string()))?;
        
        let html_content = self.storage_client.get_object(html_path).await?;
        
        // Convert HTML to Markdown
        let markdown = self.markdown_client.convert_html_to_markdown(&html_content).await?;
        
        Ok(markdown)
    }
    
    async fn store_markdown(&self, page: &Page, markdown: &str) -> Result<String> {
        // Generate a storage path
        let storage_path = format!("jobs/{}/pages/{}/markdown.md", page.job_id, page.id);
        
        // Store the content
        self.storage_client.put_object(&storage_path, markdown.as_bytes()).await?;
        
        Ok(storage_path)
    }
    
    async fn save_page(&self, page: &Page) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO pages (
                id, job_id, url, normalized_url, content_hash, http_status, http_headers,
                crawled_at, html_storage_path, markdown_storage_path, title, metadata,
                error_message, depth, parent_url
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15
            )
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
        .execute(&self.db_pool)
        .await?;
        
        Ok(())
    }
    
    async fn update_job_stats(&self, job_id: &Uuid, crawled: bool, failed: bool, skipped: bool) -> Result<()> {
        let mut query = "UPDATE jobs SET ".to_string();
        let mut updates = Vec::new();
        
        if crawled {
            updates.push("pages_crawled = pages_crawled + 1".to_string());
        }
        
        if failed {
            updates.push("pages_failed = pages_failed + 1".to_string());
        }
        
        if skipped {
            updates.push("pages_skipped = pages_skipped + 1".to_string());
        }
        
        query.push_str(&updates.join(", "));
        query.push_str(&format!(" WHERE id = '{}'", job_id));
        
        sqlx::query(&query)
            .execute(&self.db_pool)
            .await?;
        
        Ok(())
    }
    
    async fn get_job(&self, job_id: Uuid) -> Result<Job> {
        let job = sqlx::query_as!(
            Job,
            r#"
            SELECT 
                id, config_id, status, started_at, completed_at, error_message,
                pages_crawled, pages_failed, pages_skipped, created_at, updated_at
            FROM jobs
            WHERE id = $1
            "#,
            job_id
        )
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Job not found: {}", job_id)))?;
        
        Ok(job)
    }
    
    async fn get_scraper_config(&self, config_id: Uuid) -> Result<ScraperConfig> {
        let config = sqlx::query_as!(
            ScraperConfig,
            r#"
            SELECT 
                id, name, description, start_url, include_patterns as "include_patterns: Vec<String>",
                exclude_patterns as "exclude_patterns: Vec<String>", max_depth, max_pages,
                respect_robots_txt, follow_links, user_agent, created_at, updated_at
            FROM configs
            WHERE id = $1
            "#,
            config_id
        )
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Scraper config not found: {}", config_id)))?;
        
        Ok(config)
    }
    
    async fn mark_job_running(&self, job: &mut Job) -> Result<()> {
        // Update the job status in the database
        sqlx::query!(
            r#"
            UPDATE jobs
            SET status = $1, started_at = $2, updated_at = NOW()
            WHERE id = $3
            "#,
            JobStatus::Running as i32,
            chrono::Utc::now(),
            job.id
        )
        .execute(&self.db_pool)
        .await?;
        
        // Update the job object
        job.status = JobStatus::Running as i32;
        job.started_at = Some(chrono::Utc::now());
        
        Ok(())
    }
    
    async fn mark_job_completed(&self, job: &mut Job) -> Result<()> {
        // Update the job status in the database
        sqlx::query!(
            r#"
            UPDATE jobs
            SET status = $1, completed_at = $2, updated_at = NOW()
            WHERE id = $3
            "#,
            JobStatus::Completed as i32,
            chrono::Utc::now(),
            job.id
        )
        .execute(&self.db_pool)
        .await?;
        
        // Update the job object
        job.status = JobStatus::Completed as i32;
        job.completed_at = Some(chrono::Utc::now());
        
        Ok(())
    }
    
    async fn mark_job_failed(&self, job_id: Uuid, error_message: &str) -> Result<()> {
        // Update the job status in the database
        sqlx::query!(
            r#"
            UPDATE jobs
            SET status = $1, error_message = $2, completed_at = $3, updated_at = NOW()
            WHERE id = $4
            "#,
            JobStatus::Failed as i32,
            error_message,
            chrono::Utc::now(),
            job_id
        )
        .execute(&self.db_pool)
        .await?;
        
        Ok(())
    }
} 