use std::time::Duration;
use mockito::Server;

#[tokio::test]
async fn test_mockito_server() {
    // Create a mock server
    let mut server = Server::new();
    
    // Set up a mock endpoint
    let mock = server.mock("GET", "/test")
        .with_status(200)
        .with_header("content-type", "text/plain")
        .with_body("test response")
        .create();
    
    // Get the server URL
    let url = server.url();
    
    // Verify the URL is valid
    assert!(url.starts_with("http://"));
    
    // We can't directly assert that the mock wasn't called,
    // but we can verify the server is working
    assert!(mock.matched());
} 