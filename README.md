# 🫂 Firmata -- Legal-Oriented Web Scraper as a Service

## Introduction

Firmata is a cutting-edge, high-performance web scraping platform specifically engineered for the legal domain. Built with Rust at its core, this service combines blazing-fast performance with enterprise-grade reliability to transform how legal professionals gather and process web-based legal content.

In today's data-driven legal landscape, the ability to efficiently extract, process, and analyze content from legal websites is a critical competitive advantage. Firmata addresses this need by providing a robust, scalable solution that can handle everything from small targeted crawls to large-scale data acquisition projects with minimal resource overhead.

What sets Firmata apart:

- **Unparalleled Performance**: Leveraging Rust's zero-cost abstractions and memory safety guarantees, Firmata delivers exceptional throughput while maintaining minimal resource footprint
- **Legal-Domain Awareness**: Purpose-built for legal content with specialized handling for case law, statutes, regulations, and legal commentary
- **Enterprise Scalability**: Designed from the ground up to scale horizontally across distributed infrastructure
- **Compliance-First Approach**: Built-in respect for robots.txt, configurable request delays, and ethical crawling practices
- **Seamless Integration**: Comprehensive API and webhook system to integrate with existing legal tech stacks

## Service Architecture

Firmata implements a sophisticated clean architecture that ensures maintainability, testability, and extensibility as the platform evolves. This architectural approach creates clear boundaries between system concerns, allowing for independent evolution of components.

### Core Architectural Layers

![Firmata Architecture](https://via.placeholder.com/800x400?text=Firmata+Architecture+Diagram)

#### Domain Layer

The heart of Firmata, containing:

- **Rich Domain Models**: Comprehensive representations of core entities (scraper configurations, jobs, pages, webhooks)
- **Domain Services**: Encapsulated business logic for content extraction, URL normalization, and content classification
- **Domain Events**: Event-driven architecture enabling reactive processing and extensibility
- **Value Objects**: Immutable, self-validating types ensuring data integrity throughout the system

#### Application Layer

The orchestration layer that coordinates domain activities:

- **Use Cases**: Discrete, focused operations that implement specific business requirements
- **Command/Query Handlers**: Implementation of CQRS principles for clear separation of read and write operations
- **Application Services**: Coordination of multiple domain operations into cohesive workflows
- **Event Handlers**: Processing of domain events to trigger side effects and maintain system consistency

#### Infrastructure Layer

The technical foundation enabling external system integration:

- **Database Adapters**: Type-safe PostgreSQL integration via SQLx with optimized query patterns
- **Queue Management**: Redis-based job distribution with prioritization and failure handling
- **Storage Services**: S3-compatible content storage with efficient binary handling
- **HTTP Clients**: Robust web clients with retry logic, circuit breaking, and rate limiting
- **Markdown Processing**: High-performance HTML-to-Markdown conversion pipeline

#### API Layer

The external interface exposing Firmata's capabilities:

- **RESTful Endpoints**: Comprehensive API following REST best practices
- **HATEOAS Implementation**: Hypermedia controls for discoverable API navigation
- **Authentication/Authorization**: Secure access control with fine-grained permissions
- **Rate Limiting**: Protection against API abuse
- **Comprehensive Documentation**: OpenAPI/Swagger specifications for all endpoints

### Cross-Cutting Concerns

- **Observability**: Structured logging, metrics collection, and distributed tracing
- **Error Handling**: Consistent error representation and recovery strategies
- **Configuration Management**: Environment-based configuration with sensible defaults
- **Security**: Input validation, output encoding, and protection against common vulnerabilities

This architecture enables Firmata to deliver exceptional performance while maintaining the flexibility to adapt to evolving legal data acquisition needs. The clean separation of concerns allows for targeted optimization and independent scaling of system components based on workload characteristics.

---

By leveraging this sophisticated architecture, Firmata transforms the complex challenge of legal web scraping into a streamlined, reliable process that legal professionals can depend on for their most critical data acquisition needs.



# API Examples

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
