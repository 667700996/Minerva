mod ui;

use std::{env, sync::mpsc, thread};

use anyhow::Result;
use clap::{Parser, ValueEnum};
use futures::StreamExt;
use minerva_controller::{AdbController, DeviceController, MockController};
use minerva_engine::NullEngine;
use minerva_network::{LocalServer, RealtimeServer};
use minerva_ops::TelemetryStore;
use minerva_orchestrator::{MatchRunner, Orchestrator};
use minerva_types::{
    config::{
        EmulatorConfig, EngineConfig, MinervaConfig, NetworkConfig, OpsConfig, OrchestratorConfig,
        VisionConfig,
    },
    time_control::TimeControl,
    ui::FormationPreset,
};
use minerva_vision::TemplateMatchingRecognizer;
use ui::{run as run_ui, UiMessage};

#[derive(Debug, Parser)]
#[command(name = "minerva-cli", about = "Minerva 오케스트레이션 CLI", version)]
struct CliArgs {
    /// 사용할 TOML 설정 파일 경로
    #[arg(value_name = "CONFIG")]
    config: Option<String>,

    /// 대국 턴 반복 횟수 (기본 1)
    #[arg(long, value_name = "N")]
    max_retries: Option<u8>,

    /// 시작 진형 (MasangMasang | SangMasangMa | MasangSangMa | SangMaMaSang)
    #[arg(long, value_name = "PRESET")]
    formation: Option<FormationPreset>,

    /// 컨트롤러 모드 (adb | mock)
    #[arg(long, value_enum, default_value_t = ControllerKind::Adb)]
    controller: ControllerKind,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum ControllerKind {
    Adb,
    Mock,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = CliArgs::parse();
    let mut config = load_config(args.config.as_deref());
    if let Some(max_retries) = args.max_retries {
        config.orchestrator.max_retries = max_retries;
    }
    if let Some(formation) = args.formation {
        config.orchestrator.formation = formation;
    }
    if let Err(err) = config.validate() {
        eprintln!("설정 값이 올바르지 않아 기본값으로 되돌립니다: {err}");
        config = default_config();
    }
    let config_summary = format!(
        "턴 {} | 진형 {}",
        config.orchestrator.max_retries, config.orchestrator.formation
    );
    match args.controller {
        ControllerKind::Adb => {
            let controller = AdbController::new(config.emulator.clone())?;
            run_application(controller, config, config_summary).await
        }
        ControllerKind::Mock => {
            let controller = MockController::new(config.emulator.clone());
            run_application(controller, config, config_summary).await
        }
    }
}

fn load_config(cli_path: Option<&str>) -> MinervaConfig {
    let path = cli_path
        .map(|p| p.to_string())
        .or_else(|| env::var("MINERVA_CONFIG").ok())
        .unwrap_or_else(|| "configs/dev.toml".into());

    match MinervaConfig::from_file(&path) {
        Ok(cfg) => {
            if let Err(err) = cfg.validate() {
                eprintln!(
                    "설정 파일 '{}' 검증 실패: {err}. 기본값으로 되돌립니다.",
                    path
                );
                default_config()
            } else {
                cfg
            }
        }
        Err(err) => {
            eprintln!(
                "설정 파일 '{}' 읽기 실패: {err}. 기본값으로 되돌립니다.",
                path
            );
            default_config()
        }
    }
}

fn default_config() -> MinervaConfig {
    let config = MinervaConfig {
        emulator: EmulatorConfig {
            serial: "127.0.0.1:5555".into(),
            socket: "127.0.0.1:5555".into(),
            fixed_resolution: Some((1080, 1920)),
            adb_path: None,
        },
        vision: VisionConfig {
            template_dir: "assets/templates".into(),
            confidence_threshold: 0.95,
            refresh_interval_ms: 500,
            capture_dir: Some("captures".into()),
            tile_capture_dir: Some("captures/tiles".into()),
        },
        engine: EngineConfig {
            threads: 1,
            max_depth: 1,
            nnue_path: None,
        },
        network: NetworkConfig {
            bind_addr: "127.0.0.1".into(),
            websocket_port: 3000,
            auth_token: None,
        },
        ops: OpsConfig {
            log_level: "info".into(),
            telemetry_dir: "telemetry".into(),
        },
        orchestrator: OrchestratorConfig {
            time_control: TimeControl::blitz(),
            max_retries: 1,
            formation: FormationPreset::MasangSangMa,
        },
    };
    debug_assert!(config.validate().is_ok());
    config
}

async fn run_application<C>(
    controller: C,
    config: MinervaConfig,
    config_summary: String,
) -> Result<()>
where
    C: DeviceController + Send + Sync + 'static,
{
    let recognizer = TemplateMatchingRecognizer::new(config.vision.clone());
    let engine = NullEngine::new();
    let network = LocalServer::new(64);
    let telemetry = TelemetryStore::new();

    let (ui_tx, ui_rx) = mpsc::channel::<UiMessage>();
    let ui_forward_network = network.clone();
    let ui_forward_tx = ui_tx.clone();
    let ui_forward_handle = tokio::spawn(async move {
        let mut stream = ui_forward_network.subscribe();
        while let Some(event) = stream.next().await {
            if ui_forward_tx.send(UiMessage::Event(event)).is_err() {
                break;
            }
        }
    });

    let ui_thread = thread::spawn(move || {
        if let Err(err) = run_ui(ui_rx, config_summary) {
            eprintln!("터미널 UI 오류: {err:?}");
        }
    });

    let mut orchestrator = Orchestrator::new(
        config.orchestrator.clone(),
        controller,
        recognizer,
        engine,
        network,
        telemetry,
    );

    orchestrator.boot(&config).await?;
    let run_result = orchestrator.run().await;

    let _ = ui_tx.send(UiMessage::Shutdown);
    drop(ui_tx);

    ui_forward_handle.abort();
    let _ = ui_forward_handle.await;
    let _ = ui_thread.join();

    run_result?;
    Ok(())
}
