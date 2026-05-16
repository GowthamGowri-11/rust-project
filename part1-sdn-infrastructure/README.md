# RustFlow-AI Part 1: Network + SDN Infrastructure

Production-grade Rust-native OpenFlow controller with Mininet integration for AI-driven SDN traffic engineering.

## 🎯 Part 1 Objectives

This part implements the complete Network + SDN Infrastructure layer:

- ✅ Rust-native OpenFlow controller
- ✅ Async networking architecture (tokio)
- ✅ Switch management and registration
- ✅ Dynamic flow rule installation
- ✅ Mininet topology setup
- ✅ Traffic generation environment
- ✅ Controller-to-switch communication

## 📁 Project Structure

```
part1-sdn-infrastructure/
├── crates/
│   ├── controller/          # OpenFlow controller
│   │   ├── src/
│   │   │   ├── main.rs             # Controller entry point
│   │   │   ├── switch_manager.rs   # Switch registration & management
│   │   │   ├── flow_manager.rs     # Flow table management
│   │   │   ├── routing.rs          # Routing engine
│   │   │   └── packet_handler.rs   # Packet-in processing
│   │   └── Cargo.toml
│   ├── network_core/        # Core networking layer
│   │   ├── src/
│   │   │   ├── connection.rs       # Async connection manager
│   │   │   ├── dispatcher.rs       # Event dispatcher
│   │   │   └── topology_manager.rs # Topology management
│   │   └── Cargo.toml
│   └── shared/              # Shared types and utilities
│       ├── src/
│       │   ├── types.rs            # Common data structures
│       │   ├── messages.rs         # OpenFlow messages
│       │   └── error.rs            # Error types
│       └── Cargo.toml
├── mininet/
│   ├── topologies/
│   │   ├── multipath_topo.py       # Multi-path topology
│   │   └── congestion_topo.py      # Congestion-prone topology
│   ├── traffic/
│   │   └── traffic_generator.py    # Traffic generation
│   └── run_multipath.sh            # Quick start script
├── docker/
│   ├── Dockerfile.controller       # Controller container
│   └── Dockerfile.mininet          # Mininet container
├── docker-compose.yml
└── README.md
```

## 🚀 Quick Start

### Option 1: Local Development

#### Prerequisites
- Rust 1.75+
- Mininet (Linux only)
- Python 3.8+

#### 1. Start the Controller

```bash
cd crates/controller
cargo run --release
```

The controller will listen on `0.0.0.0:6653` for OpenFlow connections.

#### 2. Start Mininet Topology

In a new terminal:

```bash
cd mininet
sudo ./run_multipath.sh
```

Or manually:

```bash
sudo python3 topologies/multipath_topo.py
```

#### 3. Generate Traffic

In the Mininet CLI:

```bash
# Load traffic generator
mininet> py exec(open('traffic/traffic_generator.py').read())
mininet> py traffic_gen = TrafficGenerator(net)

# Generate video traffic
mininet> py traffic_gen.generate_video_traffic('h1', 'h3', 60, '5M')

# Generate bulk download
mininet> py traffic_gen.generate_bulk_download('h2', 'h4', 60)

# Generate mixed traffic
mininet> py traffic_gen.generate_mixed_traffic(60)
```

### Option 2: Docker Deployment

```bash
# Build and start all services
docker-compose up -d

# View controller logs
docker-compose logs -f controller

# Access Mininet container
docker exec -it rustflow-mininet bash

# Inside Mininet container
cd /mininet
python3 topologies/multipath_topo.py
```

## 🏗️ Architecture

### Controller Components

#### 1. **SwitchManager**
- Registers and tracks connected switches
- Maintains switch state and capabilities
- Handles switch connection/disconnection

#### 2. **FlowManager**
- Installs and removes flow rules
- Tracks flow table state per switch
- Generates FlowMod messages

#### 3. **RoutingEngine**
- Computes shortest paths using BFS
- Generates flow rules for paths
- Supports multi-path routing

#### 4. **PacketHandler**
- Processes Packet-In messages
- Implements L2 learning switch logic
- Parses Ethernet frames

### Network Core

#### 1. **ConnectionManager**
- Async TCP listener for OpenFlow connections
- Per-switch connection handling
- Message send/receive queues

#### 2. **EventDispatcher**
- Dispatches OpenFlow events to handlers
- Async event processing
- Message type routing

#### 3. **TopologyManager**
- Maintains network topology graph
- Tracks switches and links
- Provides neighbor discovery

## 📊 Topologies

### Multi-Path Topology

6 switches, 4 hosts with multiple paths for load balancing:

```
   h1 --- s1 --- s2 --- s5 --- h3
           |  X  |      |
           | / \ |      |
           s3 - s4 --- s6 --- h4
           |
          h2
```

**Features:**
- Multiple paths between hosts
- 50 Mbps core links
- 100 Mbps edge links
- 10ms core latency

### Congestion Topology

Intentional bottlenecks for testing traffic engineering:

```
h1 ---\           /--- h5
h2 --- s1 - s3 - s5 --- h6
h3 ---/     |     \--- h7
h4 -------- s4 -------- h8
```

**Features:**
- 10 Mbps bottleneck links (s1-s3, s3-s5)
- 100 Mbps edge links
- Alternative 50 Mbps paths
- Congestion-prone design

## 🔧 Configuration

### Controller Settings

Edit environment variables or `.env` file:

```bash
RUST_LOG=info,controller=debug
CONTROLLER_HOST=0.0.0.0
CONTROLLER_PORT=6653
```

### Mininet Settings

Modify topology scripts to adjust:
- Link bandwidth: `bw=100` (Mbps)
- Link delay: `delay='10ms'`
- Packet loss: `loss=0.1` (10%)

## 🧪 Testing

### Build and Test

```bash
# Build all crates
cargo build --workspace

# Run tests
cargo test --workspace

# Run with debug logging
RUST_LOG=debug cargo run --bin controller
```

### Verify Controller

```bash
# Check if controller is listening
nc -zv localhost 6653

# View controller logs
tail -f logs/controller.log
```

### Test Connectivity

In Mininet CLI:

```bash
# Ping all hosts
mininet> pingall

# Ping specific hosts
mininet> h1 ping -c 3 h3

# Check switch connections
mininet> net

# Dump flows
mininet> dpctl dump-flows
```

## 📈 Traffic Generation

### Video Traffic (UDP)
```python
traffic_gen.generate_video_traffic('h1', 'h3', duration=60, rate='5M')
```

### Bulk Download (TCP)
```python
traffic_gen.generate_bulk_download('h2', 'h4', duration=60)
```

### Mixed Traffic
```python
traffic_gen.generate_mixed_traffic(duration=60)
```

## 🔍 Monitoring

### Controller Metrics

The controller logs:
- Switch connections/disconnections
- Flow installations
- Packet-In events
- Topology changes

### Network Metrics

Use iperf logs:

```bash
# View traffic logs
cat /tmp/h1_video.log
cat /tmp/h2_bulk.log
```

## 🐛 Troubleshooting

### Controller won't start
```bash
# Check if port is in use
sudo lsof -i :6653

# Kill existing process
sudo kill -9 <PID>
```

### Mininet connection fails
```bash
# Clean up Mininet
sudo mn -c

# Restart OVS
sudo service openvswitch-switch restart
```

### Switches not connecting
```bash
# Check controller IP in topology script
# Should be '127.0.0.1' for local or 'controller' for Docker

# Verify firewall
sudo ufw allow 6653
```

## 📚 API Reference

### SwitchManager

```rust
pub fn register_switch(&self, switch: SwitchInfo)
pub fn unregister_switch(&self, switch_id: SwitchId)
pub fn get_switch(&self, switch_id: SwitchId) -> Option<SwitchInfo>
pub fn get_all_switches(&self) -> Vec<SwitchInfo>
```

### FlowManager

```rust
pub fn install_flow(&self, rule: FlowRule) -> Result<FlowId>
pub fn remove_flow(&self, flow_id: FlowId) -> Result<()>
pub fn get_switch_flows(&self, switch_id: SwitchId) -> Vec<FlowRule>
```

### RoutingEngine

```rust
pub fn find_shortest_path(&self, src: SwitchId, dst: SwitchId) -> Option<Vec<SwitchId>>
pub fn compute_flow_rules(&self, path: &[SwitchId]) -> Vec<FlowRule>
```

## 🎯 Next Steps (Part 2)

Part 2 will add:
- ML-based traffic prediction
- Congestion detection
- Dynamic path optimization
- ONNX model integration
- Real-time analytics

## 📝 License

MIT License - see LICENSE file

## 🤝 Contributing

See CONTRIBUTING.md for development guidelines

---

**Status:** ✅ Part 1 Complete - Network + SDN Infrastructure Ready
