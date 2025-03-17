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

### Local Development Environment

We provide a Docker Compose setup for local development:

```bash
# Start the development environment
docker-compose up -d

# Initialize the development environment (create bucket, run migrations)
./scripts/init-dev.sh
```

This will start:
- PostgreSQL database
- Redis server
- MinIO (S3-compatible storage)

## CI/CD Pipeline

This project uses GitHub Actions for continuous integration and deployment:

1. **Lint**: Runs `rustfmt` and `clippy` to ensure code quality
2. **Test**: Runs the test suite with all dependencies (PostgreSQL, Redis, MinIO)
3. **Build**: Builds the release binaries
4. **Deploy**: Builds and pushes a Docker image, then deploys to the production environment

The pipeline is triggered on:
- Push to the `main` branch
- Pull requests to the `main` branch

### Deployment

The service is deployed as a Docker container. The deployment process:

1. Builds a Docker image with the release binaries
2. Pushes the image to DockerHub with tags:
   - `latest`
   - Git commit SHA
3. Deploys the image to the production environment

## API Endpoints

### Scraper Configurations

- `GET /api/configs` - List all scraper configurations
- `POST /api/configs` - Create a new scraper configuration
- `GET /api/configs/{id}` - Get a specific scraper configuration
- `PUT /api/configs/{id}` - Update a scraper configuration
- `DELETE /api/configs/{id}` - Delete a scraper configuration
- `POST /api/configs/{id}/start` - Start a new job for a configuration

### Jobs

- `GET /api/jobs` - List all jobs
- `GET /api/jobs/{id}` - Get a specific job
- `POST /api/jobs/{id}/cancel` - Cancel a job

### Pages

- `GET /api/pages` - List all pages (can filter by job_id query parameter)
- `GET /api/pages/{id}` - Get a specific page
- `GET /api/pages/{id}/html` - Get the content of a page
- `GET /api/pages/{id}/markdown` - Get the markdown content of a page

### Webhooks

- `GET /api/webhooks` - List all webhooks
- `POST /api/webhooks` - Create a new webhook
- `GET /api/webhooks/{id}` - Get a specific webhook
- `PUT /api/webhooks/{id}` - Update a webhook
- `DELETE /api/webhooks/{id}` - Delete a webhook

## License

MIT 