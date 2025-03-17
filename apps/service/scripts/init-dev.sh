#!/bin/bash
set -e

# Start the development environment
echo "Starting development environment..."
docker compose up -d postgres redis minio

# Wait for services to be ready
echo "Waiting for services to be ready..."
sleep 5

# Create the bucket in MinIO
echo "Creating bucket in MinIO..."
docker run --rm --network host \
  -e AWS_ACCESS_KEY_ID=minioadmin \
  -e AWS_SECRET_ACCESS_KEY=minioadmin \
  amazon/aws-cli --endpoint-url http://localhost:9000 \
  s3 mb s3://scraper-dev --region us-east-1

# Run database migrations
echo "Running database migrations..."
cargo run --bin migrate

echo "Development environment is ready!"
echo "You can now run the service with: cargo run" 