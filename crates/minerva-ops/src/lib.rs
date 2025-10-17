//! Operational helpers: logging, telemetry persistence, replay support.

use std::{path::PathBuf, sync::Arc};

use minerva_types::{
    config::OpsConfig, events::SystemEvent, telemetry::MatchTelemetry, MinervaError, Result,
};
use tokio::sync::Mutex;
use tracing::info;
use tracing_subscriber::{fmt, EnvFilter};

pub fn init_tracing(config: &OpsConfig) -> Result<()> {
    let filter = EnvFilter::try_new(config.log_level.clone())
        .or_else(|_| EnvFilter::try_new("info"))
        .map_err(|err| MinervaError::Ops(format!("failed to create log filter: {err}")))?;

    fmt()
        .with_env_filter(filter)
        .try_init()
        .map_err(|err| MinervaError::Ops(format!("tracing init error: {err}")))?;
    Ok(())
}

/// In-memory telemetry store for early development.
#[derive(Clone, Default)]
pub struct TelemetryStore {
    events: Arc<Mutex<Vec<SystemEvent>>>,
    matches: Arc<Mutex<Vec<MatchTelemetry>>>,
}

impl TelemetryStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn record_event(&self, event: SystemEvent) -> Result<()> {
        self.events.lock().await.push(event);
        Ok(())
    }

    pub async fn record_match(&self, telemetry: MatchTelemetry) -> Result<()> {
        self.matches.lock().await.push(telemetry);
        Ok(())
    }

    pub async fn snapshot_events(&self) -> Vec<SystemEvent> {
        self.events.lock().await.clone()
    }
}

pub fn ensure_telemetry_dir(path: &str) -> Result<PathBuf> {
    let dir = PathBuf::from(path);
    std::fs::create_dir_all(&dir)
        .map_err(|err| MinervaError::Ops(format!("failed to create telemetry dir: {err}")))?;
    info!("Telemetry directory ready at {:?}", dir);
    Ok(dir)
}
