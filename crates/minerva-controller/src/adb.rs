use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Instant,
};

use async_trait::async_trait;
use chrono::Utc;
use image::ImageFormat;
use minerva_types::{
    board::Square, config::EmulatorConfig, telemetry::LatencySample, ui::Point, vision::ImageFrame,
    Result,
};
use tokio::{process::Command, time::Duration};

use crate::{
    controller_error, ensure_actions_present, ControllerMetrics, DeviceController, InputAction,
};

const DEFAULT_ADB: &str = "adb";

pub struct AdbController {
    config: EmulatorConfig,
    adb_path: PathBuf,
    metrics: Arc<Mutex<ControllerMetrics>>,
}

impl AdbController {
    pub fn new(config: EmulatorConfig) -> Result<Self> {
        let adb_path = config
            .adb_path
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(DEFAULT_ADB));

        Ok(Self {
            config,
            adb_path,
            metrics: Arc::new(Mutex::new(ControllerMetrics::default())),
        })
    }

    fn serial(&self) -> &str {
        if self.config.serial.is_empty() {
            "emulator-5554"
        } else {
            &self.config.serial
        }
    }

    async fn run_adb(&self, args: &[&str]) -> Result<Vec<u8>> {
        let mut command = Command::new(&self.adb_path);
        command.args(args);
        let output = command.output().await.map_err(|err| {
            controller_error(format!("ADB 명령 실행 실패({:?}): {}", args.join(" "), err))
        })?;

        if output.status.success() {
            Ok(output.stdout)
        } else {
            Err(controller_error(format!(
                "ADB 명령 실패({:?}): {}",
                args.join(" "),
                String::from_utf8_lossy(&output.stderr)
            )))
        }
    }

    async fn run_shell(&self, shell_args: &[String]) -> Result<()> {
        let mut args = vec![
            "-s".to_string(),
            self.serial().to_string(),
            "shell".to_string(),
        ];
        args.extend(shell_args.iter().cloned());
        let string_args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let output = self.run_adb(&string_args).await?;
        if !output.is_empty() {
            tracing::debug!(
                "ADB shell 출력: {}",
                String::from_utf8_lossy(&output).trim()
            );
        }
        Ok(())
    }

    async fn record_success(&self, start: Instant, injection_ms: u64) {
        if let Ok(mut guard) = self.metrics.lock() {
            guard.last_latency = Some(LatencySample {
                observation_ms: 0,
                decision_ms: 0,
                injection_ms,
                total_ms: start.elapsed().as_millis() as u64,
                captured_at: Utc::now(),
            });
            guard.successful_inputs += 1;
        }
    }

    async fn record_failure(&self) {
        if let Ok(mut guard) = self.metrics.lock() {
            guard.failed_inputs += 1;
        }
    }
}

#[async_trait]
impl DeviceController for AdbController {
    async fn connect(&mut self) -> Result<()> {
        tracing::info!("ADB 컨트롤러 연결: {}", self.serial());
        // Ensure server running
        let _ = self.run_adb(&["start-server"]).await?;
        let args = ["-s", self.serial(), "wait-for-device"];
        let _ = self.run_adb(&args).await?;
        Ok(())
    }

    async fn capture_frame(&self) -> Result<ImageFrame> {
        let args = ["-s", self.serial(), "exec-out", "screencap", "-p"];
        let raw = self.run_adb(&args).await?;
        let img = image::load_from_memory_with_format(&raw, ImageFormat::Png)
            .map_err(|err| controller_error(format!("스크린샷 디코딩 실패: {err}")))?;
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        let data = rgba.into_raw();
        Ok(ImageFrame::from_rgba(width, height, data))
    }

    async fn tap_square(&self, square: Square) -> Result<()> {
        let point = minerva_types::ui::square_to_point(square).ok_or_else(|| {
            controller_error(format!(
                "보드 좌표 범위를 벗어남: file={}, rank={}",
                square.file, square.rank
            ))
        })?;
        self.tap_point(point).await
    }

    async fn tap_point(&self, point: Point) -> Result<()> {
        self.inject_actions(vec![InputAction::Tap {
            x: point.x,
            y: point.y,
        }])
        .await
    }

    async fn inject_actions(&self, actions: Vec<InputAction>) -> Result<()> {
        ensure_actions_present(&actions)?;
        let start = Instant::now();
        for action in &actions {
            let result = match action {
                InputAction::Tap { x, y } => {
                    self.run_shell(&["input".into(), "tap".into(), x.to_string(), y.to_string()])
                        .await
                }
                InputAction::Swipe {
                    start: s,
                    end,
                    duration_ms,
                } => {
                    self.run_shell(&[
                        "input".into(),
                        "swipe".into(),
                        s.0.to_string(),
                        s.1.to_string(),
                        end.0.to_string(),
                        end.1.to_string(),
                        duration_ms.to_string(),
                    ])
                    .await
                }
                InputAction::KeyEvent { code } => {
                    self.run_shell(&["input".into(), "keyevent".into(), code.to_string()])
                        .await
                }
            };

            if let Err(err) = result {
                self.record_failure().await;
                return Err(err);
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        let injection_ms = start.elapsed().as_millis() as u64;
        self.record_success(start, injection_ms).await;
        Ok(())
    }

    fn metrics(&self) -> ControllerMetrics {
        self.metrics.lock().map(|m| m.clone()).unwrap_or_default()
    }
}
