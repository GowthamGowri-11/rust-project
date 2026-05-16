# Quick Start Guide - Part 4 Features

## Starting the System

### Windows
```powershell
.\scripts\deploy.ps1
```

### Linux/Mac
```bash
chmod +x scripts/*.sh
./scripts/deploy.sh
```

## Accessing Services

- **Dashboard API**: http://localhost:8080
- **Prometheus**: http://localhost:9091
- **Grafana**: http://localhost:3000 (admin/admin)
- **Controller**: localhost:6653

## Testing Part 4 Features

### 1. Check System Health
```bash
curl http://localhost:8080/api/v1/health
```

### 2. View Topology Heatmap
```bash
curl http://localhost:8080/api/v1/topology/heatmap | jq '.'
```

### 3. Get Performance Metrics
```bash
curl http://localhost:8080/api/v1/performance | jq '.'
```

### 4. Check Resilience Status
```bash
curl http://localhost:8080/api/v1/resilience/status | jq '.'
```

### 5. Run a Benchmark
```bash
curl -X POST http://localhost:8080/api/v1/benchmark/run \
  -H "Content-Type: application/json" \
  -d '{
    "name": "test-benchmark",
    "duration_secs": 60,
    "num_flows": 10
  }'
```

### 6. Get Benchmark Results
```bash
curl http://localhost:8080/api/v1/benchmark/results | jq '.'
```

### 7. Trigger Chaos Scenario
```bash
# Link failure
curl -X POST http://localhost:8080/api/v1/chaos/trigger \
  -H "Content-Type: application/json" \
  -d '{
    "scenario_type": "LinkFailure",
    "target": "link-s1-s2",
    "duration_ms": 5000
  }'

# Congestion burst
curl -X POST http://localhost:8080/api/v1/chaos/trigger \
  -H "Content-Type: application/json" \
  -d '{
    "scenario_type": "CongestionBurst",
    "target": "link-s2-s3",
    "duration_ms": 3000
  }'
```

## Running Test Scripts

### Chaos Engineering Tests
```bash
# Linux/Mac
./scripts/test-chaos.sh

# Windows (manual)
# Run the curl commands from the script manually
```

### Benchmarking Tests
```bash
# Linux/Mac
./scripts/run-benchmark.sh

# Windows (manual)
# Run the curl commands from the script manually
```

## Viewing Logs

```bash
# All services
docker-compose logs -f

# Specific service
docker-compose logs -f rustflow-api
docker-compose logs -f prometheus
docker-compose logs -f grafana
```

## Stopping the System

```bash
docker-compose down
```

## Restarting Services

```bash
# Restart all
docker-compose restart

# Restart specific service
docker-compose restart rustflow-api
```

## Monitoring with Prometheus

1. Open http://localhost:9091
2. Go to Status → Targets
3. Verify rustflow-api is UP
4. Query metrics:
   - `rustflow_bandwidth_total`
   - `rustflow_latency_ms`
   - `rustflow_packet_loss_rate`

## Visualizing with Grafana

1. Open http://localhost:3000
2. Login: admin/admin
3. Add Prometheus data source:
   - URL: http://prometheus:9090
4. Import dashboard from `deployments/grafana/dashboards/rustflow.json`

## API Endpoints Reference

### Core Endpoints
- `GET /` - API info
- `GET /api/v1/health` - Health check
- `GET /api/v1/topology` - Network topology
- `GET /api/v1/switches` - Switch list
- `GET /api/v1/flows` - Flow list
- `GET /api/v1/metrics` - Network metrics
- `POST /api/v1/routes/optimize` - Trigger optimization

### Part 4 Endpoints
- `GET /api/v1/topology/heatmap` - Topology with heatmap
- `GET /api/v1/performance` - Performance metrics
- `GET /api/v1/resilience/status` - Resilience status
- `POST /api/v1/benchmark/run` - Run benchmark
- `GET /api/v1/benchmark/results` - Get results
- `POST /api/v1/chaos/trigger` - Trigger chaos

### Metrics
- `GET /metrics` - Prometheus metrics

## Troubleshooting

### Port Already in Use
```bash
# Check what's using the port
netstat -ano | findstr :8080

# Kill the process (Windows)
taskkill /PID <PID> /F

# Or change the port in docker-compose.yml
```

### Docker Not Running
```bash
# Start Docker Desktop
# Or start Docker service
sudo systemctl start docker
```

### Services Not Starting
```bash
# Check logs
docker-compose logs

# Rebuild images
docker-compose build --no-cache

# Remove old containers
docker-compose down -v
docker-compose up -d
```

### API Not Responding
```bash
# Check if container is running
docker ps

# Check container logs
docker logs rustflow-api

# Restart container
docker-compose restart rustflow-api
```

## Development Mode

### Build Locally
```bash
cargo build --release
```

### Run API Locally
```bash
cargo run --bin dashboard_api
```

### Run Tests
```bash
cargo test
```

### Check Code
```bash
cargo clippy
cargo fmt --check
```

## Configuration

### Environment Variables
Create `.env` file:
```env
RUST_LOG=info
API_HOST=0.0.0.0
API_PORT=8080
PROMETHEUS_PORT=9090
```

### Config File
Edit `configs/default.toml`:
```toml
[api]
host = "0.0.0.0"
port = 8080

[controller]
host = "0.0.0.0"
port = 6653

[monitoring]
ebpf_enabled = false
collection_interval_ms = 1000
```

## Next Steps

1. Explore the API endpoints
2. Run chaos engineering tests
3. Execute benchmarks
4. View metrics in Grafana
5. Customize dashboards
6. Configure alerts
7. Deploy to production

## Support

- Documentation: `docs/`
- Architecture: `docs/ARCHITECTURE.md`
- API Reference: `docs/API.md`
- Part 4 Details: `docs/PART4_RESILIENCE_BENCHMARKING.md`
- Completion Summary: `PART4_COMPLETION_SUMMARY.md`

---

**Happy Testing! 🚀**
