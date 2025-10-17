use thiserror::Error;

use crate::events::EventKind;

pub type Result<T, E = MinervaError> = std::result::Result<T, E>;

/// Unified error type covering common failure scenarios across subsystems.
#[derive(Debug, Error)]
pub enum MinervaError {
    #[error("configuration error: {0}")]
    Configuration(String),
    #[error("controller error: {0}")]
    Controller(String),
    #[error("vision error: {0}")]
    Vision(String),
    #[error("engine error: {0}")]
    Engine(String),
    #[error("network error: {0}")]
    Network(String),
    #[error("orchestrator error: {0}")]
    Orchestrator(String),
    #[error("operational error: {0}")]
    Ops(String),
    #[error("invalid event stream: {0:?}")]
    Event(EventKind),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
