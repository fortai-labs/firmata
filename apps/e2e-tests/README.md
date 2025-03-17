# End-to-End Tests for Fortai Scraper Service

This directory contains end-to-end tests for the Fortai scraper service. The tests verify that the service is working correctly by:

1. Testing S3 connectivity
2. Checking service health
3. Creating and running a scraper job
4. Monitoring job status
5. Retrieving and verifying job results
6. Downloading HTML and Markdown content to the filesystem

## Prerequisites

- Node.js 16+
- pnpm
- Running Fortai scraper service
- S3-compatible storage service (e.g., MinIO)

## Configuration

The tests use environment variables for configuration. You can set these in a `.env` file in this directory:

```
SERVICE_URL=http://localhost:8080
S3_ENDPOINT=http://localhost:9000
S3_REGION=us-east-1
S3_BUCKET=scraper-dev
S3_ACCESS_KEY_ID=minioadmin
S3_SECRET_ACCESS_KEY=minioadmin
```

## Running the Tests

1. Install dependencies:
   ```
   pnpm install
   ```

2. Run the tests:
   ```
   pnpm test
   ```

## Test Flow

The test script performs the following steps:

1. **S3 Connection Tests**:
   - Upload a test file to S3
   - Retrieve the test file from S3
   - Delete the test file from S3

2. **Service Health Check**:
   - Verify the service is running and healthy

3. **Job Creation and Monitoring**:
   - Create a new scraper configuration
   - Start a job with the configuration
   - Monitor the job status until completion

4. **Result Verification**:
   - Retrieve job results
   - Verify the content was stored correctly in S3

5. **Content Download**:
   - Download HTML and Markdown content from S3
   - Save the content to the filesystem for inspection

## Downloaded Files

The test script downloads content from the crawled pages and saves it to the `downloads` directory. The files are organized as follows:

```
downloads/
  └── [sanitized_url]/
      ├── content.html   # HTML content of the page
      ├── content.md     # Markdown content of the page (if available)
      └── metadata.json  # Page metadata from the API
```

Where `[sanitized_url]` is the URL of the page with special characters replaced by underscores.

## Examining Downloaded Content

After running the tests, you can examine the downloaded content in the `downloads` directory. This is useful for:

- Verifying the HTML content was crawled correctly
- Checking the quality of the Markdown conversion
- Inspecting page metadata

## Troubleshooting

If the tests fail, check the following:

- Ensure the scraper service is running and accessible at the configured URL
- Verify the S3 service is running and the credentials are correct
- Check that the bucket exists and is accessible
- Look for error messages in the test output for specific issues
- Examine the downloaded files for any content issues

## Adding More Tests

To add more tests, modify the `test-service.js` file. The script is designed to be extensible, so you can add more test cases as needed.

For example, you could:
- Test different URL patterns
- Verify specific content extraction
- Test error handling scenarios
- Add performance tests

## Customizing the Test Configuration

You can modify the test configuration in the `createJob` function to test different scraper settings:

- Change the base URL to test different websites
- Adjust depth and page limits
- Modify include/exclude patterns
- Test different user agents or request delays 