# Installation Guide

## Prerequisites

### Windows Installation

#### 1. Install Rust

Download and run the Rust installer:
- Visit: https://rustup.rs/
- Download: `rustup-init.exe`
- Run the installer and follow prompts
- Choose default installation (option 1)

Or use PowerShell:
```powershell
# Download and run rustup installer
Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile "$env:TEMP\rustup-init.exe"
& "$env:TEMP\rustup-init.exe" -y
```

After installation, restart your terminal and verify:
```bash
cargo --version
rustc --version
```

#### 2. Install Visual Studio C++ Build Tools (Required for Rust on Windows)

Download from: https://visualstudio.microsoft.com/visual-cpp-build-tools/

Or install via winget:
```powershell
winget install Microsoft.VisualStudio.2022.BuildTools
```

During installation, select:
- Desktop development with C++
- Windows 10/11 SDK

#### 3. Install Docker Desktop (Optional, for containerized deployment)

Download from: https://www.docker.com/products/docker-desktop/

Or via winget:
```powershell
winget install Docker.DockerDesktop
```

### Linux Installation

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install build essentials
sudo apt-get update
sudo apt-get install build-essential pkg-config libssl-dev

# Install Docker (optional)
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh
```

### macOS Installation

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Xcode Command Line Tools
xcode-select --install

# Install Docker (optional)
brew install --cask docker
```

## Building RustFlow-AI

### Step 1: Clone or Navigate to Project

```bash
cd rustflow-ai
```

### Step 2: Verify Rust Installation

```bash
cargo --version
# Should output: cargo 1.75.0 or higher
```

### Step 3: Build the Project

```bash
# Debug build (faster compilation, slower runtime)
cargo build

# Release build (slower compilation, optimized runtime)
cargo build --release
```

### Step 4: Run Tests

```bash
cargo test --workspace
```

### Step 5: Run the Application

```bash
# Run the dashboard API
cargo run --bin dashboard_api

# Or use the Makefile
make run
```

## Docker Deployment (Alternative)

If you prefer containerized deployment:

```bash
# Build containers
docker-compose build

# Start all services
docker-compose up -d

# View logs
docker-compose logs -f

# Stop services
docker-compose down
```

## Troubleshooting

### Windows: "cargo not recognized"
- Restart your terminal after installing Rust
- Add to PATH: `%USERPROFILE%\.cargo\bin`

### Windows: "link.exe not found"
- Install Visual Studio C++ Build Tools
- Restart terminal after installation

### Linux: "OpenSSL not found"
```bash
sudo apt-get install libssl-dev pkg-config
```

### Permission denied on scripts
```bash
# Linux/macOS
chmod +x scripts/*.sh

# Windows: Run PowerShell as Administrator
```

## Verification

After successful installation and build:

```bash
# Check binary was created
ls target/release/dashboard_api  # Linux/macOS
dir target\release\dashboard_api.exe  # Windows

# Run the application
./target/release/dashboard_api  # Linux/macOS
.\target\release\dashboard_api.exe  # Windows
```

The API should start on http://localhost:8080

## Next Steps

1. Copy `.env.example` to `.env` and configure
2. Read `docs/DEVELOPMENT.md` for development workflow
3. Read `docs/DEPLOYMENT.md` for production deployment
4. Access API at http://localhost:8080/api/v1/health
