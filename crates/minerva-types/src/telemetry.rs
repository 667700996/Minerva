use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencySample {
    pub observation_ms: u64,
    pub decision_ms: u64,
    pub injection_ms: u64,
    pub total_ms: u64,
    pub captured_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EngineMetrics {
    pub nodes: u64,
    pub depth: u8,
    pub nps: u64,
    pub hashfull: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MatchTelemetry {
    pub latency_samples: Vec<LatencySample>,
    pub engine_history: Vec<EngineMetrics>,
    pub notes: Vec<String>,
}
