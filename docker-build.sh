#!/bin/bash
# Docker-based build script - avoids Windows cargo issues

# Build in Docker container
docker run --rm \
  -v "$(pwd)":/workspace \
  -w /workspace \
  rust:1.75 \
  cargo build --release

# Run tests in Docker
docker run --rm \
  -v "$(pwd)":/workspace \
  -w /workspace \
  rust:1.75 \
  cargo test --all

echo "Build complete! Binaries in target/release/"
