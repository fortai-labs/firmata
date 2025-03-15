FROM debian:bullseye-slim

# Install dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy binaries and configuration
COPY scraper-service /app/scraper-service
COPY migrate /app/migrate
COPY config/ /app/config/
COPY .env.example /app/.env.example
COPY README.md /app/README.md

# Make binaries executable
RUN chmod +x /app/scraper-service /app/migrate

# Expose the service port
EXPOSE 8080

# Set environment variables
ENV RUN_MODE=production
ENV CONFIG_DIR=/app/config

# Run the service
CMD ["/app/scraper-service"] 