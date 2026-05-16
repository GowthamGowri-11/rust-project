# RustFlow-AI Deployment Script (PowerShell)

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "RustFlow-AI Deployment" -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan

# Check if Docker is running
try {
    docker info | Out-Null
} catch {
    Write-Host "Error: Docker is not running" -ForegroundColor Red
    exit 1
}

# Build Docker images
Write-Host "Building Docker images..." -ForegroundColor Yellow
docker-compose build

# Start services
Write-Host "Starting services..." -ForegroundColor Yellow
docker-compose up -d

# Wait for services to be ready
Write-Host "Waiting for services to start..." -ForegroundColor Yellow
Start-Sleep -Seconds 10

# Check service health
Write-Host "Checking service health..." -ForegroundColor Yellow
try {
    $response = Invoke-WebRequest -Uri "http://localhost:8080/api/v1/health" -UseBasicParsing
    Write-Host "API health check: OK" -ForegroundColor Green
} catch {
    Write-Host "Warning: API health check failed" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "Deployment Complete!" -ForegroundColor Green
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "Services:"
Write-Host "  - Dashboard API:  http://localhost:8080"
Write-Host "  - Prometheus:     http://localhost:9091"
Write-Host "  - Grafana:        http://localhost:3000 (admin/admin)"
Write-Host "  - Controller:     localhost:6653"
Write-Host ""
Write-Host "Useful commands:"
Write-Host "  - View logs:      docker-compose logs -f"
Write-Host "  - Stop services:  docker-compose down"
Write-Host "  - Restart:        docker-compose restart"
Write-Host "==========================================" -ForegroundColor Cyan
