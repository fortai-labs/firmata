use anyhow::Result;
use async_trait::async_trait;
use aws_config::Region;
use aws_sdk_s3::{Client,  config::Credentials};
use std::io::Cursor;
use uuid::Uuid;

use crate::config::settings::Storage as StorageConfig;
use crate::utils::error::AppError;

#[async_trait]
pub trait StorageClient {
    async fn upload_html(&self, job_id: &Uuid, url: &str, content: &str) -> Result<String>;
    async fn upload_markdown(&self, job_id: &Uuid, url: &str, content: &str) -> Result<String>;
    async fn get_object(&self, path: &str) -> Result<String>;
    async fn delete_object(&self, path: &str) -> Result<()>;
}

pub struct S3StorageClient {
    client: Client,
    bucket: String,
}

impl S3StorageClient {
    pub async fn new(config: &StorageConfig) -> Result<Self> {
        // Set up credentials
        let credentials = Credentials::new(
            &config.access_key,
            &config.secret_key,
            None,
            None,
            "s3-credentials",
        );

        // Set up region directly
        let region = Region::new(config.region.clone());

        // Build the S3 client configuration
        let s3_config = aws_sdk_s3::Config::builder()
            .region(region)
            .endpoint_url(&config.endpoint)
            .credentials_provider(credentials)
            .build();

        // Create the client
        let client = Client::from_conf(s3_config);

        // Verify bucket exists
        let buckets = client.list_buckets().send().await?;
        let bucket_exists = buckets.buckets().iter().any(|b| b.name() == Some(&config.bucket));

        if !bucket_exists {
            // Create the bucket if it doesn't exist
            client
                .create_bucket()
                .bucket(&config.bucket)
                .send()
                .await
                .map_err(|e| AppError::Storage(format!("Failed to create bucket: {}", e)))?;
        }

        Ok(Self {
            client,
            bucket: config.bucket.clone(),
        })
    }

    fn generate_path(&self, job_id: &Uuid, url: &str, extension: &str) -> String {
        // Normalize URL for path
        let normalized_url = url.replace("://", "/").replace(":", "_");
        
        // Generate a path with job ID and normalized URL
        format!("{}/{}.{}", job_id, normalized_url, extension)
    }
}

#[async_trait]
impl StorageClient for S3StorageClient {
    async fn upload_html(&self, job_id: &Uuid, url: &str, content: &str) -> Result<String> {
        let path = self.generate_path(job_id, url, "html");
        
        // Upload the content
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&path)
            .body(content.as_bytes().to_vec().into())
            .content_type("text/html")
            .metadata("source_url", url)
            .metadata("job_id", job_id.to_string())
            .send()
            .await
            .map_err(|e| AppError::Storage(format!("Failed to upload HTML: {}", e)))?;
        
        Ok(path)
    }

    async fn upload_markdown(&self, job_id: &Uuid, url: &str, content: &str) -> Result<String> {
        let path = self.generate_path(job_id, url, "md");
        
        // Upload the content
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&path)
            .body(content.as_bytes().to_vec().into())
            .content_type("text/markdown")
            .metadata("source_url", url)
            .metadata("job_id", job_id.to_string())
            .send()
            .await
            .map_err(|e| AppError::Storage(format!("Failed to upload Markdown: {}", e)))?;
        
        Ok(path)
    }

    async fn get_object(&self, path: &str) -> Result<String> {
        // Get the object
        let response = self.client
            .get_object()
            .bucket(&self.bucket)
            .key(path)
            .send()
            .await
            .map_err(|e| AppError::Storage(format!("Failed to get object: {}", e)))?;
        
        // Read the body
        let body = response.body.collect().await?;
        let bytes = body.into_bytes();
        
        // Convert to string
        let content = String::from_utf8(bytes.to_vec())
            .map_err(|e| AppError::Storage(format!("Failed to convert object to string: {}", e)))?;
        
        Ok(content)
    }

    async fn delete_object(&self, path: &str) -> Result<()> {
        // Delete the object
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(path)
            .send()
            .await
            .map_err(|e| AppError::Storage(format!("Failed to delete object: {}", e)))?;
        
        Ok(())
    }
} 