#!/bin/bash
set -e

echo "Running tests..."

cargo test --workspace --verbose

echo "All tests passed!"
