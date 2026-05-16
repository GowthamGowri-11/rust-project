#!/bin/bash
# Benchmarking Script

API_URL="http://localhost:8080/api/v1"

echo "=========================================="
echo "RustFlow-AI Benchmarking Suite"
echo "=========================================="

# Run baseline benchmark
echo ""
echo "Running baseline benchmark..."
curl -X POST "$API_URL/benchmark/run" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "baseline",
    "duration_secs": 60,
    "num_flows": 10
  }'

sleep 5

# Run optimized benchmark
echo ""
echo "Running optimized benchmark..."
curl -X POST "$API_URL/benchmark/run" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "optimized",
    "duration_secs": 60,
    "num_flows": 10
  }'

sleep 5

# Get results
echo ""
echo "Fetching benchmark results..."
curl "$API_URL/benchmark/results" | jq '.'

echo ""
echo "=========================================="
echo "Benchmarking completed!"
echo "=========================================="
