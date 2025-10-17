use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::telemetry::{EngineMetrics, LatencySample};

/// High-level event bus message kinds moving through the system.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventKind {
    Lifecycle,
    BoardUpdate,
    EngineDecision,
    Telemetry,
    Network,
    Ops,
}

/// Immutable event envelope for logging, networking, and replay.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemEvent {
    pub id: Uuid,
    pub kind: EventKind,
    pub timestamp: DateTime<Utc>,
    pub payload: EventPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventPayload {
    Lifecycle(LifecycleEvent),
    Board(BoardEvent),
    Engine(EngineEvent),
    Telemetry(TelemetryEvent),
    Network(NetworkEvent),
    Ops(OpsEvent),
    Unknown(serde_json::Value),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleEvent {
    pub phase: LifecyclePhase,
    pub details: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum LifecyclePhase {
    Boot,
    Ready,
    MatchStart,
    MatchEnd,
    Shutdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardEvent {
    pub snapshot: crate::game::GameSnapshot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineEvent {
    pub metrics: EngineMetrics,
    pub best_line: Vec<crate::game::Move>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryEvent {
    pub latency: Option<LatencySample>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkEvent {
    pub topic: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpsEvent {
    pub message: String,
    pub tags: Vec<String>,
}

impl SystemEvent {
    pub fn new(kind: EventKind, payload: EventPayload) -> Self {
        Self {
            id: Uuid::new_v4(),
            kind,
            timestamp: Utc::now(),
            payload,
        }
    }
}
