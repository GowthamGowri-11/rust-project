#!/bin/bash
# RustFlow-AI Deployment Script

set -e

echo "=========================================="
echo "RustFlow-AI Deployment"
echo "=========================================="

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "Error: Docker is not running"
    exit 1
fi

# Build Docker images
echo "Building Docker images..."
docker-compose build

# Start services
echo "Starting services..."
docker-compose up -d

# Wait for services to be ready
echo "Waiting for services to start..."
sleep 10

# Check service health
echo "Checking service health..."
curl -f http://localhost:8080/api/v1/health || echo "Warning: API health check failed"

echo ""
echo "=========================================="
echo "Deployment Complete!"
echo "=========================================="
echo "Services:"
echo "  - Dashboard API:  http://localhost:8080"
echo "  - Prometheus:     http://localhost:9091"
echo "  - Grafana:        http://localhost:3000 (admin/admin)"
echo "  - Controller:     localhost:6653"
echo ""
echo "Useful commands:"
echo "  - View logs:      docker-compose logs -f"
echo "  - Stop services:  docker-compose down"
echo "  - Restart:        docker-compose restart"
echo "=========================================="
