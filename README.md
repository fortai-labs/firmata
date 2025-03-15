# Turborepo starter

This Turborepo includes a Rust-based legal website scraper service alongside Next.js applications.

## Using this example

Run the following command:

```sh
npx create-turbo@latest
```

## What's inside?

This Turborepo includes the following packages/apps:

### Apps and Packages

- `docs`: a [Next.js](https://nextjs.org/) app
- `web`: another [Next.js](https://nextjs.org/) app
- `service`: a [Rust](https://www.rust-lang.org/)-based legal website scraper microservice
- `@repo/ui`: a stub React component library shared by both `web` and `docs` applications
- `@repo/eslint-config`: `eslint` configurations (includes `eslint-config-next` and `eslint-config-prettier`)
- `@repo/typescript-config`: `tsconfig.json`s used throughout the monorepo

Each package/app is 100% [TypeScript](https://www.typescriptlang.org/), except for the Rust-based service.

### Legal Website Scraper Service

The Rust-based legal website scraper service is designed to efficiently crawl and extract content from legal websites. Key features include:

- **High Performance**: Built with Rust for optimal speed and resource efficiency
- **Configurable Crawling**: Supports custom crawling rules, depth limits, and URL patterns
- **Content Processing**: Converts HTML to Markdown for easier consumption
- **Scalable Architecture**: Uses Redis for job queuing and PostgreSQL for data persistence
- **S3 Storage**: Stores crawled content in S3-compatible storage
- **RESTful API**: Provides a comprehensive API for managing scraper configurations, jobs, and results
- **Webhook Support**: Notifies external systems about scraping events
- **Analytics**: Tracks performance metrics and crawling statistics

#### Service Architecture

The service follows a clean architecture approach with clear separation of concerns:

- **Domain Layer**: Core business entities and logic (jobs, pages, configs, webhooks)
- **Application Layer**: Use cases and service coordination
- **Infrastructure Layer**: External systems integration (database, queue, storage)
- **API Layer**: RESTful endpoints with HATEOAS principles

#### Technology Stack

- **Axum**: Web framework for building the API
- **Tokio**: Asynchronous runtime
- **SQLx**: Type-safe database access
- **Redis**: Job queuing and distributed processing
- **AWS SDK**: S3-compatible storage integration
- **Tonic**: gRPC client for Markdown conversion

#### API Examples

The service provides a RESTful API with the following endpoints:

**Health Check**
```bash
curl http://localhost:8080/health
# Response: {"status":"ok","version":"0.1.0"}
```

**Create Scraper Configuration**
```bash
curl -X POST http://localhost:8080/api/configs \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Example Law Firm",
    "base_url": "https://example-law.com",
    "include_patterns": [".*\\.html$", ".*/articles/.*"],
    "exclude_patterns": [".*/admin/.*", ".*/login/.*"],
    "max_depth": 3,
    "max_pages_per_job": 100,
    "respect_robots_txt": true,
    "user_agent": "FortaiBot/1.0",
    "request_delay_ms": 1000
  }'
```

**List Configurations**
```bash
curl http://localhost:8080/api/configs
```

**Start a Scraping Job**
```bash
curl -X POST http://localhost:8080/api/configs/{config_id}/start
```

**Get Job Status**
```bash
curl http://localhost:8080/api/jobs/{job_id}
```

**List Pages Crawled by a Job**
```bash
curl http://localhost:8080/api/pages?job_id={job_id}
```

**Get Page Content**
```bash
curl http://localhost:8080/api/pages/{page_id}/markdown
```

**Register a Webhook**
```bash
curl -X POST http://localhost:8080/api/webhooks \
  -H "Content-Type: application/json" \
  -d '{
    "config_id": "{config_id}",
    "url": "https://your-service.com/webhook",
    "events": ["job.completed", "page.crawled"],
    "description": "Notification endpoint for completed jobs"
  }'
```

#### Setup and Configuration

The service requires the following dependencies:

- PostgreSQL database
- Redis server
- S3-compatible storage (MinIO for local development)
- Markdown conversion service (optional)

Configuration is managed through environment variables or a `.env` file:

```
# API Configuration
API_PORT=8080
API_HOST=0.0.0.0

# Database Configuration
DATABASE_URL=postgres://postgres:postgres@localhost:5432/scraper_dev
DATABASE_MAX_CONNECTIONS=10

# Redis Configuration
REDIS_URL=redis://localhost:6379
REDIS_POOL_SIZE=20

# S3 Storage Configuration
S3_ENDPOINT=http://localhost:9000
S3_REGION=us-east-1
S3_BUCKET=scraper-dev
S3_ACCESS_KEY=minioadmin
S3_SECRET_KEY=minioadmin

# Markdown Service Configuration
MARKDOWN_SERVICE_URL=http://localhost:50051

# Scraper Configuration
SCRAPER_DEFAULT_USER_AGENT=FortaiBot/1.0
SCRAPER_DEFAULT_DELAY_MS=1000
SCRAPER_MAX_CONCURRENT_REQUESTS=10
SCRAPER_MAX_RETRIES=3

# Scheduler Configuration
SCHEDULER_ENABLED=true
SCHEDULER_CHECK_INTERVAL_SECONDS=60
```

#### Database Schema

The service uses the following main tables:

- `scraper_configs`: Stores configuration for crawling websites
- `jobs`: Tracks scraping jobs and their status
- `pages`: Stores information about crawled pages
- `webhooks`: Manages webhook registrations for event notifications

#### Running with Docker

A Docker Compose setup is provided for local development:

```bash
cd apps/service
docker-compose up -d
```

This starts PostgreSQL, Redis, and MinIO containers for local development.

### Utilities

This Turborepo has some additional tools already setup for you:

- [TypeScript](https://www.typescriptlang.org/) for static type checking
- [ESLint](https://eslint.org/) for code linting
- [Prettier](https://prettier.io) for code formatting
- [Rust](https://www.rust-lang.org/) for high-performance services

### Build

To build all apps and packages, run the following command:

```
cd my-turborepo
pnpm build
```

To build the Rust service:

```
cd my-turborepo/apps/service
cargo build
```

### Develop

To develop all apps and packages, run the following command:

```
cd my-turborepo
pnpm dev
```

For the Rust service:

```
cd my-turborepo/apps/service
cargo run --bin scraper-service
```

### Database Migrations

To run database migrations for the Rust service:

```
cd my-turborepo/apps/service
cargo run --bin migrate
```

### Remote Caching

> [!TIP]
> Vercel Remote Cache is free for all plans. Get started today at [vercel.com](https://vercel.com/signup?/signup?utm_source=remote-cache-sdk&utm_campaign=free_remote_cache).

Turborepo can use a technique known as [Remote Caching](https://turbo.build/repo/docs/core-concepts/remote-caching) to share cache artifacts across machines, enabling you to share build caches with your team and CI/CD pipelines.

By default, Turborepo will cache locally. To enable Remote Caching you will need an account with Vercel. If you don't have an account you can [create one](https://vercel.com/signup?utm_source=turborepo-examples), then enter the following commands:

```
cd my-turborepo
npx turbo login
```

This will authenticate the Turborepo CLI with your [Vercel account](https://vercel.com/docs/concepts/personal-accounts/overview).

Next, you can link your Turborepo to your Remote Cache by running the following command from the root of your Turborepo:

```
npx turbo link
```

## Useful Links

Learn more about the power of Turborepo:

- [Tasks](https://turbo.build/repo/docs/core-concepts/monorepos/running-tasks)
- [Caching](https://turbo.build/repo/docs/core-concepts/caching)
- [Remote Caching](https://turbo.build/repo/docs/core-concepts/remote-caching)
- [Filtering](https://turbo.build/repo/docs/core-concepts/monorepos/filtering)
- [Configuration Options](https://turbo.build/repo/docs/reference/configuration)
- [CLI Usage](https://turbo.build/repo/docs/reference/command-line-reference)
