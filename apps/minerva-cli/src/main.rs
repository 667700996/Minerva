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
};
use minerva_vision::TemplateMatchingRecognizer;

#[tokio::main]
async fn main() -> Result<()> {
    let config = default_config();
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

fn default_config() -> MinervaConfig {
    MinervaConfig {
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
        },
    }
}
