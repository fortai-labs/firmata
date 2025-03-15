use anyhow::Result;
use std::collections::HashMap;
use tonic::transport::Channel;

use crate::config::settings::Grpc;
use crate::utils::error::AppError;

// Include the generated code
pub mod markdown {
    tonic::include_proto!("markdown");
}

use markdown::{
    markdown_converter_client::MarkdownConverterClient,
    ConversionRequest,
};

#[derive(Clone)]
pub struct MarkdownClient {
    client: MarkdownConverterClient<Channel>,
}

impl MarkdownClient {
    pub async fn new(config: &Grpc) -> Result<Self> {
        // Connect to the server
        let channel = Channel::from_shared(config.markdown_service_url.clone())
            .map_err(|e| AppError::MarkdownService(format!("Invalid URL: {}", e)))?
            .connect()
            .await
            .map_err(|e| AppError::MarkdownService(format!("Failed to connect: {}", e)))?;
        
        // Create the client
        let client = MarkdownConverterClient::new(channel);
        
        Ok(Self { client })
    }
    
    pub async fn convert_html_to_markdown(
        &self,
        html_content: &str,
        url: &str,
        metadata: HashMap<String, String>,
    ) -> Result<(String, Vec<String>, HashMap<String, String>)> {
        // Create the request
        let request = ConversionRequest {
            html_content: html_content.to_string(),
            url: url.to_string(),
            metadata,
        };
        
        // Clone the client for this request
        let mut client = self.client.clone();
        
        // Send the request
        let response = client
            .convert_html_to_markdown(request)
            .await
            .map_err(|e| AppError::MarkdownService(format!("Failed to convert HTML to Markdown: {}", e)))?;
        
        let response = response.into_inner();
        
        Ok((
            response.markdown_content,
            response.extracted_links,
            response.metadata,
        ))
    }
} 