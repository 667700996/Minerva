use serde::{Deserialize, Serialize};

use crate::time_control::TimeControl;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmulatorConfig {
    pub serial: String,
    pub socket: String,
    pub fixed_resolution: Option<(u32, u32)>,
    pub adb_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionConfig {
    pub template_dir: String,
    pub confidence_threshold: f32,
    pub refresh_interval_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    pub threads: usize,
    pub max_depth: u8,
    pub nnue_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub bind_addr: String,
    pub websocket_port: u16,
    pub auth_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpsConfig {
    pub log_level: String,
    pub telemetry_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorConfig {
    pub time_control: TimeControl,
    pub max_retries: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinervaConfig {
    pub emulator: EmulatorConfig,
    pub vision: VisionConfig,
    pub engine: EngineConfig,
    pub network: NetworkConfig,
    pub ops: OpsConfig,
    pub orchestrator: OrchestratorConfig,
}
