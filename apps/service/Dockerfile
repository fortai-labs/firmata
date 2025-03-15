FROM rust:1.76 as builder

# Create app directory
WORKDIR /app

# Copy source code
COPY . .

# Build the application
RUN cargo build --release

FROM debian:bullseye-slim

# Install dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy binaries and configuration from builder
COPY --from=builder /app/target/release/scraper-service /app/scraper-service
COPY --from=builder /app/target/release/migrate /app/migrate
COPY --from=builder /app/config/ /app/config/
COPY --from=builder /app/.env.example /app/.env.example
COPY --from=builder /app/README.md /app/README.md

# Make binaries executable
RUN chmod +x /app/scraper-service /app/migrate

# Expose the service port
EXPOSE 8080

# Set environment variables
ENV RUN_MODE=production
ENV CONFIG_DIR=/app/config

# Run the service
CMD ["/app/scraper-service"] 