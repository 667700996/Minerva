use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

use crate::{MinervaError, Result};

use crate::{time_control::TimeControl, ui::FormationPreset};

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
    #[serde(default)]
    pub formation: FormationPreset,
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

    pub fn validate(&self) -> Result<()> {
        if self.engine.threads == 0 {
            return Err(MinervaError::Configuration(
                "engine.threads must be greater than zero".into(),
            ));
        }
        if self.engine.max_depth == 0 {
            return Err(MinervaError::Configuration(
                "engine.max_depth must be greater than zero".into(),
            ));
        }
        if !(0.0..=1.0).contains(&self.vision.confidence_threshold) {
            return Err(MinervaError::Configuration(
                "vision.confidence_threshold must be between 0.0 and 1.0".into(),
            ));
        }
        if self.network.websocket_port == 0 {
            return Err(MinervaError::Configuration(
                "network.websocket_port must be a valid port (>0)".into(),
            ));
        }
        if self.orchestrator.max_retries == 0 {
            return Err(MinervaError::Configuration(
                "orchestrator.max_retries must be greater than zero".into(),
            ));
        }
        Ok(())
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
                formation: FormationPreset::SangMasangMa,
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
        assert_eq!(loaded.orchestrator.formation, config.orchestrator.formation);
        fs::remove_file(&temp_path).expect("cleanup temp config");
    }

    #[test]
    fn validate_configuration_rules() {
        let mut config = MinervaConfig {
            emulator: EmulatorConfig {
                serial: "device".into(),
                socket: "device".into(),
                fixed_resolution: None,
                adb_path: None,
            },
            vision: VisionConfig {
                template_dir: "templates".into(),
                confidence_threshold: 0.5,
                refresh_interval_ms: 250,
            },
            engine: EngineConfig {
                threads: 0,
                max_depth: 1,
                nnue_path: None,
            },
            network: NetworkConfig {
                bind_addr: "0.0.0.0".into(),
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
                formation: FormationPreset::default(),
            },
        };

        assert!(config.validate().is_err());
        config.engine.threads = 2;
        config.engine.max_depth = 0;
        assert!(config.validate().is_err());
        config.engine.max_depth = 4;
        config.vision.confidence_threshold = 1.5;
        assert!(config.validate().is_err());
        config.vision.confidence_threshold = 0.9;
        config.network.websocket_port = 0;
        assert!(config.validate().is_err());
        config.network.websocket_port = 3000;
        config.orchestrator.max_retries = 0;
        assert!(config.validate().is_err());
        config.orchestrator.max_retries = 1;
        assert!(config.validate().is_ok());
    }
}
