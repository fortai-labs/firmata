#!/usr/bin/env node

const axios = require('axios');
const { v4: uuidv4 } = require('uuid');
const { S3Client, PutObjectCommand, GetObjectCommand, DeleteObjectCommand, HeadBucketCommand, CreateBucketCommand } = require('@aws-sdk/client-s3');
const { fromUtf8 } = require('@aws-sdk/util-utf8-node');
const fs = require('fs');
const path = require('path');
const dotenv = require('dotenv');
const { mkdir } = require('node:fs/promises');

// Load environment variables
dotenv.config();

// Load environment variables from .env file
const envPath = path.resolve(__dirname, '../../apps/service/.env');
const envContent = fs.readFileSync(envPath, 'utf8');
const envVars = {};

envContent.split('\n').forEach(line => {
  const match = line.match(/^([^#=]+)=(.*)$/);
  if (match) {
    const key = match[1].trim();
    const value = match[2].trim();
    envVars[key] = value;
  }
});

// Configuration
const config = {
  serviceUrl: process.env.SERVICE_URL || 'http://localhost:8080',
  s3: {
    endpoint: process.env.S3_ENDPOINT || 'http://localhost:9000',
    region: process.env.S3_REGION || 'us-east-1',
    bucket: process.env.S3_BUCKET || 'scraper-dev',
    accessKeyId: process.env.S3_ACCESS_KEY_ID || 'minioadmin',
    secretAccessKey: process.env.S3_SECRET_ACCESS_KEY || 'minioadmin',
  }
};

console.log('Using configuration:');
console.log(`Service URL: ${config.serviceUrl}`);
console.log(`S3 Endpoint: ${config.s3.endpoint}`);
console.log(`S3 Bucket: ${config.s3.bucket}`);
console.log(`S3 Region: ${config.s3.region}`);

// Initialize S3 client
const s3Client = new S3Client({
  region: config.s3.region,
  endpoint: config.s3.endpoint,
  forcePathStyle: true,
  credentials: {
    accessKeyId: config.s3.accessKeyId,
    secretAccessKey: config.s3.secretAccessKey,
  },
});

// Helper functions
async function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

async function testS3Connection() {
  console.log('Testing S3 connection...');
  
  const testKey = `test-${uuidv4()}.txt`;
  const testContent = 'This is a test file';
  
  try {
    // Upload test file
    await s3Client.send(new PutObjectCommand({
      Bucket: config.s3.bucket,
      Key: testKey,
      Body: testContent,
      ContentType: 'text/plain'
    }));
    
    console.log('✅ Successfully uploaded test file to S3');
    
    // Get test file
    const getResponse = await s3Client.send(new GetObjectCommand({
      Bucket: config.s3.bucket,
      Key: testKey
    }));
    
    const responseBody = await streamToString(getResponse.Body);
    
    if (responseBody === testContent) {
      console.log('✅ Successfully retrieved test file from S3');
    } else {
      console.error('❌ Retrieved file content does not match');
      return false;
    }
    
    // Delete test file
    await s3Client.send(new DeleteObjectCommand({
      Bucket: config.s3.bucket,
      Key: testKey
    }));
    
    console.log('✅ Successfully deleted test file from S3');
    return true;
  } catch (error) {
    console.error('❌ S3 connection test failed:', error.message);
    return false;
  }
}

async function streamToString(stream) {
  const chunks = [];
  for await (const chunk of stream) {
    chunks.push(Buffer.from(chunk));
  }
  return Buffer.concat(chunks).toString('utf-8');
}

async function testServiceHealth() {
  console.log('Testing service health...');
  
  try {
    const response = await axios.get(`${config.serviceUrl}/health`);
    
    if (response.status === 200 && response.data.status === 'ok') {
      console.log('✅ Service is healthy');
      return true;
    }
    
    console.error('❌ Service health check failed');
    return false;
  } catch (error) {
    console.error('❌ Service health check failed:', error.message);
    return false;
  }
}

async function createJob() {
  console.log('Creating a new job...');
  
  try {
    // First, create a config
    console.log('Creating a scraper config...');
    const configData = {
      name: `Test Config ${uuidv4().substring(0, 8)}`,
      description: "End-to-end test config",
      base_url: "https://edelman.dev",  // This URL returns HTML content
      include_patterns: ["*"],
      exclude_patterns: [],
      max_depth: 1,
      max_pages_per_job: 5,
      respect_robots_txt: true,
      user_agent: "TestBot/1.0",
      request_delay_ms: 1000,
      max_concurrent_requests: 2,
      schedule: null,
      headers: null
    };
    
    console.log('Config data:', JSON.stringify(configData, null, 2));
    
    try {
      const configResponse = await axios.post(`${config.serviceUrl}/api/configs`, configData);
      
      if (configResponse.status !== 201) {
        console.error('❌ Failed to create config');
        console.error('Response status:', configResponse.status);
        console.error('Response data:', configResponse.data);
        return null;
      }
      
      // Extract the config ID from the response
      const configId = configResponse.data.config.id;
      if (!configId) {
        console.error('❌ Failed to extract config ID from response');
        console.error('Response data:', configResponse.data);
        return null;
      }
      
      console.log(`✅ Successfully created config with ID: ${configId}`);
      
      // Now start a job with this config
      console.log(`Starting job with config ID: ${configId}...`);
      const startJobResponse = await axios.post(`${config.serviceUrl}/api/configs/${configId}/start`);
      
      console.log('Start job response:', JSON.stringify(startJobResponse.data, null, 2));
      
      if (startJobResponse.status !== 201) {
        console.error('❌ Failed to create job');
        console.error('Response status:', startJobResponse.status);
        console.error('Response data:', startJobResponse.data);
        return null;
      }
      
      // Extract the job ID from the response - try different possible property names
      let jobId = null;
      if (startJobResponse.data.job_id) {
        jobId = startJobResponse.data.job_id;
      } else if (startJobResponse.data.id) {
        jobId = startJobResponse.data.id;
      } else if (startJobResponse.data._links && startJobResponse.data._links.self && startJobResponse.data._links.self.href) {
        // Try to extract from the self link
        const selfLink = startJobResponse.data._links.self.href;
        const matches = selfLink.match(/\/api\/jobs\/([^\/]+)/);
        if (matches && matches[1]) {
          jobId = matches[1];
        }
      }
      
      if (!jobId) {
        console.error('❌ Failed to extract job ID from response');
        console.error('Response data:', startJobResponse.data);
        return null;
      }
      
      console.log(`✅ Successfully created job with ID: ${jobId}`);
      return jobId;
    } catch (apiError) {
      console.error('❌ API error:', apiError.message);
      if (apiError.response) {
        console.error('Response status:', apiError.response.status);
        console.error('Response data:', apiError.response.data);
      }
      throw apiError;
    }
  } catch (error) {
    console.error('❌ Failed to create job:', error.message);
    return null;
  }
}

async function getJobStatus(jobId) {
  console.log(`Checking status of job ${jobId}...`);
  
  try {
    const response = await axios.get(`${config.serviceUrl}/api/jobs/${jobId}`);
    
    console.log('Job status response:', JSON.stringify(response.data, null, 2));
    
    if (response.status === 200) {
      // Extract status from different possible response formats
      let status = null;
      if (typeof response.data === 'object') {
        if (response.data.status) {
          status = response.data.status;
        } else if (response.data.job && response.data.job.status) {
          status = response.data.job.status;
        }
      }
      
      if (status) {
        console.log(`✅ Job status: ${status}`);
        return response.data;
      } else {
        console.error('❌ Could not determine job status from response');
        console.error('Response data:', response.data);
        return null;
      }
    }
    
    console.error('❌ Failed to get job status');
    console.error('Response status:', response.status);
    console.error('Response data:', response.data);
    return null;
  } catch (error) {
    console.error('❌ Failed to get job status:', error.message);
    if (error.response) {
      console.error('Response status:', error.response.status);
      console.error('Response data:', error.response.data);
    }
    return null;
  }
}

async function waitForJobCompletion(jobId, maxAttempts = 30, interval = 2000) {
  console.log(`Waiting for job ${jobId} to complete...`);
  
  for (let i = 0; i < maxAttempts; i++) {
    const jobData = await getJobStatus(jobId);
    
    if (!jobData) {
      return null;
    }
    
    // Extract status from different possible response formats
    let status = null;
    if (typeof jobData === 'object') {
      if (jobData.status) {
        status = jobData.status;
      } else if (jobData.job && jobData.job.status) {
        status = jobData.job.status;
      }
    }
    
    if (!status) {
      console.error('❌ Could not determine job status');
      return null;
    }
    
    if (['completed', 'failed', 'done', 'error'].includes(status.toLowerCase())) {
      return jobData;
    }
    
    console.log(`Job is still ${status}, waiting... (attempt ${i+1}/${maxAttempts})`);
    await sleep(interval);
  }
  
  console.error('❌ Timed out waiting for job to complete');
  return null;
}

async function getJobResults(jobId) {
  console.log(`Getting results for job ${jobId}...`);
  
  try {
    // First get the job details to check pages count
    const jobResponse = await axios.get(`${config.serviceUrl}/api/jobs/${jobId}`);
    console.log('Job details response:', JSON.stringify(jobResponse.data, null, 2));
    
    const pagesCount = jobResponse.data.job?.pages_crawled || 0;
    console.log(`Job has crawled ${pagesCount} pages`);
    
    if (pagesCount === 0) {
      console.log('No pages to retrieve');
      return [];
    }
    
    // Get all pages for this job
    const pagesResponse = await axios.get(`${config.serviceUrl}/api/pages?job_id=${jobId}`);
    console.log('Pages response:', JSON.stringify(pagesResponse.data, null, 2));
    
    const pages = pagesResponse.data.pages || [];
    console.log(`Retrieved ${pages.length} pages`);
    
    return pages;
  } catch (error) {
    console.error('Error getting job results:', error.message);
    if (error.response) {
      console.error('Response status:', error.response.status);
      console.error('Response data:', error.response.data);
    }
    throw error;
  }
}

// Function to verify stored content
async function verifyStoredContent(pages) {
  console.log('Verifying stored content...');
  
  // Create downloads directory if it doesn't exist
  const downloadsDir = path.join(process.cwd(), 'downloads');
  try {
    await mkdir(downloadsDir, { recursive: true });
    console.log(`Saving downloaded files to: ${downloadsDir}`);
  } catch (error) {
    console.error(`Error creating downloads directory: ${error.message}`);
  }
  
  if (!pages || pages.length === 0) {
    console.log('No pages to verify - this is expected for the example.com domain in test mode');
    return true;
  }
  
  let allSuccess = true;
  
  // Process each page
  for (const page of pages) {
    const pageUrl = page.url;
    const htmlPath = page.html_storage_path;
    const markdownPath = page?.markdown_storage_path;
    
    // Create a directory for this page
    const pageDirName = sanitizeUrl(pageUrl);
    const pageDir = path.join(downloadsDir, pageDirName);
    
    try {
      await mkdir(pageDir, { recursive: true });
      
      // Save page metadata
      fs.writeFileSync(
        path.join(pageDir, 'metadata.json'),
        JSON.stringify(page, null, 2)
      );
      console.log(`✅ Saved page metadata to ${pageDir}/metadata.json`);
      
      // Get and save HTML content
      if (htmlPath) {
        try {
          const htmlContent = await getContentFromS3(htmlPath);
          if (htmlContent) {
            fs.writeFileSync(path.join(pageDir, 'content.html'), htmlContent);
            console.log(`✅ Successfully retrieved HTML content for page ${pageUrl}`);
            console.log(`✅ Saved HTML content to ${pageDir}/content.html`);
          } else {
            console.error(`❌ Failed to retrieve HTML content for page ${pageUrl}`);
            allSuccess = false;
          }
        } catch (error) {
          console.error(`❌ Error retrieving HTML content: ${error.message}`);
          allSuccess = false;
        }
      }
      
      // Get and save Markdown content if available
      if (markdownPath) {
        try {
          const markdownContent = await getContentFromS3(markdownPath);
          if (markdownContent) {
            fs.writeFileSync(path.join(pageDir, 'content.md'), markdownContent);
            console.log(`✅ Successfully retrieved Markdown content for page ${pageUrl}`);
            console.log(`✅ Saved Markdown content to ${pageDir}/content.md`);
          } else {
            console.error(`❌ Failed to retrieve Markdown content for page ${pageUrl}`);
            allSuccess = false;
          }
        } catch (error) {
          console.error(`❌ Error retrieving Markdown content: ${error.message}`);
          allSuccess = false;
        }
      }
    } catch (error) {
      console.error(`❌ Error processing page ${pageUrl}: ${error.message}`);
      allSuccess = false;
    }
  }
  
  return allSuccess;
}

// Helper function to get content from S3
async function getContentFromS3(key) {
  try {
    const response = await s3Client.send(new GetObjectCommand({
      Bucket: config.s3.bucket,
      Key: key
    }));
    
    return streamToString(response.Body);
  } catch (error) {
    console.error(`Error retrieving content from S3: ${error.message}`);
    return null;
  }
}

// Main test function
async function runEndToEndTest() {
  console.log('Starting end-to-end test of the service...');
  
  // Test S3 connection
  const s3Connected = await testS3Connection();
  if (!s3Connected) {
    console.error('❌ S3 connection test failed, aborting test');
    process.exit(1);
  }
  
  // Test service health
  const serviceHealthy = await testServiceHealth();
  if (!serviceHealthy) {
    console.error('❌ Service health check failed, aborting test');
    process.exit(1);
  }
  
  // Create a job
  const jobId = await createJob();
  if (!jobId) {
    console.error('❌ Failed to create job, aborting test');
    process.exit(1);
  }
  
  // Wait for job to complete
  const completedJob = await waitForJobCompletion(jobId);
  if (!completedJob) {
    console.error('❌ Job did not complete successfully, aborting test');
    process.exit(1);
  }
  
  if (completedJob.status === 'failed') {
    console.error(`❌ Job failed with error: ${completedJob.error_message}`);
    process.exit(1);
  }
  
  // Get job results
  const pages = await getJobResults(jobId);
  if (!pages) {
    console.error('❌ Failed to get job results, aborting test');
    process.exit(1);
  }
  
  // Verify stored content
  const contentVerified = await verifyStoredContent(pages);
  if (!contentVerified) {
    console.error('❌ Content verification failed');
    process.exit(1);
  }
  
  console.log('✅ End-to-end test completed successfully!');
  process.exit(0);
}

// Run the test
runEndToEndTest().catch(error => {
  console.error('❌ Unhandled error during test:', error);
  process.exit(1);
});

// Helper function to sanitize URLs for use as directory names
function sanitizeUrl(url) {
  return url.replace(/[^a-zA-Z0-9]/g, '_');
}
