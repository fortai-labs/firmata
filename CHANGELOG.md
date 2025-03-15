# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Rust-based legal website scraper microservice
  - Domain models for scraper configurations, jobs, pages, and webhooks
  - Infrastructure components for database, Redis queue, S3 storage, and gRPC
  - Application services and worker processes
  - RESTful API with comprehensive endpoints for managing scraper operations
  - Database migrations for PostgreSQL
  - Webhook support for event notifications
  - Analytics endpoints for monitoring scraper performance
  - Crawler module with rate limiting and politeness controls
    - Domain-specific rate limiting
    - Respect for robots.txt directives
    - Configurable concurrent request limits
    - Automatic retry logic for transient errors
    - URL filtering based on include/exclude patterns
    - HTML content processing with link extraction
    - Content size limits for safety
  - Comprehensive test suite
    - Unit tests for crawler functionality
    - Integration tests with mock HTTP server
    - Tests for rate limiting and concurrency
    - Tests for robots.txt parsing and compliance
    - Tests for URL filtering and pattern matching

- Knowledge base enhancements
  - ADR 005: Rust-based Legal Website Scraper Microservice
  - ADR 005-A: Rust-based Legal Website Scraper Service Implementation
  - ADR 006: ADR Naming Convention for Related Decisions
  - ADR-RULE-003: Related ADR Naming Convention
  - Updated ADR template with Related ADRs section

### Changed

- Updated project README with information about the Rust-based legal website scraper service
- Enhanced Turbo configuration to support Rust-based services
- Improved configuration system with environment variables and config files
- Refactored worker implementation to use the dedicated crawler module

### Removed

- Docs application (replaced with service documentation)

## [0.1.0] - 2024-03-15

### Added

- Initial project setup with Turborepo
- Basic Next.js applications for web and docs
- Shared UI components
- ESLint and TypeScript configurations
