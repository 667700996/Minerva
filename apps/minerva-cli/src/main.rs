use std::env;

use anyhow::Result;
use minerva_controller::MockController;
use minerva_engine::NullEngine;
use minerva_network::LocalServer;
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

#[tokio::main]
async fn main() -> Result<()> {
    let config = load_config();
    let controller = MockController::new(config.emulator.clone());
    let recognizer = TemplateMatchingRecognizer::new();
    let engine = NullEngine::new();
    let network = LocalServer::new(64);
    let telemetry = TelemetryStore::new();

    let mut orchestrator = Orchestrator::new(
        config.orchestrator.clone(),
        controller,
        recognizer,
        engine,
        network,
        telemetry,
    );

    orchestrator.boot(&config).await?;
    orchestrator.run().await?;
    Ok(())
}

fn load_config() -> MinervaConfig {
    let from_env = env::var("MINERVA_CONFIG").ok();
    let from_args = env::args().nth(1);
    let path = from_args
        .or(from_env)
        .unwrap_or_else(|| "configs/dev.toml".into());
    match MinervaConfig::from_file(&path) {
        Ok(cfg) => {
            if let Err(err) = cfg.validate() {
                eprintln!(
                    "Invalid config in '{}': {err}. Falling back to internal defaults.",
                    path
                );
                default_config()
            } else {
                cfg
            }
        }
        Err(err) => {
            eprintln!(
                "Failed to load config from '{}': {err}. Falling back to internal defaults.",
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
            max_retries: 3,
            formation: FormationPreset::MasangMasang,
        },
    };
    debug_assert!(config.validate().is_ok());
    config
}
