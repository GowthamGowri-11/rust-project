# RustFlow-AI

**AI-Driven SDN Traffic Engineering System in Rust**

## Overview

RustFlow-AI is a production-grade, Rust-native intelligent Software-Defined Networking (SDN) system that combines real-time network monitoring, machine learning-based traffic prediction, and dynamic routing optimization.

## Features

- 🚀 **Async-First Architecture** - Built on tokio for high-performance concurrent operations
- 🧠 **ML-Powered Decision Making** - ONNX-based inference for traffic prediction and congestion detection
- 🔄 **Dynamic Traffic Engineering** - Real-time path optimization and load balancing
- 📊 **eBPF Monitoring** - Low-overhead network telemetry collection
- 🛡️ **Automatic Failure Recovery** - Self-healing network with backup path management
- 📈 **Production Metrics** - Prometheus integration for observability
- 🎯 **OpenFlow Control** - Direct switch management and flow rule installation

## Architecture

```
rustflow-ai/
├── crates/
│   ├── controller/       # OpenFlow switch management
│   ├── monitoring/       # eBPF-based traffic monitoring
│   ├── analytics/        # Traffic analysis and feature extraction
│   ├── ml_engine/        # ML inference engine
│   ├── optimizer/        # Path selection and load balancing
│   ├── resilience/       # Failure detection and recovery
│   ├── metrics/          # Prometheus exporters
│   └── dashboard_api/    # REST API server
├── configs/              # Configuration files
├── scripts/              # Build and deployment scripts
├── deployments/          # Docker and K8s manifests
└── docs/                 # Documentation
```

## Quick Start

### Prerequisites

- Rust 1.75+ (with cargo)
- Docker & Docker Compose
- Linux kernel 5.10+ (for eBPF support)

### Build

```bash
# Build all crates
make build

# Build in release mode
make release

# Run tests
make test
```

### Run

```bash
# Start all services with Docker Compose
docker-compose up -d

# Or run locally
cargo run --bin dashboard_api
```

### Configuration

Copy `.env.example` to `.env` and adjust settings:

```bash
cp .env.example .env
```

## Development

### Project Structure

Each crate follows a consistent structure:

```
crate_name/
├── Cargo.toml
├── src/
│   ├── lib.rs          # Public API
│   ├── service.rs      # Core service logic
│   ├── types.rs        # Data structures
│   └── error.rs        # Error types
└── tests/
```

### Adding a New Module

```bash
cargo new --lib crates/my_module
```

Then add it to the workspace `Cargo.toml`.

## Modules

### Controller
OpenFlow communication, switch management, and flow rule installation.

### Monitoring
eBPF-based packet capture, latency tracking, and bandwidth monitoring.

### Analytics
Traffic pattern analysis, congestion detection, and feature extraction for ML.

### ML Engine
ONNX model loading, inference execution, and traffic prediction.

### Optimizer
Path computation, load balancing algorithms, and traffic prioritization.

### Resilience
Failure detection, automatic recovery, and backup path management.

### Metrics
Prometheus metrics exporters and performance benchmarking.

### Dashboard API
REST API for topology visualization, metrics queries, and control operations.

## API Endpoints

```
GET  /api/v1/topology          # Network topology
GET  /api/v1/switches          # Connected switches
GET  /api/v1/flows             # Active flows
GET  /api/v1/metrics           # System metrics
POST /api/v1/routes/optimize   # Trigger optimization
GET  /api/v1/health            # Health check
```

## Metrics

Prometheus metrics available at `:9090/metrics`

- Network throughput
- Packet loss rates
- Latency percentiles
- Flow table utilization
- ML inference latency
- Optimization execution time

## Roadmap

- [x] Core architecture and workspace setup
- [ ] OpenFlow 1.3 controller implementation
- [ ] eBPF monitoring probes
- [ ] ONNX inference integration
- [ ] Shortest path optimizer
- [ ] Failure detection system
- [ ] REST API implementation
- [ ] Distributed deployment support
- [ ] Advanced ML models (GNN-based)
- [ ] Multi-controller clustering

## Contributing

Contributions welcome! Please read CONTRIBUTING.md first.

## License

MIT License - see LICENSE file for details.

## Contact

- GitHub: https://github.com/rustflow-ai/core
- Issues: https://github.com/rustflow-ai/core/issues
