use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

use crate::{MinervaError, Result};

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

impl MinervaConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_ref = path.as_ref();
        let contents = fs::read_to_string(path_ref).map_err(|err| {
            MinervaError::Configuration(format!(
                "unable to read config file {}: {err}",
                path_ref.display()
            ))
        })?;
        toml::from_str(&contents).map_err(|err| {
            MinervaError::Configuration(format!(
                "failed to parse config file {}: {err}",
                path_ref.display()
            ))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time_control::TimeControlMode;
    use std::fs;

    #[test]
    fn load_minerva_config_from_file() {
        let temp_path = std::env::temp_dir().join("minerva-config-test.toml");
        let config = MinervaConfig {
            emulator: EmulatorConfig {
                serial: "127.0.0.1:5555".into(),
                socket: "127.0.0.1:5555".into(),
                fixed_resolution: Some((1080, 1920)),
                adb_path: None,
            },
            vision: VisionConfig {
                template_dir: "templates".into(),
                confidence_threshold: 0.9,
                refresh_interval_ms: 250,
            },
            engine: EngineConfig {
                threads: 2,
                max_depth: 4,
                nnue_path: None,
            },
            network: NetworkConfig {
                bind_addr: "0.0.0.0".into(),
                websocket_port: 3100,
                auth_token: Some("token".into()),
            },
            ops: OpsConfig {
                log_level: "debug".into(),
                telemetry_dir: "telemetry".into(),
            },
            orchestrator: OrchestratorConfig {
                time_control: TimeControl {
                    mode: TimeControlMode::Rapid,
                    base_ms: 15 * 60 * 1000,
                    increment_ms: 5_000,
                    max_depth_hint: Some(12),
                },
                max_retries: 2,
            },
        };

        let doc = toml::to_string(&config).expect("serialize config");
        fs::write(&temp_path, doc).expect("write temp config");

        let loaded = MinervaConfig::from_file(&temp_path).expect("load config");
        assert_eq!(loaded.engine.max_depth, config.engine.max_depth);
        assert_eq!(
            loaded.orchestrator.max_retries,
            config.orchestrator.max_retries
        );
        fs::remove_file(&temp_path).expect("cleanup temp config");
    }
}
