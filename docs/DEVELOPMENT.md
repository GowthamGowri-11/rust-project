# Development Guide

## Prerequisites

- Rust 1.75+
- Docker & Docker Compose
- Linux kernel 5.10+ (for eBPF)

## Setup

1. Clone the repository
2. Copy `.env.example` to `.env`
3. Run `make build`

## Building

```bash
# Debug build
make build

# Release build
make release

# Run tests
make test

# Format code
make fmt

# Run clippy
make clippy
```

## Running Locally

```bash
# Start API server
make run

# Or with debug logging
make dev
```

## Docker Development

```bash
# Build containers
make docker-build

# Start services
make docker-up

# View logs
make docker-logs

# Stop services
make docker-down
```

## Adding a New Crate

1. Create crate: `cargo new --lib crates/my_crate`
2. Add to workspace in root `Cargo.toml`
3. Implement service trait
4. Add tests

## Testing

```bash
# Run all tests
cargo test --workspace

# Run specific crate tests
cargo test -p controller

# Run with output
cargo test -- --nocapture
```

## Code Style

- Use `rustfmt` for formatting
- Run `clippy` before committing
- Follow async-first patterns
- Use trait-based abstractions
- Comprehensive error handling

## Debugging

Enable debug logging:
```bash
RUST_LOG=debug cargo run
```

## Contributing

1. Fork the repository
2. Create feature branch
3. Make changes
4. Run tests and linters
5. Submit pull request
