use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TimeControlMode {
    Blitz,
    Rapid,
    Classic,
    Custom,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TimeControl {
    pub mode: TimeControlMode,
    pub base_ms: u64,
    pub increment_ms: u64,
    pub max_depth_hint: Option<u8>,
}

impl TimeControl {
    pub fn blitz() -> Self {
        Self {
            mode: TimeControlMode::Blitz,
            base_ms: 10 * 60 * 1000,
            increment_ms: 0,
            max_depth_hint: Some(10),
        }
    }
}
