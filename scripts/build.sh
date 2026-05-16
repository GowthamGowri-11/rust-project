#!/bin/bash
set -e

echo "Building RustFlow-AI..."

cargo build --release --workspace

echo "Build complete!"
echo "Binaries available in target/release/"
