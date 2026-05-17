# Issue Clusters - Quick Reference

**6 remaining critical issues grouped into 5 logical clusters**

---

## 📊 CLUSTER OVERVIEW

| Cluster | Issues | Status | Duration | Impact | Dependencies |
|---------|--------|--------|----------|--------|--------------|
| **1. OpenFlow** | 9 | ✅ COMPLETE | ~10h | +15pts | None |
| **2. Monitoring** | 2 | 📋 READY | 2-3w | +10pts | Cluster 1 ✅ |
| **3. ML** | 2 | 📋 BLOCKED | 1-2w | +10pts | Cluster 2 |
| **4. Resilience** | 1 | 📋 BLOCKED | 1-2w | +5pts | Cluster 2 |
| **5. Integration** | 1 | 📋 BLOCKED | 2-3w | +5pts | All above |

**Total Remaining**: 6 issues, 6-10 weeks

---

## ✅ CLUSTER 1: OPENFLOW (COMPLETE)

**What**: OpenFlow control plane reliability  
**Where**: `crates/controller/`  
**Status**: ✅ ALL 9 FIXES COMPLETE

### Issues Fixed
1. ✅ Match fields & actions encoding
2. ✅ Flow operation race condition
3. ✅ XID atomic generation
4. ✅ Partial write handling
5. ✅ Partial read handling
6. ✅ Stream lock deadlock
7. ✅ Task cancellation safety
8. ✅ Backpressure handling
9. ✅ Flow verification (barriers)

**Result**: Production-ready OpenFlow controller

---

## 📋 CLUSTER 2: MONITORING (NEXT)

**What**: Real network telemetry via eBPF  
**Where**: `crates/monitoring/`  
**Status**: 📋 READY TO START  
**Duration**: 2-3 weeks  
**Impact**: +10 points (45→55)

### Issues (2)

#### 2.1: eBPF Programs Compilation
**Problem**: eBPF programs are C strings, never compiled/loaded  
**Fix**: 
- Set up eBPF build pipeline with `aya-bpf`
- Compile programs to `.o` files
- Load into kernel at runtime
- Attach to network interfaces
- Set up event streaming

**Files**:
- `crates/monitoring/src/ebpf/programs.rs`
- `crates/monitoring/src/ebpf/manager.rs`
- `crates/monitoring/src/ebpf/probes.rs`

#### 2.2: Metric Collectors
**Problem**: All collectors return fake data  
**Fix**:
- Bandwidth: Parse `/proc/net/dev` or eBPF
- Latency: ICMP ping or eBPF timestamps
- Packet loss: eBPF drop events
- Flow stats: OpenFlow MULTIPART_REQUEST

**Files**:
- `crates/monitoring/src/collectors/bandwidth.rs`
- `crates/monitoring/src/collectors/latency.rs`
- `crates/monitoring/src/collectors/packet_loss.rs`
- `crates/monitoring/src/collectors/flow_stats.rs`

### Why Together?
- Same crate (`monitoring`)
- Both about network telemetry
- Collectors can use eBPF events
- Can work in parallel (eBPF + /proc)

### Validation
- [ ] eBPF programs load: `bpftool prog list`
- [ ] Probes attached: `bpftool prog show`
- [ ] Events flowing: Check ringbuf
- [ ] Metrics collected: Query Prometheus
- [ ] Dashboard shows real data

---

## 📋 CLUSTER 3: ML (AFTER MONITORING)

**What**: Real ML inference with ONNX  
**Where**: `crates/ml_engine/`  
**Status**: 📋 BLOCKED (needs Cluster 2 data)  
**Duration**: 1-2 weeks  
**Impact**: +10 points (55→65)

### Issues (2)

#### 3.1: ONNX Model Loading
**Problem**: `load_model()` just sets flag, doesn't load  
**Fix**:
- Add `ort` (ONNX Runtime) dependency
- Implement actual model loading
- Validate inputs/outputs
- Extract metadata
- Add error handling

**Files**:
- `crates/ml_engine/src/inference.rs`
- `crates/ml_engine/src/service.rs`

#### 3.2: Real Inference
**Problem**: `infer()` returns hardcoded values  
**Fix**:
- Implement actual ONNX inference
- Fix preprocessing (match model)
- Fix postprocessing (match model)
- Add batch inference
- Add timeout handling

**Files**:
- `crates/ml_engine/src/inference.rs`
- `crates/ml_engine/src/classifiers.rs`

### Why Together?
- Same crate (`ml_engine`)
- Both about ML inference
- Issue 3.2 depends on 3.1
- Sequential implementation

### Validation
- [ ] Model loads successfully
- [ ] Inference produces valid outputs
- [ ] Predictions change with input
- [ ] Inference latency <100ms
- [ ] Traffic classification works
- [ ] Congestion prediction works

---

## 📋 CLUSTER 4: RESILIENCE (AFTER MONITORING)

**What**: Automatic failure recovery  
**Where**: `crates/resilience/`  
**Status**: 📋 BLOCKED (needs Cluster 2)  
**Duration**: 1-2 weeks  
**Impact**: +5 points (65→70)

### Issues (1)

#### 4.1: Recovery Execution
**Problem**: `execute_recovery()` has TODOs, never executes  
**Fix**:
- Integrate with controller for flow installation
- Implement rerouting logic
- Implement failover logic
- Add recovery validation
- Add rollback on failure
- Integrate with monitoring for detection

**Files**:
- `crates/resilience/src/recovery.rs`
- `crates/resilience/src/service.rs`
- `crates/resilience/src/detection.rs`

### Why Separate?
- Single issue, focused domain
- Integrates controller (✅) + monitoring (📋)
- Can start after Cluster 2 (doesn't need ML)

### Validation
- [ ] Simulate link failure
- [ ] Traffic rerouted automatically
- [ ] Recovery time <5 seconds
- [ ] No packet loss during recovery
- [ ] Rollback works on failure

---

## 📋 CLUSTER 5: INTEGRATION (FINAL)

**What**: Connect all components  
**Where**: All crates  
**Status**: 📋 BLOCKED (needs ALL)  
**Duration**: 2-3 weeks  
**Impact**: +5 points (70→75)

### Issues (1)

#### 5.1: Component Communication
**Problem**: Components isolated, no data flow  
**Fix**:
- Implement event bus (tokio channels)
- Add workflow orchestration
- Add data pipeline
- Add integration tests
- Add end-to-end validation

**Files**: All crates

### Why Last?
- Spans all crates
- Needs all components working
- Validates entire system
- End-to-end testing

### Validation
- [ ] Event bus operational
- [ ] All components connected
- [ ] Data flows end-to-end
- [ ] Integration tests pass
- [ ] Dashboard shows live data
- [ ] System self-optimizes

---

## 🎯 IMPLEMENTATION PATHS

### Sequential (1 Engineer, 10 weeks)
```
Week 1-3:  Cluster 2 (Monitoring)
Week 4-5:  Cluster 3 (ML)
Week 6-7:  Cluster 4 (Resilience)
Week 8-10: Cluster 5 (Integration)
```

### Parallel (2 Engineers, 8 weeks)
```
Week 1-3:  Team A: Cluster 2 | Team B: Cluster 3 (mock data)
Week 4-5:  Combined: Cluster 4
Week 6-8:  Combined: Cluster 5
```

---

## 📈 PRODUCTION READINESS

```
✅ 45/100  Cluster 1 complete (OpenFlow)
   ↓
📋 55/100  After Cluster 2 (Monitoring)
   ↓
📋 65/100  After Cluster 3 (ML)
   ↓
📋 70/100  After Cluster 4 (Resilience)
   ↓
📋 75/100  After Cluster 5 (Integration)
   ↓
🎯 80+/100 Production ready
```

---

## 🔄 DEPENDENCY FLOW

```
┌─────────────┐
│  Cluster 1  │ ✅ COMPLETE
│  OpenFlow   │
└──────┬──────┘
       │
       ├──────────────────┐
       ▼                  ▼
┌─────────────┐    ┌─────────────┐
│  Cluster 2  │    │  Cluster 4  │
│ Monitoring  │───▶│ Resilience  │
└──────┬──────┘    └──────┬──────┘
       │                  │
       ▼                  │
┌─────────────┐           │
│  Cluster 3  │           │
│     ML      │           │
└──────┬──────┘           │
       │                  │
       └────────┬─────────┘
                ▼
         ┌─────────────┐
         │  Cluster 5  │
         │ Integration │
         └─────────────┘
```

---

## 💡 KEY INSIGHTS

### Why Clustering Works
1. **Focus**: One domain at a time
2. **Reuse**: Related issues share code
3. **Testing**: Validate incrementally
4. **Conflicts**: Work in isolated crates
5. **Milestones**: Measurable progress

### Critical Path
```
C1 (OpenFlow) → C2 (Monitoring) → C3 (ML) → C5 (Integration)
                      ↓
                 C4 (Resilience) ──────────┘
```

### Risk Areas
- **Cluster 2**: eBPF complexity (highest risk)
- **Cluster 3**: Needs real data from C2
- **Cluster 5**: Integration complexity

---

## 📝 NEXT STEPS

### Immediate (Start Cluster 2)
1. Set up eBPF build pipeline with `aya-bpf`
2. Create `monitoring-ebpf` crate
3. Write packet monitor eBPF program
4. Compile to `.o` file
5. Test loading into kernel

### This Week
- Complete eBPF program compilation
- Implement probe attachment
- Set up event streaming
- Start collector implementation

### This Month
- Complete Cluster 2 (Monitoring)
- Start Cluster 3 (ML)
- Begin integration planning

---

**Current Status**: ✅ Cluster 1 complete, 📋 Ready for Cluster 2  
**Next Action**: Set up eBPF build pipeline  
**Timeline**: 6-10 weeks to complete all clusters
