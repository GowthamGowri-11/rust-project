#!/bin/bash
# Chaos Engineering Test Script

API_URL="http://localhost:8080/api/v1"

echo "=========================================="
echo "RustFlow-AI Chaos Engineering Tests"
echo "=========================================="

# Test 1: Link Failure
echo ""
echo "Test 1: Simulating link failure..."
curl -X POST "$API_URL/chaos/trigger" \
  -H "Content-Type: application/json" \
  -d '{
    "scenario_type": "LinkFailure",
    "target": "link-s1-s2",
    "duration_ms": 5000
  }'

sleep 2

# Test 2: Congestion Burst
echo ""
echo "Test 2: Simulating congestion burst..."
curl -X POST "$API_URL/chaos/trigger" \
  -H "Content-Type: application/json" \
  -d '{
    "scenario_type": "CongestionBurst",
    "target": "link-s2-s3",
    "duration_ms": 3000
  }'

sleep 2

# Test 3: Switch Disconnect
echo ""
echo "Test 3: Simulating switch disconnect..."
curl -X POST "$API_URL/chaos/trigger" \
  -H "Content-Type: application/json" \
  -d '{
    "scenario_type": "SwitchDisconnect",
    "target": "switch-3",
    "duration_ms": 4000
  }'

sleep 2

# Check resilience status
echo ""
echo "Checking resilience status..."
curl "$API_URL/resilience/status" | jq '.'

echo ""
echo "=========================================="
echo "Chaos tests completed!"
echo "=========================================="
