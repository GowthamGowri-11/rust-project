# Deployment Guide

## Docker Deployment

### Quick Start

```bash
# Build and start all services
docker-compose up -d

# View logs
docker-compose logs -f

# Stop services
docker-compose down
```

### Services

- **rustflow-api**: REST API server (port 8080)
- **rustflow-controller**: OpenFlow controller (port 6653)
- **prometheus**: Metrics collection (port 9091)
- **grafana**: Visualization dashboard (port 3000)

## Manual Deployment

### Build

```bash
cargo build --release
```

### Run

```bash
# Start API server
./target/release/dashboard_api

# Configure environment
export RUST_LOG=info
export API_PORT=8080
```

## Kubernetes Deployment

Coming soon.

## Configuration

Edit `.env` file or use environment variables:

```bash
CONTROLLER_HOST=0.0.0.0
CONTROLLER_PORT=6653
API_PORT=8080
EBPF_ENABLED=false
```

## Monitoring

Access Grafana at http://localhost:3000
- Username: admin
- Password: admin

## Troubleshooting

### Port conflicts
Change ports in `docker-compose.yml` or `.env`

### eBPF not working
Ensure Linux kernel 5.10+ and proper capabilities

### Connection issues
Check firewall rules and network configuration
