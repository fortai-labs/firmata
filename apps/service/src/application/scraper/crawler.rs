use anyhow::Result;
use reqwest::{Client as HttpClient, Response, StatusCode};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, Semaphore};
use tokio::time::sleep;
use tracing::{debug, error, info, warn};
use url::Url;
use regex::Regex;
use md5;

use crate::domain::page::Page;
use crate::utils::error::AppError;

/// Configuration for the crawler
#[derive(Debug, Clone)]
pub struct CrawlerConfig {
    /// Maximum number of concurrent requests
    pub max_concurrent_requests: usize,
    /// Delay between requests to the same domain (in milliseconds)
    pub delay_between_requests_ms: u64,
    /// Maximum number of retries for failed requests
    pub max_retries: usize,
    /// User agent to use for requests
    pub user_agent: String,
    /// Timeout for requests (in seconds)
    pub request_timeout_secs: u64,
    /// Whether to respect robots.txt
    pub respect_robots_txt: bool,
    /// Maximum size of a page to download (in bytes)
    pub max_page_size_bytes: usize,
}

impl Default for CrawlerConfig {
    fn default() -> Self {
        Self {
            max_concurrent_requests: 10,
            delay_between_requests_ms: 1000,
            max_retries: 3,
            user_agent: "FortaiLegalScraper/1.0".to_string(),
            request_timeout_secs: 30,
            respect_robots_txt: true,
            max_page_size_bytes: 10 * 1024 * 1024, // 10 MB
        }
    }
}

/// Crawler for scraping websites
pub struct Crawler {
    http_client: HttpClient,
    config: CrawlerConfig,
    domain_delays: Arc<Mutex<HashMap<String, Instant>>>,
    semaphore: Arc<Semaphore>,
    robots_txt_cache: Arc<Mutex<HashMap<String, RobotsTxt>>>,
}

impl Crawler {
    /// Create a new crawler with the given configuration
    pub fn new(config: CrawlerConfig) -> Result<Self> {
        let http_client = HttpClient::builder()
            .timeout(Duration::from_secs(config.request_timeout_secs))
            .user_agent(&config.user_agent)
            .build()?;
        
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_requests));
        
        Ok(Self {
            http_client,
            config,
            domain_delays: Arc::new(Mutex::new(HashMap::new())),
            semaphore,
            robots_txt_cache: Arc::new(Mutex::new(HashMap::new())),
        })
    }
    
    /// Crawl a URL and return the page and any discovered URLs
    pub async fn crawl_url(
        &self,
        url: &str,
        depth: i32,
        parent_url: Option<String>,
        include_patterns: &[String],
        exclude_patterns: &[String],
    ) -> Result<(Page, Vec<String>)> {
        // Check if we should crawl this URL based on patterns
        if !self.should_crawl_url(url, include_patterns, exclude_patterns) {
            return Err(AppError::InvalidInput(format!("URL does not match include/exclude patterns: {}", url)).into());
        }
        
        // Normalize the URL
        let normalized_url = self.normalize_url(url)?;
        
        // Check robots.txt if enabled
        if self.config.respect_robots_txt {
            let domain = self.extract_domain(&normalized_url)?;
            let path = self.extract_path(&normalized_url)?;
            
            if !self.is_allowed_by_robots_txt(&domain, &path).await? {
                return Err(AppError::InvalidInput(format!("URL is disallowed by robots.txt: {}", url)).into());
            }
        }
        
        // Acquire semaphore to limit concurrent requests
        let _permit = self.semaphore.clone().acquire_owned().await?;
        
        // Apply rate limiting for the domain
        let domain = self.extract_domain(&normalized_url)?;
        self.apply_rate_limiting(&domain).await;
        
        // Make the HTTP request with retries
        let response = self.make_request_with_retries(&normalized_url).await?;
        
        // Process the response
        let (page, discovered_urls) = self.process_response(
            response,
            &normalized_url,
            depth,
            parent_url,
        ).await?;
        
        Ok((page, discovered_urls))
    }
    
    /// Check if a URL should be crawled based on include/exclude patterns
    pub fn should_crawl_url(&self, url: &str, include_patterns: &[String], exclude_patterns: &[String]) -> bool {
        // If include patterns are specified, at least one must match
        if !include_patterns.is_empty() {
            let matches_include = include_patterns.iter().any(|pattern| {
                let re = Regex::new(pattern).unwrap_or_else(|_| Regex::new(".*").unwrap());
                re.is_match(url)
            });
            
            if !matches_include {
                return false;
            }
        }
        
        // If exclude patterns are specified, none must match
        if !exclude_patterns.is_empty() {
            let matches_exclude = exclude_patterns.iter().any(|pattern| {
                let re = Regex::new(pattern).unwrap_or_else(|_| Regex::new("$^").unwrap());
                re.is_match(url)
            });
            
            if matches_exclude {
                return false;
            }
        }
        
        true
    }
    
    /// Normalize a URL by removing fragments and normalizing the path
    pub fn normalize_url(&self, url: &str) -> Result<String> {
        let parsed_url = Url::parse(url).map_err(|e| AppError::InvalidInput(format!("Invalid URL: {}, error: {}", url, e)))?;
        
        // Create a new URL without the fragment
        let mut normalized = parsed_url.clone();
        normalized.set_fragment(None);
        
        // Ensure the path ends with a trailing slash if it's just a domain
        if normalized.path() == "" || normalized.path() == "/" {
            normalized.set_path("/");
        }
        
        Ok(normalized.to_string())
    }
    
    /// Extract the domain from a URL
    fn extract_domain(&self, url: &str) -> Result<String> {
        let parsed_url = Url::parse(url).map_err(|e| AppError::InvalidInput(format!("Invalid URL: {}, error: {}", url, e)))?;
        
        parsed_url.host_str()
            .map(|h| h.to_string())
            .ok_or_else(|| AppError::InvalidInput(format!("URL has no host: {}", url)).into())
    }
    
    /// Extract the path from a URL
    fn extract_path(&self, url: &str) -> Result<String> {
        let parsed_url = Url::parse(url).map_err(|e| AppError::InvalidInput(format!("Invalid URL: {}, error: {}", url, e)))?;
        
        Ok(parsed_url.path().to_string())
    }
    
    /// Apply rate limiting for a domain
    async fn apply_rate_limiting(&self, domain: &str) {
        let delay_duration = Duration::from_millis(self.config.delay_between_requests_ms);
        
        let mut domain_delays = self.domain_delays.lock().await;
        
        if let Some(last_request_time) = domain_delays.get(domain) {
            let elapsed = last_request_time.elapsed();
            
            if elapsed < delay_duration {
                let sleep_duration = delay_duration - elapsed;
                // Release the lock before sleeping
                drop(domain_delays);
                
                debug!("Rate limiting: sleeping for {}ms before requesting {}", sleep_duration.as_millis(), domain);
                sleep(sleep_duration).await;
                
                // Reacquire the lock to update the last request time
                domain_delays = self.domain_delays.lock().await;
            }
        }
        
        // Update the last request time
        domain_delays.insert(domain.to_string(), Instant::now());
    }
    
    /// Make an HTTP request with retries
    async fn make_request_with_retries(&self, url: &str) -> Result<Response> {
        let mut retries = 0;
        let max_retries = self.config.max_retries;
        
        loop {
            match self.http_client.get(url).send().await {
                Ok(response) => {
                    // Check if we got a server error (5xx)
                    if response.status().is_server_error() {
                        if retries < max_retries {
                            retries += 1;
                            let backoff_duration = Duration::from_millis(2u64.pow(retries as u32) * 100);
                            warn!("Server error {} for URL: {}, retrying in {}ms ({}/{})", 
                                  response.status(), url, backoff_duration.as_millis(), retries, max_retries);
                            sleep(backoff_duration).await;
                            continue;
                        } else {
                            return Err(AppError::ExternalServiceError(
                                format!("Server error after {} retries: {}", max_retries, response.status())
                            ).into());
                        }
                    }
                    
                    return Ok(response);
                },
                Err(e) => {
                    if retries < max_retries {
                        retries += 1;
                        let backoff_duration = Duration::from_millis(2u64.pow(retries as u32) * 100);
                        warn!("Request error for URL: {}, retrying in {}ms ({}/{}): {}", 
                              url, backoff_duration.as_millis(), retries, max_retries, e);
                        sleep(backoff_duration).await;
                    } else {
                        return Err(AppError::ExternalServiceError(
                            format!("Request failed after {} retries: {}", max_retries, e)
                        ).into());
                    }
                }
            }
        }
    }
    
    /// Process an HTTP response and extract page information and discovered URLs
    async fn process_response(
        &self,
        response: Response,
        url: &str,
        depth: i32,
        parent_url: Option<String>,
    ) -> Result<(Page, Vec<String>)> {
        let status = response.status();
        let headers = response.headers().clone();
        
        // Convert headers to a JSON value
        let headers_json = serde_json::to_value(
            headers.iter()
                .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                .collect::<HashMap<String, String>>()
        )?;
        
        // Check content length if available
        if let Some(content_length) = headers.get("content-length") {
            if let Ok(length) = content_length.to_str()?.parse::<usize>() {
                if length > self.config.max_page_size_bytes {
                    return Err(AppError::InvalidInput(
                        format!("Content too large: {} bytes (max: {} bytes)", length, self.config.max_page_size_bytes)
                    ).into());
                }
            }
        }
        
        // Get the response body
        let body = response.text().await?;
        
        // Calculate content hash
        let content_hash = format!("{:x}", md5::compute(&body));
        
        // Extract title
        let title = self.extract_title(&body);
        
        // Extract links if status is OK
        let discovered_urls = if status == StatusCode::OK {
            self.extract_links(&body, url)
        } else {
            Vec::new()
        };
        
        // Create the page object
        let page = Page {
            id: uuid::Uuid::new_v4(),
            job_id: uuid::Uuid::nil(), // This will be set by the caller
            url: url.to_string(),
            normalized_url: self.normalize_url(url)?,
            content_hash: Some(content_hash),
            http_status: status.as_u16() as i32,
            http_headers: Some(headers_json),
            crawled_at: chrono::Utc::now(),
            html_storage_path: None, // This will be set by the caller
            markdown_storage_path: None, // This will be set by the caller
            title,
            metadata: Some(serde_json::json!({
                "content_length": body.len(),
                "content_type": headers.get("content-type").and_then(|v| v.to_str().ok()).unwrap_or(""),
            })),
            error_message: if status.is_client_error() || status.is_server_error() {
                Some(format!("HTTP error: {}", status))
            } else {
                None
            },
            depth,
            parent_url,
        };
        
        Ok((page, discovered_urls))
    }
    
    /// Extract the title from HTML content
    fn extract_title(&self, html: &str) -> Option<String> {
        let re = Regex::new(r"<title[^>]*>(.*?)</title>").ok()?;
        re.captures(html)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().trim().to_string())
    }
    
    /// Extract links from HTML content
    fn extract_links(&self, html: &str, base_url: &str) -> Vec<String> {
        let base_url = Url::parse(base_url).unwrap_or_else(|_| {
            // Fallback to a dummy URL if parsing fails
            Url::parse("http://example.com").unwrap()
        });
        
        let re = Regex::new(r#"<a[^>]+href=["']([^"']+)["']"#).unwrap_or_else(|_| Regex::new("").unwrap());
        
        let mut urls = HashSet::new();
        
        for cap in re.captures_iter(html) {
            if let Some(href) = cap.get(1) {
                let href = href.as_str();
                
                // Skip empty links, javascript, mailto, tel, etc.
                if href.is_empty() || 
                   href.starts_with("javascript:") || 
                   href.starts_with("mailto:") || 
                   href.starts_with("tel:") || 
                   href.starts_with("#") {
                    continue;
                }
                
                // Resolve relative URLs
                if let Ok(absolute_url) = base_url.join(href) {
                    // Only keep http and https URLs
                    if absolute_url.scheme() == "http" || absolute_url.scheme() == "https" {
                        urls.insert(absolute_url.to_string());
                    }
                }
            }
        }
        
        urls.into_iter().collect()
    }
    
    /// Check if a URL is allowed by robots.txt
    async fn is_allowed_by_robots_txt(&self, domain: &str, path: &str) -> Result<bool> {
        let mut robots_cache = self.robots_txt_cache.lock().await;
        
        // Check if we have cached robots.txt for this domain
        if !robots_cache.contains_key(domain) {
            // Fetch robots.txt
            let robots_url = format!("http://{}/robots.txt", domain);
            
            match self.http_client.get(&robots_url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        let content = response.text().await?;
                        let robots = RobotsTxt::parse(&content, &self.config.user_agent);
                        robots_cache.insert(domain.to_string(), robots);
                    } else {
                        // If robots.txt doesn't exist or can't be fetched, assume everything is allowed
                        robots_cache.insert(domain.to_string(), RobotsTxt::allow_all());
                    }
                },
                Err(_) => {
                    // If robots.txt can't be fetched, assume everything is allowed
                    robots_cache.insert(domain.to_string(), RobotsTxt::allow_all());
                }
            }
        }
        
        // Check if the path is allowed
        if let Some(robots) = robots_cache.get(domain) {
            Ok(robots.is_allowed(path))
        } else {
            // This should never happen, but just in case
            Ok(true)
        }
    }
}

/// Simple robots.txt parser
#[derive(Debug, Clone)]
struct RobotsTxt {
    allow_rules: Vec<String>,
    disallow_rules: Vec<String>,
}

impl RobotsTxt {
    /// Parse robots.txt content
    fn parse(content: &str, user_agent: &str) -> Self {
        let mut current_agent = String::new();
        let mut allow_rules = Vec::new();
        let mut disallow_rules = Vec::new();
        
        // Split content into lines
        for line in content.lines() {
            let line = line.trim();
            
            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            // Parse the line
            if let Some(pos) = line.find(':') {
                let (key, value) = line.split_at(pos);
                let key = key.trim().to_lowercase();
                let value = value[1..].trim();
                
                match key.as_str() {
                    "user-agent" => {
                        current_agent = value.to_string();
                    },
                    "allow" => {
                        if current_agent == "*" || current_agent == user_agent {
                            allow_rules.push(value.to_string());
                        }
                    },
                    "disallow" => {
                        if current_agent == "*" || current_agent == user_agent {
                            disallow_rules.push(value.to_string());
                        }
                    },
                    _ => {}
                }
            }
        }
        
        Self {
            allow_rules,
            disallow_rules,
        }
    }
    
    /// Create a robots.txt that allows all paths
    fn allow_all() -> Self {
        Self {
            allow_rules: vec!["/".to_string()],
            disallow_rules: Vec::new(),
        }
    }
    
    /// Check if a path is allowed
    fn is_allowed(&self, path: &str) -> bool {
        // Check if the path matches any disallow rule
        for rule in &self.disallow_rules {
            if path.starts_with(rule) {
                // Check if there's a more specific allow rule
                for allow_rule in &self.allow_rules {
                    if path.starts_with(allow_rule) && allow_rule.len() > rule.len() {
                        return true;
                    }
                }
                return false;
            }
        }
        
        // If no disallow rule matches, the path is allowed
        true
    }
} 