use crate::board::Square;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

impl Point {
    pub const fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }
}

pub const START_APPLY: Point = Point::new(550, 1180);
pub const START_CONFIRM_YES: Point = Point::new(280, 710);
pub const START_CONFIRM_OK: Point = Point::new(360, 750);

pub const FORMATION_MASANG_MASANG: Point = Point::new(280, 560);
pub const FORMATION_SANG_MASANG_MA: Point = Point::new(450, 560);
pub const FORMATION_MASANG_SANG_MA: Point = Point::new(280, 620);
pub const FORMATION_SANG_MA_MA_SANG: Point = Point::new(450, 620);
pub const FORMATION_CONFIRM: Point = Point::new(450, 680);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StartFlowStep {
    Apply,
    ConfirmYes,
    ConfirmOk,
}

pub fn start_flow_point(step: StartFlowStep) -> Point {
    match step {
        StartFlowStep::Apply => START_APPLY,
        StartFlowStep::ConfirmYes => START_CONFIRM_YES,
        StartFlowStep::ConfirmOk => START_CONFIRM_OK,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FormationPreset {
    MasangMasang,
    SangMasangMa,
    MasangSangMa,
    SangMaMaSang,
}

impl Default for FormationPreset {
    fn default() -> Self {
        FormationPreset::MasangMasang
    }
}

pub fn formation_point(preset: FormationPreset) -> Point {
    match preset {
        FormationPreset::MasangMasang => FORMATION_MASANG_MASANG,
        FormationPreset::SangMasangMa => FORMATION_SANG_MASANG_MA,
        FormationPreset::MasangSangMa => FORMATION_MASANG_SANG_MA,
        FormationPreset::SangMaMaSang => FORMATION_SANG_MA_MA_SANG,
    }
}

pub const BOARD_FILES: [u32; 9] = [40, 125, 200, 280, 360, 440, 520, 600, 680];
pub const BOARD_RANKS: [u32; 10] = [880, 800, 740, 670, 600, 530, 450, 380, 300, 240];

pub fn square_to_point(square: Square) -> Option<Point> {
    let file = BOARD_FILES.get(square.file as usize)?;
    let rank = BOARD_RANKS.get(square.rank as usize)?;
    Some(Point::new(*file, *rank))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Square;

    #[test]
    fn map_square_to_point() {
        let square = Square::new(0, 0);
        let point = square_to_point(square).expect("map square");
        assert_eq!(point, Point::new(40, 880));
    }

    #[test]
    fn start_flow_points_match_constants() {
        assert_eq!(
            start_flow_point(StartFlowStep::Apply),
            Point::new(550, 1180)
        );
        assert_eq!(
            start_flow_point(StartFlowStep::ConfirmYes),
            Point::new(280, 710)
        );
        assert_eq!(
            start_flow_point(StartFlowStep::ConfirmOk),
            Point::new(360, 750)
        );
    }

    #[test]
    fn formation_points_match_constants() {
        assert_eq!(
            formation_point(FormationPreset::MasangMasang),
            Point::new(280, 560)
        );
        assert_eq!(
            formation_point(FormationPreset::SangMasangMa),
            Point::new(450, 560)
        );
        assert_eq!(
            formation_point(FormationPreset::MasangSangMa),
            Point::new(280, 620)
        );
        assert_eq!(
            formation_point(FormationPreset::SangMaMaSang),
            Point::new(450, 620)
        );
    }
}
