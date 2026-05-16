#!/bin/bash
# Test script for controller functionality

echo "Testing RustFlow-AI Controller"
echo "==============================="
echo ""

# Check if controller is running
echo -n "Checking if controller is running on port 6653... "
if nc -z localhost 6653 2>/dev/null; then
    echo "✓ Controller is running"
else
    echo "✗ Controller is not running"
    echo ""
    echo "Please start the controller first:"
    echo "  cargo run --bin controller"
    exit 1
fi

echo ""
echo "Controller is ready for OpenFlow connections!"
echo ""
echo "To test with Mininet:"
echo "  cd mininet"
echo "  sudo python3 topologies/multipath_topo.py"
