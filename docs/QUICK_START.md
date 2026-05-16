# Quick Start Guide

## For Windows Users (Current System)

### Step 1: Install Rust

Open PowerShell and run:
```powershell
# Download Rust installer
Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile "$env:TEMP\rustup-init.exe"

# Run installer
& "$env:TEMP\rustup-init.exe"
```

Follow the prompts and choose option 1 (default installation).

**Important:** Close and reopen your terminal after installation!

### Step 2: Install C++ Build Tools

Rust on Windows requires Visual Studio C++ Build Tools.

Download from: https://visualstudio.microsoft.com/visual-cpp-build-tools/

Or use winget:
```powershell
winget install Microsoft.VisualStudio.2022.BuildTools
```

Select "Desktop development with C++" during installation.

### Step 3: Verify Installation

```bash
cargo --version
rustc --version
```

You should see version numbers (1.75.0 or higher).

### Step 4: Build RustFlow-AI

```bash
# Navigate to project directory
cd rustflow-ai

# Build the project (this will take a few minutes first time)
cargo build --release
```

### Step 5: Run the Application

```bash
# Run the dashboard API
cargo run --bin dashboard_api
```

Or run the compiled binary:
```bash
.\target\release\dashboard_api.exe
```

### Step 6: Test the API

Open browser and visit:
- http://localhost:8080/api/v1/health

Or use PowerShell:
```powershell
Invoke-WebRequest -Uri "http://localhost:8080/api/v1/health"
```

## Using Docker (Alternative - No Rust Installation Needed)

If you have Docker Desktop installed:

```bash
# Build and start all services
docker-compose up -d

# View logs
docker-compose logs -f rustflow-api

# Access services
# API: http://localhost:8080
# Grafana: http://localhost:3000
# Prometheus: http://localhost:9091
```

## Common Issues

### "cargo not recognized"
- Restart your terminal after installing Rust
- Check PATH includes: `%USERPROFILE%\.cargo\bin`

### "link.exe not found"
- Install Visual Studio C++ Build Tools
- Restart terminal

### Build takes too long
- First build downloads dependencies (5-10 minutes)
- Subsequent builds are much faster
- Use `cargo build` (debug) instead of `--release` for faster compilation

## What's Next?

- Read [ARCHITECTURE.md](ARCHITECTURE.md) to understand the system
- Read [API.md](API.md) for API documentation
- Read [DEVELOPMENT.md](DEVELOPMENT.md) for development workflow
- Explore the code in `crates/` directory
