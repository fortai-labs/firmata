use std::time::{Duration, Instant};
use mockito::{mock, server_url};
use tokio::time::sleep;
use url::Url;

use crate::application::scraper::crawler::{Crawler, CrawlerConfig};

#[tokio::test]
async fn test_crawler_config_default() {
    let config = CrawlerConfig::default();
    
    assert_eq!(config.max_concurrent_requests, 10);
    assert_eq!(config.delay_between_requests_ms, 1000);
    assert_eq!(config.max_retries, 3);
    assert_eq!(config.user_agent, "FortaiLegalScraper/1.0");
    assert_eq!(config.request_timeout_secs, 30);
    assert_eq!(config.respect_robots_txt, true);
    assert_eq!(config.max_page_size_bytes, 10 * 1024 * 1024);
}

#[tokio::test]
async fn test_crawler_new() {
    let config = CrawlerConfig::default();
    let crawler = Crawler::new(config).unwrap();
    
    // Just testing that we can create a crawler without errors
    assert!(true);
}

#[tokio::test]
async fn test_should_crawl_url_with_include_patterns() {
    let config = CrawlerConfig::default();
    let crawler = Crawler::new(config).unwrap();
    
    let include_patterns = vec![
        r"https://example\.com/legal/.*".to_string(),
        r"https://example\.com/terms/.*".to_string(),
    ];
    let exclude_patterns = vec![];
    
    // URLs that should match
    assert!(crawler.should_crawl_url("https://example.com/legal/privacy", &include_patterns, &exclude_patterns));
    assert!(crawler.should_crawl_url("https://example.com/terms/service", &include_patterns, &exclude_patterns));
    
    // URLs that should not match
    assert!(!crawler.should_crawl_url("https://example.com/about", &include_patterns, &exclude_patterns));
    assert!(!crawler.should_crawl_url("https://other.com/legal/privacy", &include_patterns, &exclude_patterns));
}

#[tokio::test]
async fn test_should_crawl_url_with_exclude_patterns() {
    let config = CrawlerConfig::default();
    let crawler = Crawler::new(config).unwrap();
    
    let include_patterns = vec![];
    let exclude_patterns = vec![
        r".*\.pdf$".to_string(),
        r".*login.*".to_string(),
    ];
    
    // URLs that should be excluded
    assert!(!crawler.should_crawl_url("https://example.com/document.pdf", &include_patterns, &exclude_patterns));
    assert!(!crawler.should_crawl_url("https://example.com/login", &include_patterns, &exclude_patterns));
    assert!(!crawler.should_crawl_url("https://example.com/user/login/form", &include_patterns, &exclude_patterns));
    
    // URLs that should be included
    assert!(crawler.should_crawl_url("https://example.com/about", &include_patterns, &exclude_patterns));
    assert!(crawler.should_crawl_url("https://example.com/legal/privacy", &include_patterns, &exclude_patterns));
}

#[tokio::test]
async fn test_should_crawl_url_with_both_patterns() {
    let config = CrawlerConfig::default();
    let crawler = Crawler::new(config).unwrap();
    
    let include_patterns = vec![
        r"https://example\.com/.*".to_string(),
    ];
    let exclude_patterns = vec![
        r".*\.pdf$".to_string(),
        r".*login.*".to_string(),
    ];
    
    // URLs that should match include but not exclude
    assert!(crawler.should_crawl_url("https://example.com/about", &include_patterns, &exclude_patterns));
    assert!(crawler.should_crawl_url("https://example.com/legal/privacy", &include_patterns, &exclude_patterns));
    
    // URLs that match include but also exclude
    assert!(!crawler.should_crawl_url("https://example.com/document.pdf", &include_patterns, &exclude_patterns));
    assert!(!crawler.should_crawl_url("https://example.com/login", &include_patterns, &exclude_patterns));
    
    // URLs that don't match include
    assert!(!crawler.should_crawl_url("https://other.com/about", &include_patterns, &exclude_patterns));
}

#[tokio::test]
async fn test_normalize_url() {
    let config = CrawlerConfig::default();
    let crawler = Crawler::new(config).unwrap();
    
    // Test removing fragments
    assert_eq!(
        crawler.normalize_url("https://example.com/page#section").unwrap(),
        "https://example.com/page"
    );
    
    // Test adding trailing slash to domain
    assert_eq!(
        crawler.normalize_url("https://example.com").unwrap(),
        "https://example.com/"
    );
    
    // Test preserving path
    assert_eq!(
        crawler.normalize_url("https://example.com/path/to/page").unwrap(),
        "https://example.com/path/to/page"
    );
    
    // Test error on invalid URL
    assert!(crawler.normalize_url("not a url").is_err());
}

#[tokio::test]
async fn test_extract_domain() {
    let config = CrawlerConfig::default();
    let crawler = Crawler::new(config).unwrap();
    
    assert_eq!(
        crawler.extract_domain("https://example.com/path").unwrap(),
        "example.com"
    );
    
    assert_eq!(
        crawler.extract_domain("http://sub.example.com/path").unwrap(),
        "sub.example.com"
    );
    
    assert!(crawler.extract_domain("not a url").is_err());
}

#[tokio::test]
async fn test_extract_path() {
    let config = CrawlerConfig::default();
    let crawler = Crawler::new(config).unwrap();
    
    assert_eq!(
        crawler.extract_path("https://example.com/path/to/page").unwrap(),
        "/path/to/page"
    );
    
    assert_eq!(
        crawler.extract_path("https://example.com").unwrap(),
        "/"
    );
    
    assert!(crawler.extract_path("not a url").is_err());
}

#[tokio::test]
async fn test_rate_limiting() {
    let mut config = CrawlerConfig::default();
    config.delay_between_requests_ms = 500; // Set a 500ms delay
    
    let crawler = Crawler::new(config).unwrap();
    
    // Apply rate limiting for the first time
    let domain = "example.com";
    let start = Instant::now();
    crawler.apply_rate_limiting(domain).await;
    
    // This should return immediately
    let first_duration = start.elapsed();
    assert!(first_duration < Duration::from_millis(50));
    
    // Apply rate limiting again for the same domain
    let start = Instant::now();
    crawler.apply_rate_limiting(domain).await;
    
    // This should wait for the delay
    let second_duration = start.elapsed();
    assert!(second_duration >= Duration::from_millis(450)); // Allow some margin
    assert!(second_duration < Duration::from_millis(600)); // But not too much
    
    // Apply rate limiting for a different domain
    let start = Instant::now();
    crawler.apply_rate_limiting("other.com").await;
    
    // This should return immediately
    let third_duration = start.elapsed();
    assert!(third_duration < Duration::from_millis(50));
}

#[tokio::test]
async fn test_extract_title() {
    let config = CrawlerConfig::default();
    let crawler = Crawler::new(config).unwrap();
    
    // Test with a simple title
    let html = r#"<html><head><title>Test Page</title></head><body></body></html>"#;
    assert_eq!(crawler.extract_title(html), Some("Test Page".to_string()));
    
    // Test with a title containing HTML entities
    let html = r#"<html><head><title>Test &amp; Page</title></head><body></body></html>"#;
    assert_eq!(crawler.extract_title(html), Some("Test &amp; Page".to_string()));
    
    // Test with no title
    let html = r#"<html><head></head><body></body></html>"#;
    assert_eq!(crawler.extract_title(html), None);
}

#[tokio::test]
async fn test_extract_links() {
    let config = CrawlerConfig::default();
    let crawler = Crawler::new(config).unwrap();
    
    let html = r#"
    <html>
    <body>
        <a href="https://example.com/page1">Page 1</a>
        <a href="/page2">Page 2</a>
        <a href="page3">Page 3</a>
        <a href="#section">Section</a>
        <a href="javascript:void(0)">JS Link</a>
        <a href="mailto:test@example.com">Email</a>
        <a href="https://other.com/page">Other Site</a>
    </body>
    </html>
    "#;
    
    let base_url = "https://example.com/base";
    let links = crawler.extract_links(html, base_url);
    
    // Should extract absolute URLs and resolve relative URLs
    assert!(links.contains(&"https://example.com/page1".to_string()));
    assert!(links.contains(&"https://example.com/page2".to_string()));
    assert!(links.contains(&"https://example.com/base/page3".to_string()));
    assert!(links.contains(&"https://other.com/page".to_string()));
    
    // Should not extract fragment, javascript, or mailto links
    assert_eq!(links.len(), 4);
    assert!(!links.contains(&"https://example.com/base#section".to_string()));
    assert!(!links.contains(&"javascript:void(0)".to_string()));
    assert!(!links.contains(&"mailto:test@example.com".to_string()));
}

#[tokio::test]
async fn test_robots_txt_parsing() {
    // This test is more complex and would require mocking HTTP responses
    // We'll use mockito to create a mock server
    
    let _m = mock("GET", "/robots.txt")
        .with_status(200)
        .with_header("content-type", "text/plain")
        .with_body(r#"
        User-agent: *
        Disallow: /private/
        Disallow: /admin/
        Allow: /private/public/
        
        User-agent: FortaiLegalScraper/1.0
        Disallow: /legal/private/
        "#)
        .create();
    
    let mut config = CrawlerConfig::default();
    config.respect_robots_txt = true;
    
    let crawler = Crawler::new(config).unwrap();
    
    // Parse the domain from the mock server URL
    let url = Url::parse(&server_url()).unwrap();
    let domain = url.host_str().unwrap();
    
    // Test paths that should be allowed
    assert!(crawler.is_allowed_by_robots_txt(domain, "/public").await.unwrap());
    assert!(crawler.is_allowed_by_robots_txt(domain, "/private/public").await.unwrap());
    
    // Test paths that should be disallowed for all agents
    assert!(!crawler.is_allowed_by_robots_txt(domain, "/private").await.unwrap());
    assert!(!crawler.is_allowed_by_robots_txt(domain, "/admin").await.unwrap());
    
    // Test paths that should be disallowed specifically for our user agent
    assert!(!crawler.is_allowed_by_robots_txt(domain, "/legal/private").await.unwrap());
}

#[tokio::test]
async fn test_make_request_with_retries() {
    // Set up a mock server that fails twice then succeeds
    let _m1 = mock("GET", "/retry-test")
        .with_status(500)
        .create();
    
    let _m2 = mock("GET", "/retry-test")
        .with_status(500)
        .create();
    
    let _m3 = mock("GET", "/retry-test")
        .with_status(200)
        .with_header("content-type", "text/html")
        .with_body("<html><body>Success</body></html>")
        .create();
    
    let mut config = CrawlerConfig::default();
    config.max_retries = 3;
    config.request_timeout_secs = 1;
    
    let crawler = Crawler::new(config).unwrap();
    
    // This should succeed after 2 retries
    let url = format!("{}/retry-test", server_url());
    let response = crawler.make_request_with_retries(&url).await.unwrap();
    
    assert_eq!(response.status().as_u16(), 200);
    assert_eq!(response.text().await.unwrap(), "<html><body>Success</body></html>");
}

#[tokio::test]
async fn test_crawl_url_integration() {
    // Set up a mock server with HTML content and robots.txt
    let _m1 = mock("GET", "/robots.txt")
        .with_status(200)
        .with_header("content-type", "text/plain")
        .with_body("User-agent: *\nAllow: /\n")
        .create();
    
    let _m2 = mock("GET", "/test-page")
        .with_status(200)
        .with_header("content-type", "text/html")
        .with_body(r#"
        <html>
        <head>
            <title>Test Page</title>
        </head>
        <body>
            <a href="/link1">Link 1</a>
            <a href="/link2">Link 2</a>
            <a href="https://external.com/page">External Link</a>
        </body>
        </html>
        "#)
        .create();
    
    let mut config = CrawlerConfig::default();
    config.respect_robots_txt = true;
    
    let crawler = Crawler::new(config).unwrap();
    
    let url = format!("{}/test-page", server_url());
    let include_patterns = vec![];
    let exclude_patterns = vec![];
    
    let (page, discovered_urls) = crawler.crawl_url(
        &url,
        0,
        None,
        &include_patterns,
        &exclude_patterns
    ).await.unwrap();
    
    // Check page properties
    assert_eq!(page.url, url);
    assert_eq!(page.http_status, 200);
    assert_eq!(page.title, Some("Test Page".to_string()));
    assert!(page.error_message.is_none());
    
    // Check discovered URLs
    assert_eq!(discovered_urls.len(), 3);
    assert!(discovered_urls.contains(&format!("{}/link1", server_url())));
    assert!(discovered_urls.contains(&format!("{}/link2", server_url())));
    assert!(discovered_urls.contains(&"https://external.com/page".to_string()));
}

#[tokio::test]
async fn test_crawl_url_with_patterns() {
    // Set up a mock server
    let _m = mock("GET", "/filtered-page")
        .with_status(200)
        .with_header("content-type", "text/html")
        .with_body("<html><body>Content</body></html>")
        .create();
    
    let config = CrawlerConfig::default();
    let crawler = Crawler::new(config).unwrap();
    
    let url = format!("{}/filtered-page", server_url());
    
    // Test with include pattern that matches
    let include_patterns = vec![format!("{}.*", server_url())];
    let exclude_patterns = vec![];
    
    let result = crawler.crawl_url(
        &url,
        0,
        None,
        &include_patterns,
        &exclude_patterns
    ).await;
    
    assert!(result.is_ok());
    
    // Test with include pattern that doesn't match
    let include_patterns = vec!["https://other-site.com/.*".to_string()];
    let exclude_patterns = vec![];
    
    let result = crawler.crawl_url(
        &url,
        0,
        None,
        &include_patterns,
        &exclude_patterns
    ).await;
    
    assert!(result.is_err());
    
    // Test with exclude pattern that matches
    let include_patterns = vec![];
    let exclude_patterns = vec![".*/filtered-page".to_string()];
    
    let result = crawler.crawl_url(
        &url,
        0,
        None,
        &include_patterns,
        &exclude_patterns
    ).await;
    
    assert!(result.is_err());
}

#[tokio::test]
async fn test_concurrent_requests() {
    // Create a mock endpoint that takes some time to respond
    let _m = mock("GET", "/slow")
        .with_status(200)
        .with_header("content-type", "text/html")
        .with_body("<html><body>Slow response</body></html>")
        .delay(Duration::from_millis(200))
        .create();
    
    let mut config = CrawlerConfig::default();
    config.max_concurrent_requests = 3;
    
    let crawler = Crawler::new(config).unwrap();
    
    let url = format!("{}/slow", server_url());
    
    // Start 5 concurrent requests
    let start = Instant::now();
    
    let mut handles = vec![];
    
    for _ in 0..5 {
        let crawler_clone = crawler.clone();
        let url_clone = url.clone();
        
        let handle = tokio::spawn(async move {
            crawler_clone.crawl_url(
                &url_clone,
                0,
                None,
                &vec![],
                &vec![]
            ).await.unwrap();
        });
        
        handles.push(handle);
    }
    
    // Wait for all requests to complete
    for handle in handles {
        handle.await.unwrap();
    }
    
    let duration = start.elapsed();
    
    // With 3 concurrent requests and 5 total requests that each take 200ms,
    // we expect this to take at least 400ms (two batches)
    assert!(duration >= Duration::from_millis(390));
}

#[tokio::test]
async fn test_content_size_limit() {
    // Create a mock endpoint with a large response
    let large_content = "a".repeat(1024 * 1024); // 1MB
    
    let _m = mock("GET", "/large")
        .with_status(200)
        .with_header("content-type", "text/plain")
        .with_header("content-length", &large_content.len().to_string())
        .with_body(&large_content)
        .create();
    
    // Test with a size limit larger than the content
    let mut config = CrawlerConfig::default();
    config.max_page_size_bytes = 2 * 1024 * 1024; // 2MB
    
    let crawler = Crawler::new(config).unwrap();
    
    let url = format!("{}/large", server_url());
    
    let result = crawler.crawl_url(
        &url,
        0,
        None,
        &vec![],
        &vec![]
    ).await;
    
    assert!(result.is_ok());
    
    // Test with a size limit smaller than the content
    let mut config = CrawlerConfig::default();
    config.max_page_size_bytes = 512 * 1024; // 512KB
    
    let crawler = Crawler::new(config).unwrap();
    
    let result = crawler.crawl_url(
        &url,
        0,
        None,
        &vec![],
        &vec![]
    ).await;
    
    assert!(result.is_err());
} 