use crate::config::AppConfig;
use controller::ControllerService;
use monitoring::MonitoringService;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub controller: Arc<ControllerService>,
    pub monitoring: Arc<MonitoringService>,
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        let controller = Arc::new(ControllerService::new(
            config.controller.host.clone(),
            config.controller.port,
        ));

        let monitoring = Arc::new(MonitoringService::new(
            config.monitoring.interval_ms,
            config.monitoring.ebpf_enabled,
        ));

        Self {
            config: Arc::new(config),
            controller,
            monitoring,
        }
    }
}
