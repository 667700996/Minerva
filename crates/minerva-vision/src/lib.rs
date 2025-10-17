//! Board recognition abstractions.

use std::{fs, path::PathBuf};

use async_trait::async_trait;
use chrono::Utc;
use image::{imageops, ImageBuffer, Rgba};
use minerva_types::{
    board::BoardState,
    config::VisionConfig,
    game::GameSnapshot,
    ui::{BOARD_FILES, BOARD_RANKS},
    vision::ImageFrame,
    MinervaError, Result,
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
    _template_dir: PathBuf,
    capture_dir: Option<PathBuf>,
    tile_capture_dir: Option<PathBuf>,
    cell_half_width: u32,
    cell_half_height: u32,
    _confidence_threshold: f32,
}

impl TemplateMatchingRecognizer {
    pub fn new(config: VisionConfig) -> Self {
        let template_dir = PathBuf::from(&config.template_dir);
        let capture_dir = config.capture_dir.as_ref().map(PathBuf::from);
        let tile_capture_dir = config.tile_capture_dir.as_ref().map(PathBuf::from);
        let (cell_half_width, cell_half_height) = compute_cell_half_sizes();

        info!(
            "Vision 템플릿 경로: {:?}, 캡처 저장: {:?}, 타일 저장: {:?}",
            template_dir, capture_dir, tile_capture_dir
        );

        Self {
            _template_dir: template_dir,
            capture_dir,
            tile_capture_dir,
            cell_half_width,
            cell_half_height,
            _confidence_threshold: config.confidence_threshold,
        }
    }

    fn persist_capture(&self, frame: &ImageFrame) -> Result<Option<PathBuf>> {
        let Some(dir) = &self.capture_dir else {
            return Ok(None);
        };
        if frame.width == 0 || frame.height == 0 {
            return Ok(None);
        }

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

    fn export_tiles(&self, frame: &ImageFrame) -> Result<()> {
        let Some(dir) = &self.tile_capture_dir else {
            return Ok(());
        };
        if frame.width == 0 || frame.height == 0 {
            return Ok(());
        }

        fs::create_dir_all(dir)
            .map_err(|err| vision_error(format!("타일 디렉터리 생성 실패({:?}): {err}", dir)))?;

        let Some(buffer) =
            ImageBuffer::<Rgba<u8>, _>::from_raw(frame.width, frame.height, frame.data.clone())
        else {
            return Err(vision_error("이미지 버퍼 생성 실패"));
        };
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S_%3f");

        for (file_idx, &cx) in BOARD_FILES.iter().enumerate() {
            for (rank_idx, &cy) in BOARD_RANKS.iter().enumerate() {
                let x0 = cx.saturating_sub(self.cell_half_width);
                let y0 = cy.saturating_sub(self.cell_half_height);

                let max_width = frame.width.saturating_sub(x0);
                let max_height = frame.height.saturating_sub(y0);
                let crop_width = (self.cell_half_width * 2).min(max_width);
                let crop_height = (self.cell_half_height * 2).min(max_height);

                if crop_width == 0 || crop_height == 0 {
                    continue;
                }

                let tile = imageops::crop_imm(&buffer, x0, y0, crop_width, crop_height).to_image();
                let filename = format!(
                    "f{}_r{}_{}.png",
                    file_idx + 1,
                    rank_idx + 1,
                    timestamp
                );
                let path = dir.join(filename);
                tile
                    .save(&path)
                    .map_err(|err| vision_error(format!("타일 저장 실패: {err}")))?;
            }
        }

        Ok(())
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
        if let Err(err) = self.export_tiles(frame) {
            tracing::warn!("타일 추출 실패: {err}");
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

fn compute_cell_half_sizes() -> (u32, u32) {
    fn average_spacing(values: &[u32]) -> f32 {
        if values.len() < 2 {
            return 1.0;
        }
        let mut total = 0f32;
        let mut count = 0f32;
        for window in values.windows(2) {
            let delta = window[1] as i32 - window[0] as i32;
            total += (delta.abs()) as f32;
            count += 1.0;
        }
        if count == 0.0 {
            1.0
        } else {
            total / count
        }
    }

    let avg_width = average_spacing(&BOARD_FILES);
    let avg_height = average_spacing(&BOARD_RANKS);
    let half_width = ((avg_width * 0.45).max(8.0)) as u32;
    let half_height = ((avg_height * 0.45).max(8.0)) as u32;
    (half_width, half_height)
}

pub fn vision_error(message: impl Into<String>) -> MinervaError {
    MinervaError::Vision(message.into())
}
