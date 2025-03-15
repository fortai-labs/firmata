use anyhow::Result;
use std::collections::HashMap;
use tonic::transport::Channel;

use crate::config::settings::MarkdownService as MarkdownServiceConfig;
use crate::utils::error::AppError;

// Include the generated code
pub mod markdown {
    tonic::include_proto!("markdown");
}

use markdown::{
    markdown_converter_client::MarkdownConverterClient,
    ConversionRequest,
};

pub struct MarkdownClient {
    client: MarkdownConverterClient<Channel>,
}

impl MarkdownClient {
    pub async fn new(config: &MarkdownServiceConfig) -> Result<Self> {
        // Connect to the server
        let channel = Channel::from_shared(config.url.clone())
            .map_err(|e| AppError::MarkdownService(format!("Invalid URL: {}", e)))?
            .connect()
            .await
            .map_err(|e| AppError::MarkdownService(format!("Failed to connect: {}", e)))?;
        
        // Create the client
        let client = MarkdownConverterClient::new(channel);
        
        Ok(Self { client })
    }
    
    pub async fn convert_html_to_markdown(
        &mut self,
        html_content: &str,
        url: &str,
        metadata: HashMap<String, String>,
    ) -> Result<(String, Vec<String>, HashMap<String, String>)> {
        // Create the request
        let request = tonic::Request::new(ConversionRequest {
            html_content: html_content.to_string(),
            url: url.to_string(),
            metadata,
        });
        
        // Send the request
        let response = self.client
            .convert_html_to_markdown(request)
            .await
            .map_err(|e| AppError::MarkdownService(format!("Conversion failed: {}", e)))?;
        
        let response = response.into_inner();
        
        Ok((
            response.markdown_content,
            response.extracted_links,
            response.metadata,
        ))
    }
} 