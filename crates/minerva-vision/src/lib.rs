//! Board recognition abstractions.

use std::{fs, path::PathBuf};

use async_trait::async_trait;
use chrono::Utc;
use image::{ImageBuffer, Rgba};
use minerva_types::{
    board::BoardState, config::VisionConfig, game::GameSnapshot, vision::ImageFrame, MinervaError,
    Result,
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
pub struct TemplateMatchingRecognizer {
    capture_dir: Option<PathBuf>,
}

impl TemplateMatchingRecognizer {
    pub fn new(config: VisionConfig) -> Self {
        Self {
            capture_dir: config.capture_dir.map(PathBuf::from),
        }
    }

    fn persist_capture(&self, frame: &ImageFrame) -> Result<Option<PathBuf>> {
        let Some(dir) = &self.capture_dir else {
            return Ok(None);
        };

        fs::create_dir_all(dir)
            .map_err(|err| vision_error(format!("캡처 디렉터리 생성 실패({:?}): {err}", dir)))?;
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S_%3f");
        let path = dir.join(format!("frame_{}.png", timestamp));
        let Some(buffer) =
            ImageBuffer::<Rgba<u8>, _>::from_raw(frame.width, frame.height, frame.data.clone())
        else {
            return Err(vision_error("이미지 버퍼 생성 실패"));
        };
        buffer
            .save(&path)
            .map_err(|err| vision_error(format!("프레임 저장 실패: {err}")))?;
        Ok(Some(path))
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
        if let Ok(Some(path)) = self.persist_capture(frame) {
            info!("저장된 스크린샷: {:?}", path);
        }
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
