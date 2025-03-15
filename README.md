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
cargo run
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
