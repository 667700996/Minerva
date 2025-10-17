//! Board recognition abstractions.

use async_trait::async_trait;
use minerva_types::{
    board::BoardState, game::GameSnapshot, vision::ImageFrame, MinervaError, Result,
};
use tokio::time::{sleep, Duration};
use tracing::info;

/// Additional context that can guide recognition.
#[derive(Debug, Clone, Default)]
pub struct RecognitionHints {
    pub previous_snapshot: Option<GameSnapshot>,
}

#[async_trait]
pub trait BoardRecognizer: Send + Sync {
    async fn align_board(&self, frame: &ImageFrame) -> Result<BoardState>;
    async fn recognize(&self, frame: &ImageFrame, hints: RecognitionHints) -> Result<GameSnapshot>;
}

/// Simple recognizer placeholder using template matching semantics.
pub struct TemplateMatchingRecognizer;

impl TemplateMatchingRecognizer {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BoardRecognizer for TemplateMatchingRecognizer {
    async fn align_board(&self, frame: &ImageFrame) -> Result<BoardState> {
        info!(
            "Aligning board for frame {}x{} ({} bytes)",
            frame.width,
            frame.height,
            frame.data.len()
        );
        sleep(Duration::from_millis(20)).await;
        Ok(BoardState::empty())
    }

    async fn recognize(&self, frame: &ImageFrame, hints: RecognitionHints) -> Result<GameSnapshot> {
        let board = self.align_board(frame).await?;
        let snapshot = GameSnapshot {
            board,
            ..Default::default()
        };
        info!(
            "Returning mock snapshot; hints present: {}",
            hints.previous_snapshot.is_some()
        );
        Ok(snapshot)
    }
}

pub fn vision_error(message: impl Into<String>) -> MinervaError {
    MinervaError::Vision(message.into())
}
