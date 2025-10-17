use crate::board::Square;

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
}
