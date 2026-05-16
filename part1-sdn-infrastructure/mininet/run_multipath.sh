#!/bin/bash
# Start multi-path topology with RustFlow-AI controller

echo "Starting RustFlow-AI Multi-Path Topology"
echo "========================================"

# Check if controller is running
if ! nc -z localhost 6653 2>/dev/null; then
    echo "Warning: Controller not detected on port 6653"
    echo "Please start the RustFlow-AI controller first:"
    echo "  cd ../crates/controller && cargo run"
    echo ""
    read -p "Continue anyway? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Clean up any existing Mininet processes
sudo mn -c

# Start topology
echo "Starting multi-path topology..."
sudo python3 topologies/multipath_topo.py

echo "Topology stopped"
