//! Emulator/ADB controller abstraction layer.

use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

use async_trait::async_trait;
use chrono::Utc;
use minerva_types::{
    board::Square, config::EmulatorConfig, telemetry::LatencySample, vision::ImageFrame,
    MinervaError, Result,
};
use tokio::time::{sleep, Duration};
use tracing::info;

/// High-level input primitives.
#[derive(Debug, Clone)]
pub enum InputAction {
    Tap {
        x: u32,
        y: u32,
    },
    Swipe {
        start: (u32, u32),
        end: (u32, u32),
        duration_ms: u64,
    },
    KeyEvent {
        code: u32,
    },
}

/// Aggregated controller performance counters.
#[derive(Debug, Default, Clone)]
pub struct ControllerMetrics {
    pub last_latency: Option<LatencySample>,
    pub successful_inputs: u64,
    pub failed_inputs: u64,
}

#[async_trait]
pub trait DeviceController: Send + Sync {
    async fn connect(&mut self) -> Result<()>;
    async fn capture_frame(&self) -> Result<ImageFrame>;
    async fn tap_square(&self, square: Square) -> Result<()>;
    async fn inject_actions(&self, actions: Vec<InputAction>) -> Result<()>;
    fn metrics(&self) -> ControllerMetrics;
}

/// Lightweight controller used for early integration and testing.
pub struct MockController {
    config: EmulatorConfig,
    metrics: Arc<Mutex<ControllerMetrics>>,
}

impl MockController {
    pub fn new(config: EmulatorConfig) -> Self {
        Self {
            config,
            metrics: Arc::new(Mutex::new(ControllerMetrics::default())),
        }
    }
}

#[async_trait]
impl DeviceController for MockController {
    async fn connect(&mut self) -> Result<()> {
        info!("Connecting to mock emulator at {}", self.config.serial);
        sleep(Duration::from_millis(50)).await;
        Ok(())
    }

    async fn capture_frame(&self) -> Result<ImageFrame> {
        info!("Capturing frame using mock controller");
        sleep(Duration::from_millis(25)).await;
        Ok(ImageFrame::empty())
    }

    async fn tap_square(&self, square: Square) -> Result<()> {
        info!("Mock tap on square ({}, {})", square.file, square.rank);
        sleep(Duration::from_millis(10)).await;
        Ok(())
    }

    async fn inject_actions(&self, actions: Vec<InputAction>) -> Result<()> {
        ensure_actions_present(&actions)?;
        let start = Instant::now();
        for action in actions {
            match action {
                InputAction::Tap { x, y } => info!("Mock tap {} {}", x, y),
                InputAction::Swipe {
                    start,
                    end,
                    duration_ms,
                } => {
                    info!(
                        "Mock swipe {:?}->{:?} duration {}ms",
                        start, end, duration_ms
                    )
                }
                InputAction::KeyEvent { code } => info!("Mock key event {}", code),
            }
            sleep(Duration::from_millis(5)).await;
        }
        let total_ms = start.elapsed().as_millis() as u64;
        let mut metrics = self
            .metrics
            .lock()
            .map_err(|_| controller_error("failed to lock metrics"))?;
        metrics.last_latency = Some(LatencySample {
            observation_ms: 0,
            decision_ms: 0,
            injection_ms: total_ms,
            total_ms,
            captured_at: Utc::now(),
        });
        metrics.successful_inputs += 1;
        Ok(())
    }

    fn metrics(&self) -> ControllerMetrics {
        self.metrics.lock().map(|m| m.clone()).unwrap_or_default()
    }
}

/// Generate an error aligned with controller semantics.
pub fn controller_error(message: impl Into<String>) -> MinervaError {
    MinervaError::Controller(message.into())
}

/// Helper to ensure there is at least one action queued.
pub fn ensure_actions_present(actions: &[InputAction]) -> Result<()> {
    if actions.is_empty() {
        Err(controller_error("no input actions specified"))
    } else {
        Ok(())
    }
}
