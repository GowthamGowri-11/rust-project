use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod handlers;
mod state;

use config::AppConfig;
use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,dashboard_api=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting RustFlow-AI Dashboard API");

    // Load configuration
    let config = AppConfig::load_or_default();
    info!("Configuration loaded: API will listen on {}:{}", config.api.host, config.api.port);

    // Initialize metrics
    metrics::PrometheusExporter::init();

    // Initialize application state with services
    let state = AppState::new(config.clone());

    let app = Router::new()
        .route("/", get(handlers::root))
        .route("/api/v1/health", get(handlers::health))
        .route("/api/v1/topology", get(handlers::get_topology))
        .route("/api/v1/switches", get(handlers::get_switches))
        .route("/api/v1/flows", get(handlers::get_flows))
        .route("/api/v1/metrics", get(handlers::get_metrics))
        .route("/api/v1/routes/optimize", post(handlers::optimize_routes))
        // Part 4: Visualization & Resilience endpoints
        .route("/api/v1/topology/heatmap", get(handlers::get_topology_heatmap))
        .route("/api/v1/performance", get(handlers::get_performance_metrics))
        .route("/api/v1/resilience/status", get(handlers::get_resilience_status))
        .route("/api/v1/benchmark/run", post(handlers::run_benchmark))
        .route("/api/v1/benchmark/results", get(handlers::get_benchmark_results))
        .route("/api/v1/chaos/trigger", post(handlers::trigger_chaos))
        .route("/metrics", get(handlers::prometheus_metrics))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = SocketAddr::from((
        config.api.host.parse::<std::net::IpAddr>().unwrap_or([0, 0, 0, 0].into()),
        config.api.port,
    ));
    info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
