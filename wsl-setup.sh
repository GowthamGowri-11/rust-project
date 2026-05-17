#!/bin/bash
# WSL2 setup script for Rust development

# Install Rust in WSL2
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env

# Install dependencies
sudo apt-get update
sudo apt-get install -y build-essential pkg-config libssl-dev

# Build project
cd /mnt/c/Users/GOWTHAMGOWRI/Desktop/rsut-project
cargo build --release
cargo test --all

echo "Setup complete! Use WSL2 for all Rust development."
