# Legal Website Scraper Service

A high-performance Rust-based microservice for crawling and extracting content from legal websites.

## Features

- **High Performance**: Built with Rust for optimal speed and resource efficiency
- **Configurable Crawling**: Supports custom crawling rules, depth limits, and URL patterns
- **Content Processing**: Converts HTML to Markdown for easier consumption
- **Scalable Architecture**: Uses Redis for job queuing and PostgreSQL for data persistence
- **S3 Storage**: Stores crawled content in S3-compatible storage
- **RESTful API**: Provides a comprehensive API for managing scraper configurations, jobs, and results
- **Webhook Support**: Notifies external systems about scraping events
- **Analytics**: Tracks performance metrics and crawling statistics

## Architecture

The service follows a modular architecture with clear separation of concerns:

### Domain Layer
- **Models**: Core domain entities like `Config`, `Job`, `Page`, and `Webhook`
- **Repositories**: Interfaces for data access

### Application Layer
- **Services**: Business logic for scraper configuration and job management
- **Workers**: Background processing for crawling tasks
- **Scheduler**: Handles recurring jobs

### Infrastructure Layer
- **Database**: PostgreSQL for persistent storage
- **Queue**: Redis for job queuing
- **Storage**: S3-compatible storage for HTML and Markdown content
- **gRPC**: Client for HTML to Markdown conversion

### API Layer
- **Routes**: RESTful API endpoints
- **Handlers**: Request processing and response formatting
- **Middleware**: Authentication, logging, and error handling

## API Endpoints

### Configurations
- `GET /api/configs` - List all scraper configurations
- `POST /api/configs` - Create a new scraper configuration
- `GET /api/configs/:id` - Get a specific configuration
- `PUT /api/configs/:id` - Update a configuration
- `POST /api/configs/:id/start` - Start a job with a configuration

### Jobs
- `GET /api/jobs` - List all jobs
- `GET /api/jobs/:id` - Get a specific job
- `POST /api/jobs/:id/cancel` - Cancel a running job

### Pages
- `GET /api/pages` - List all crawled pages
- `GET /api/pages/:id` - Get a specific page
- `GET /api/pages/:id/html` - Get the HTML content of a page
- `GET /api/pages/:id/markdown` - Get the Markdown content of a page

### Webhooks
- `GET /api/webhooks` - List all webhooks
- `POST /api/webhooks` - Create a new webhook
- `GET /api/webhooks/:id` - Get a specific webhook
- `PUT /api/webhooks/:id` - Update a webhook
- `DELETE /api/webhooks/:id` - Delete a webhook

### Analytics
- `GET /api/analytics/jobs` - Get job statistics
- `GET /api/analytics/configs` - Get configuration statistics
- `GET /api/analytics/jobs/:id/timeline` - Get timeline for a specific job

## Setup and Configuration

### Prerequisites
- Rust (latest stable)
- PostgreSQL
- Redis
- S3-compatible storage (AWS S3, MinIO, etc.)

### Environment Variables
```
# Server
SERVER_ADDRESS=0.0.0.0:8080

# Database
DATABASE_URL=postgres://user:password@localhost:5432/scraper

# Redis
REDIS_URL=redis://localhost:6379
REDIS_JOB_QUEUE=scraper_jobs

# Storage
STORAGE_ENDPOINT=https://s3.amazonaws.com
STORAGE_REGION=us-east-1
STORAGE_BUCKET=scraper-content
STORAGE_ACCESS_KEY=your-access-key
STORAGE_SECRET_KEY=your-secret-key

# Logging
LOG_LEVEL=info
```

### Database Migrations
```bash
cargo run --bin migrate
```

### Running the Service
```bash
cargo run
```

## Development

### Building
```bash
cargo build
```

### Testing
```bash
cargo test
```

### Code Structure
```
src/
├── api/                 # API layer
│   ├── handlers/        # Request handlers
│   └── routes/          # API routes
│
├── application/         # Application layer
│   ├── scraper/         # Scraper service and worker
│   └── scheduler/       # Scheduler service
│
├── config/              # Configuration
│
├── domain/              # Domain models
│   ├── config.rs        # Scraper configuration
│   ├── job.rs           # Scraper job
│   ├── page.rs          # Crawled page
│   └── webhook.rs       # Webhook
│
├── infrastructure/      # Infrastructure layer
│   ├── database/        # Database access
│   ├── grpc/            # gRPC clients
│   ├── queue/           # Redis queue
│   └── storage/         # S3 storage
│
├── utils/               # Utilities
│   ├── error.rs         # Error handling
│   └── logging.rs       # Logging
│
├── main.rs              # Application entry point
└── build.rs             # Build script for gRPC
```

## License

This project is licensed under the ISC License - see the LICENSE file for details. 