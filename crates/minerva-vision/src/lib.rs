//! Board recognition abstractions.

use std::{collections::HashMap, fs, path::PathBuf};

use async_trait::async_trait;
use chrono::Utc;
use image::{imageops, DynamicImage, GenericImageView, ImageBuffer, Rgba};
use minerva_types::{
    board::{BoardState, Piece, PieceKind, PlayerSide, Square},
    config::VisionConfig,
    game::GameSnapshot,
    ui::{BOARD_FILES, BOARD_RANKS},
    vision::ImageFrame,
    MinervaError, Result,
};
use tokio::time::{sleep, Duration};
use tracing::{info, warn};

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
    confidence_threshold: f32,
    templates: TemplateSet,
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

        let templates = match TemplateSet::load(&template_dir) {
            Ok(set) => set,
            Err(err) => {
                warn!("템플릿 로드 실패: {err}; 인식은 빈 상태로 진행됩니다.");
                TemplateSet::default()
            }
        };

        Self {
            _template_dir: template_dir,
            capture_dir,
            tile_capture_dir,
            cell_half_width,
            cell_half_height,
            confidence_threshold: config.confidence_threshold,
            templates,
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
                let filename = format!("f{}_r{}_{}.png", file_idx + 1, rank_idx + 1, timestamp);
                let path = dir.join(filename);
                tile.save(&path)
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
        Ok(BoardState::initial())
    }

    async fn recognize(&self, frame: &ImageFrame, hints: RecognitionHints) -> Result<GameSnapshot> {
        let mut board = BoardState::empty();
        if let Some(prev) = hints.previous_snapshot.as_ref() {
            board.side_to_move = prev.board.side_to_move;
        }
        if let Ok(Some(path)) = self.persist_capture(frame) {
            info!("저장된 스크린샷: {:?}", path);
        }
        if let Err(err) = self.export_tiles(frame) {
            tracing::warn!("타일 추출 실패: {err}");
        }
        self.templates.recognize_tiles(
            frame,
            &mut board,
            self.cell_half_width,
            self.cell_half_height,
            self.confidence_threshold,
        );

        let mut snapshot = hints.previous_snapshot.clone().unwrap_or_default();
        snapshot.board = board;
        snapshot.created_at = Utc::now();
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

#[derive(Default, Clone)]
struct TemplateSet {
    templates: HashMap<String, DynamicImage>,
}

impl TemplateSet {
    fn load(dir: &PathBuf) -> Result<Self> {
        let mut templates = HashMap::new();
        if dir.is_dir() {
            for entry in fs::read_dir(dir)
                .map_err(|err| vision_error(format!("템플릿 디렉터리 읽기 실패: {err}")))?
            {
                let entry =
                    entry.map_err(|err| vision_error(format!("템플릿 파일 읽기 실패: {err}")))?;
                let path = entry.path();
                if path
                    .extension()
                    .and_then(|s| s.to_str())
                    .map_or(false, |ext| matches!(ext, "png" | "jpg" | "jpeg"))
                {
                    if let Ok(image) = image::open(&path) {
                        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                            templates.insert(stem.to_string(), image);
                        }
                    }
                }
            }
        }
        Ok(Self { templates })
    }

    fn recognize_tiles(
        &self,
        frame: &ImageFrame,
        board: &mut BoardState,
        half_w: u32,
        half_h: u32,
        confidence_threshold: f32,
    ) {
        if self.templates.is_empty() || frame.width == 0 || frame.height == 0 {
            return;
        }
        let Some(buffer) =
            ImageBuffer::<Rgba<u8>, _>::from_raw(frame.width, frame.height, frame.data.clone())
        else {
            return;
        };
        let big = DynamicImage::ImageRgba8(buffer);

        for (file_idx, &cx) in BOARD_FILES.iter().enumerate() {
            for (rank_idx, &cy) in BOARD_RANKS.iter().enumerate() {
                let sq = Square::new(file_idx as u8, rank_idx as u8);
                let tile = crop_tile(&big, cx, cy, half_w, half_h);
                if let Some((owner, kind)) =
                    classify_tile(&tile, &self.templates, confidence_threshold)
                {
                    board.set_piece(sq, Some(Piece { owner, kind }));
                }
            }
        }
    }
}

fn crop_tile(image: &DynamicImage, cx: u32, cy: u32, half_w: u32, half_h: u32) -> DynamicImage {
    let x0 = cx.saturating_sub(half_w);
    let y0 = cy.saturating_sub(half_h);
    let w = (half_w * 2).min(image.width().saturating_sub(x0));
    let h = (half_h * 2).min(image.height().saturating_sub(y0));
    let crop = imageops::crop_imm(image, x0, y0, w.max(1), h.max(1)).to_image();
    DynamicImage::ImageRgba8(crop)
}

fn classify_tile(
    tile: &DynamicImage,
    templates: &HashMap<String, DynamicImage>,
    threshold: f32,
) -> Option<(PlayerSide, PieceKind)> {
    let mut best_score = f32::MAX;
    let mut best_label: Option<&str> = None;
    for (label, template) in templates.iter() {
        let score = template_distance(tile, template);
        if score < best_score {
            best_score = score;
            best_label = Some(label);
        }
    }
    if let Some(label) = best_label {
        let normalized = best_score / 255.0;
        if normalized > threshold {
            return None;
        }
        parse_label(label)
    } else {
        None
    }
}

fn template_distance(a: &DynamicImage, b: &DynamicImage) -> f32 {
    let (aw, ah) = a.dimensions();
    let (bw, bh) = b.dimensions();
    let w = aw.min(bw);
    let h = ah.min(bh);
    if w == 0 || h == 0 {
        return f32::MAX;
    }
    let a_resized = a.resize_exact(w, h, imageops::FilterType::Nearest);
    let b_resized = b.resize_exact(w, h, imageops::FilterType::Nearest);
    let mut sum = 0f32;
    for y in 0..h {
        for x in 0..w {
            let pa = a_resized.get_pixel(x, y);
            let pb = b_resized.get_pixel(x, y);
            sum += (pa[0] as f32 - pb[0] as f32).abs();
            sum += (pa[1] as f32 - pb[1] as f32).abs();
            sum += (pa[2] as f32 - pb[2] as f32).abs();
        }
    }
    sum / (w * h * 3) as f32
}

fn parse_label(label: &str) -> Option<(PlayerSide, PieceKind)> {
    // Expected format: "blue_soldier" or "red_chariot"
    let parts: Vec<_> = label.split('_').collect();
    if parts.len() != 2 {
        return None;
    }
    let owner = match parts[0] {
        "blue" => PlayerSide::Blue,
        "red" => PlayerSide::Red,
        _ => return None,
    };
    let kind = match parts[1] {
        "general" => PieceKind::General,
        "guard" => PieceKind::Guard,
        "elephant" => PieceKind::Elephant,
        "horse" => PieceKind::Horse,
        "chariot" => PieceKind::Chariot,
        "cannon" => PieceKind::Cannon,
        "soldier" => PieceKind::Soldier,
        _ => return None,
    };
    Some((owner, kind))
}

pub fn vision_error(message: impl Into<String>) -> MinervaError {
    MinervaError::Vision(message.into())
}
