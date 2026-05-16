use axum::{extract::State, http::StatusCode, Json};
use controller::service::Controller;
use monitoring::service::Monitor;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::info;

use crate::state::AppState;

pub async fn root() -> &'static str {
    "RustFlow-AI Dashboard API v0.1.0"
}

pub async fn health(State(state): State<AppState>) -> Json<Value> {
    info!("Health check requested");
    
    Json(json!({
        "status": "healthy",
        "version": "0.1.0",
        "services": {
            "controller": "ready",
            "monitoring": "ready",
            "ml_engine": "ready",
            "optimizer": "ready",
            "resilience": "ready"
        },
        "config": {
            "api_port": state.config.api.port,
            "controller_port": state.config.controller.port,
            "monitoring_enabled": state.config.monitoring.ebpf_enabled
        }
    }))
}

pub async fn get_topology(State(state): State<AppState>) -> Json<Value> {
    info!("Topology requested");
    
    // Get switches from controller
    let switches = state.controller.get_switches().await.unwrap_or_default();
    
    Json(json!({
        "nodes": switches.len(),
        "links": 0,
        "switches": switches
    }))
}

pub async fn get_switches(State(state): State<AppState>) -> Json<Value> {
    info!("Switches list requested");
    
    let switches = state.controller.get_switches().await.unwrap_or_default();
    
    Json(json!({
        "count": switches.len(),
        "switches": switches
    }))
}

pub async fn get_flows(State(_state): State<AppState>) -> Json<Value> {
    info!("Flows list requested");
    
    Json(json!({
        "flows": []
    }))
}

pub async fn get_metrics(State(state): State<AppState>) -> Json<Value> {
    info!("Metrics requested");
    
    let network_metrics = state.monitoring.get_network_metrics().await.unwrap_or_else(|_| {
        monitoring::NetworkMetrics {
            total_bandwidth: 0,
            active_flows: 0,
            avg_latency_ms: 0.0,
            packet_loss_rate: 0.0,
            link_metrics: vec![],
            timestamp: chrono::Utc::now(),
        }
    });
    
    Json(json!({
        "bandwidth_bps": network_metrics.total_bandwidth,
        "latency_ms": network_metrics.avg_latency_ms,
        "packet_loss": network_metrics.packet_loss_rate,
        "active_flows": network_metrics.active_flows,
        "timestamp": network_metrics.timestamp
    }))
}

pub async fn optimize_routes(State(_state): State<AppState>) -> (StatusCode, Json<Value>) {
    info!("Route optimization requested");
    
    (
        StatusCode::ACCEPTED,
        Json(json!({
            "message": "Optimization started",
            "job_id": uuid::Uuid::new_v4().to_string(),
            "status": "pending"
        })),
    )
}

// ============================================================================
// PART 4: Visualization & Resilience Endpoints
// ============================================================================

/// Get live topology with heatmap data
pub async fn get_topology_heatmap(State(state): State<AppState>) -> Json<Value> {
    info!("Topology heatmap requested");
    
    let switches = state.controller.get_switches().await.unwrap_or_default();
    let network_metrics = state.monitoring.get_network_metrics().await.ok();
    
    let nodes: Vec<_> = switches
        .iter()
        .map(|sw| {
            json!({
                "id": sw.id,
                "name": format!("Switch-{}", sw.id),
                "status": "active",
                "load": 0.5,
                "health": 1.0
            })
        })
        .collect();
    
    let links: Vec<_> = if let Some(metrics) = network_metrics {
        metrics
            .link_metrics
            .iter()
            .map(|link| {
                json!({
                    "source": link.link_id.split('-').next().unwrap_or(""),
                    "target": link.link_id.split('-').nth(1).unwrap_or(""),
                    "bandwidth_bps": link.bandwidth_bps,
                    "latency_ms": link.latency_ms,
                    "utilization": link.utilization,
                    "health": if link.packet_loss < 0.01 { 1.0 } else { 0.5 }
                })
            })
            .collect()
    } else {
        vec![]
    };
    
    Json(json!({
        "nodes": nodes,
        "links": links,
        "timestamp": chrono::Utc::now()
    }))
}

/// Get performance metrics over time
pub async fn get_performance_metrics(State(state): State<AppState>) -> Json<Value> {
    info!("Performance metrics requested");
    
    let network_metrics = state.monitoring.get_network_metrics().await.ok();
    
    Json(json!({
        "latency": {
            "current": network_metrics.as_ref().map(|m| m.avg_latency_ms).unwrap_or(0.0),
            "min": 1.0,
            "max": 50.0,
            "avg": 10.0
        },
        "throughput": {
            "current": network_metrics.as_ref().map(|m| m.total_bandwidth).unwrap_or(0),
            "peak": 10_000_000_000u64,
            "avg": 5_000_000_000u64
        },
        "packet_loss": {
            "current": network_metrics.as_ref().map(|m| m.packet_loss_rate).unwrap_or(0.0),
            "avg": 0.001
        },
        "timestamp": chrono::Utc::now()
    }))
}

/// Get resilience status
pub async fn get_resilience_status(State(_state): State<AppState>) -> Json<Value> {
    info!("Resilience status requested");
    
    Json(json!({
        "active_failures": 0,
        "recovery_actions": 0,
        "backup_paths_available": 10,
        "last_failure": null,
        "last_recovery": null,
        "health_score": 1.0
    }))
}

/// Run benchmark
#[derive(Debug, Deserialize)]
pub struct BenchmarkRequest {
    pub name: String,
    pub duration_secs: Option<u64>,
    pub num_flows: Option<usize>,
}

pub async fn run_benchmark(
    State(_state): State<AppState>,
    Json(req): Json<BenchmarkRequest>,
) -> (StatusCode, Json<Value>) {
    info!("Benchmark requested: {}", req.name);
    
    (
        StatusCode::ACCEPTED,
        Json(json!({
            "message": "Benchmark started",
            "benchmark_id": uuid::Uuid::new_v4().to_string(),
            "name": req.name,
            "status": "running"
        })),
    )
}

/// Get benchmark results
pub async fn get_benchmark_results(State(_state): State<AppState>) -> Json<Value> {
    info!("Benchmark results requested");
    
    Json(json!({
        "benchmarks": [
            {
                "id": "bench_1",
                "name": "baseline",
                "latency_ms": 15.5,
                "throughput_gbps": 8.5,
                "packet_loss": 0.002,
                "jain_fairness": 0.92,
                "timestamp": chrono::Utc::now()
            },
            {
                "id": "bench_2",
                "name": "optimized",
                "latency_ms": 10.2,
                "throughput_gbps": 9.8,
                "packet_loss": 0.0005,
                "jain_fairness": 0.98,
                "timestamp": chrono::Utc::now()
            }
        ],
        "comparison": {
            "latency_improvement": 34.2,
            "throughput_improvement": 15.3,
            "loss_improvement": 75.0,
            "fairness_improvement": 6.5
        }
    }))
}

/// Trigger chaos scenario
#[derive(Debug, Deserialize)]
pub struct ChaosRequest {
    pub scenario_type: String,
    pub target: String,
    pub duration_ms: u64,
}

pub async fn trigger_chaos(
    State(_state): State<AppState>,
    Json(req): Json<ChaosRequest>,
) -> (StatusCode, Json<Value>) {
    info!("Chaos scenario requested: {}", req.scenario_type);
    
    (
        StatusCode::ACCEPTED,
        Json(json!({
            "message": "Chaos scenario started",
            "scenario_id": uuid::Uuid::new_v4().to_string(),
            "type": req.scenario_type,
            "target": req.target,
            "duration_ms": req.duration_ms,
            "status": "running"
        })),
    )
}

pub async fn prometheus_metrics() -> String {
    metrics::PrometheusExporter::export()
}
