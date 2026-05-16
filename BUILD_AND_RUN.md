# 🔨 Build and Run - RustFlow-AI

## Quick Commands

### Build the Project
```bash
# Navigate to project directory
cd C:\Users\GOWTHAMGOWRI\Desktop\rsut-project

# Clean previous build (optional)
cargo clean

# Build in release mode (optimized)
cargo build --release

# Or build in debug mode (faster compilation)
cargo build
```

### Run the Server
```bash
# Option 1: Run with cargo
cargo run --bin dashboard_api

# Option 2: Run the compiled binary
.\target\release\dashboard_api.exe

# Option 3: Double-click run.bat
```

### Test the API
```bash
# Open PowerShell and test endpoints
Invoke-WebRequest -Uri "http://localhost:8080/api/v1/health"

# Or run the test script
.\test_api.ps1

# Or open browser
# http://localhost:8080/api/v1/health
```

---

## What to Expect

### During Build:
```
   Compiling controller v0.1.0
   Compiling monitoring v0.1.0
   Compiling analytics v0.1.0
   Compiling ml_engine v0.1.0
   Compiling optimizer v0.1.0
   Compiling resilience v0.1.0
   Compiling metrics v0.1.0
   Compiling dashboard_api v0.1.0
    Finished `release` profile [optimized] target(s) in 3.86s
```

### When Server Starts:
```
Starting RustFlow-AI Dashboard API
Configuration loaded: API will listen on 0.0.0.0:8080
Initializing Prometheus metrics
Listening on 0.0.0.0:8080
```

### When You Make a Request:
```
method=GET uri=/api/v1/health status=200 duration_ms=2 "Request completed"
```

---

## New Features to Test

### 1. Enhanced Health Check
```bash
curl http://localhost:8080/api/v1/health
```

Expected response:
```json
{
  "status": "healthy",
  "version": "0.1.0",
  "services": {
    "controller": "ready",
    "monitoring": "ready",
    "ml_engine": "ready",
    "optimizer": "ready"
  },
  "config": {
    "api_port": 8080,
    "controller_port": 6653,
    "monitoring_enabled": false
  }
}
```

### 2. Real Metrics
```bash
curl http://localhost:8080/api/v1/metrics
```

Expected response:
```json
{
  "bandwidth_bps": 0,
  "latency_ms": 0.0,
  "packet_loss": 0.0,
  "active_flows": 0,
  "timestamp": "2026-05-15T16:30:00Z"
}
```

### 3. Optimization with UUID
```bash
curl -X POST http://localhost:8080/api/v1/routes/optimize
```

Expected response:
```json
{
  "message": "Optimization started",
  "job_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "pending"
}
```

---

## Troubleshooting

### Build Fails
```bash
# Clean and rebuild
cargo clean
cargo build --release
```

### Port Already in Use
Edit `.env` or `configs/default.toml`:
```toml
[api]
port = 8081  # Change to different port
```

### Can't Access API
- Check if server is running
- Try `http://127.0.0.1:8080` instead of `localhost`
- Check firewall settings

---

## Development Workflow

### Make Changes
1. Edit files in `crates/*/src/`
2. Save changes
3. Stop server (Ctrl+C)
4. Rebuild: `cargo build`
5. Run again: `cargo run --bin dashboard_api`

### Quick Iteration
```bash
# Use cargo watch for auto-rebuild (install first)
cargo install cargo-watch

# Auto-rebuild and run on file changes
cargo watch -x 'run --bin dashboard_api'
```

---

## What's New

See **WHATS_NEW.md** for details on:
- Configuration system
- Service integration
- Error handling
- Enhanced API responses
- Request middleware
- CORS support

---

## Next Steps

1. ✅ Build the project
2. ✅ Run the server
3. ✅ Test the enhanced endpoints
4. ✅ Read IMPLEMENTATION_ROADMAP.md
5. ✅ Start implementing features

**Ready to build! Run `cargo build --release` in your PowerShell! 🚀**
