//! Search and evaluation engine abstraction.

use async_trait::async_trait;
use minerva_types::{
    game::{EngineDecision, MoveCandidate, TurnContext},
    MinervaError, Result,
};
use tokio::time::{sleep, Duration};
use tracing::info;

#[async_trait]
pub trait GameEngine: Send + Sync {
    async fn warm_up(&mut self) -> Result<()>;
    async fn evaluate_position(&self, ctx: &TurnContext) -> Result<EngineDecision>;
}

pub struct NullEngine;

impl NullEngine {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl GameEngine for NullEngine {
    async fn warm_up(&mut self) -> Result<()> {
        info!("Null engine warm-up");
        sleep(Duration::from_millis(15)).await;
        Ok(())
    }

    async fn evaluate_position(&self, ctx: &TurnContext) -> Result<EngineDecision> {
        info!(
            "Null engine evaluating turn {} for {:?}",
            ctx.snapshot.ply, ctx.side
        );
        sleep(Duration::from_millis(25)).await;
        Ok(EngineDecision {
            best_move: ctx.snapshot.last_move.clone(),
            candidates: vec![MoveCandidate {
                mv: ctx
                    .snapshot
                    .last_move
                    .clone()
                    .unwrap_or_else(|| default_hold_move()),
                score: 0.0,
                depth: 0,
            }],
            searched_nodes: 0,
            depth: 0,
            duration_ms: 25,
        })
    }
}

fn default_hold_move() -> minerva_types::game::Move {
    use minerva_types::board::Square;
    minerva_types::game::Move {
        from: Square::new(0, 0),
        to: Square::new(0, 0),
        promotion: None,
        confidence: Some(0.0),
    }
}

pub fn engine_error(message: impl Into<String>) -> MinervaError {
    MinervaError::Engine(message.into())
}
