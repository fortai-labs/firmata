# Legal Website Scraper Service

A Rust-based microservice for crawling and scraping legal websites with configurable rules, rate limiting, and politeness controls.

## Features

- **Configurable Crawling**: Define include/exclude patterns, max depth, and max pages
- **Rate Limiting**: Control request frequency and concurrency
- **Politeness Controls**: Respect robots.txt, configurable delays between requests
- **Content Processing**: Store HTML and convert to Markdown
- **Webhook Notifications**: Notify external services about job status
- **Scheduled Jobs**: Run scraper jobs on a schedule
- **API**: RESTful API for managing scraper configurations and jobs

## Architecture

The service is organized into the following layers:

- **Domain**: Core business entities and logic
- **Application**: Use cases and business rules
- **Infrastructure**: External services and technical concerns
- **API**: HTTP endpoints and handlers

## Getting Started

### Prerequisites

- Rust 1.70+
- PostgreSQL 14+
- Redis 6+
- MinIO (or S3-compatible storage)
- gRPC Markdown conversion service

### Setup

1. Clone the repository
2. Copy `.env.example` to `.env` and update the values
3. Run database migrations: `cargo run --bin migrate`
4. Start the service: `cargo run`

### Configuration

The service can be configured using:

1. Configuration files in the `config/` directory
2. Environment variables with the `APP_` prefix
3. Command-line arguments

See `config/default.toml` for available configuration options.

## Development

### Building

```bash
cargo build
```

### Running

```bash
cargo run
```

### Testing

```bash
cargo test
```

### Database Migrations

```bash
cargo run --bin migrate
```

## API Endpoints

### Scraper Configurations

- `GET /api/configs` - List all scraper configurations
- `POST /api/configs` - Create a new scraper configuration
- `GET /api/configs/{id}` - Get a specific scraper configuration
- `PUT /api/configs/{id}` - Update a scraper configuration
- `DELETE /api/configs/{id}` - Delete a scraper configuration
- `POST /api/configs/{id}/jobs` - Start a new job for a configuration

### Jobs

- `GET /api/jobs` - List all jobs
- `GET /api/jobs/{id}` - Get a specific job
- `DELETE /api/jobs/{id}` - Cancel a job

### Pages

- `GET /api/jobs/{job_id}/pages` - List all pages for a job
- `GET /api/pages/{id}` - Get a specific page
- `GET /api/pages/{id}/content` - Get the content of a page

### Webhooks

- `GET /api/webhooks` - List all webhooks
- `POST /api/webhooks` - Create a new webhook
- `GET /api/webhooks/{id}` - Get a specific webhook
- `PUT /api/webhooks/{id}` - Update a webhook
- `DELETE /api/webhooks/{id}` - Delete a webhook

## License

MIT 