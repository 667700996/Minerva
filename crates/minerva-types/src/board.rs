use serde::{Deserialize, Serialize};

/// Represents the two players in a Janggi game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayerSide {
    Blue,
    Red,
}

impl PlayerSide {
    pub fn opponent(self) -> Self {
        match self {
            PlayerSide::Blue => PlayerSide::Red,
            PlayerSide::Red => PlayerSide::Blue,
        }
    }
}

/// Piece kind in Korean Janggi.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PieceKind {
    General,
    Guard,
    Elephant,
    Horse,
    Chariot,
    Cannon,
    Soldier,
}

/// Lightweight board coordinate (0-indexed).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Square {
    pub file: u8,
    pub rank: u8,
}

impl Square {
    pub fn new(file: u8, rank: u8) -> Self {
        Self { file, rank }
    }

    pub fn offset(&self, df: i8, dr: i8) -> Option<Square> {
        let nf = self.file as i16 + df as i16;
        let nr = self.rank as i16 + dr as i16;
        if nf >= 0
            && nr >= 0
            && nf < BoardState::DEFAULT_WIDTH as i16
            && nr < BoardState::DEFAULT_HEIGHT as i16
        {
            Some(Square::new(nf as u8, nr as u8))
        } else {
            None
        }
    }
}

/// Piece with its owner and optional promotion metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Piece {
    pub owner: PlayerSide,
    pub kind: PieceKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoardDiff {
    pub square: Square,
    pub before: Option<Piece>,
    pub after: Option<Piece>,
}

/// Canonical board layout representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardState {
    pub side_to_move: PlayerSide,
    pub pieces: Vec<Option<Piece>>,
    pub width: u8,
    pub height: u8,
}

impl BoardState {
    pub const DEFAULT_WIDTH: u8 = 9;
    pub const DEFAULT_HEIGHT: u8 = 10;

    pub fn empty() -> Self {
        Self {
            side_to_move: PlayerSide::Blue,
            pieces: vec![None; (Self::DEFAULT_WIDTH as usize) * (Self::DEFAULT_HEIGHT as usize)],
            width: Self::DEFAULT_WIDTH,
            height: Self::DEFAULT_HEIGHT,
        }
    }

    pub fn initial() -> Self {
        let mut board = Self::empty();
        board.setup_initial_positions();
        board
    }

    pub fn index(&self, square: Square) -> Option<usize> {
        if square.file < self.width && square.rank < self.height {
            Some((square.rank as usize) * (self.width as usize) + square.file as usize)
        } else {
            None
        }
    }

    pub fn piece_at(&self, square: Square) -> Option<Piece> {
        self.index(square)
            .and_then(|idx| self.pieces.get(idx).copied().flatten())
    }

    pub fn set_piece(&mut self, square: Square, piece: Option<Piece>) -> bool {
        if let Some(idx) = self.index(square) {
            if let Some(slot) = self.pieces.get_mut(idx) {
                *slot = piece;
                return true;
            }
        }
        false
    }

    pub fn move_piece(&mut self, from: Square, to: Square) -> Result<Option<Piece>, String> {
        let moving = self
            .piece_at(from)
            .ok_or_else(|| format!("원점에 기물이 없습니다: ({},{})", from.file, from.rank))?;
        let captured = self.piece_at(to);
        if !self.set_piece(to, Some(moving)) {
            return Err(format!(
                "목표 좌표가 유효하지 않습니다: ({},{})",
                to.file, to.rank
            ));
        }
        self.set_piece(from, None);
        Ok(captured)
    }

    pub fn is_empty(&self, square: Square) -> bool {
        self.piece_at(square).is_none()
    }

    pub fn differences(&self, other: &BoardState) -> Vec<BoardDiff> {
        let mut diffs = Vec::new();
        let width = self.width.min(other.width);
        let height = self.height.min(other.height);
        for rank in 0..height {
            for file in 0..width {
                let square = Square::new(file, rank);
                let before = self.piece_at(square);
                let after = other.piece_at(square);
                if before != after {
                    diffs.push(BoardDiff {
                        square,
                        before,
                        after,
                    });
                }
            }
        }
        diffs
    }

    pub fn infer_move_from_diffs(
        diffs: &[BoardDiff],
    ) -> Option<(Square, Square, Piece, Option<Piece>)> {
        let mut from_square = None;
        let mut to_square = None;
        let mut moving_piece = None;
        let mut captured_piece = None;

        for diff in diffs {
            match (diff.before, diff.after) {
                (Some(before), None) => {
                    from_square = Some(diff.square);
                    moving_piece = Some(before);
                }
                (None, Some(after)) => {
                    to_square = Some(diff.square);
                    moving_piece = Some(after);
                }
                (Some(before), Some(after)) if before != after => {
                    to_square = Some(diff.square);
                    moving_piece = Some(after);
                    captured_piece = Some(before);
                }
                _ => {}
            }
        }

        if let (Some(from), Some(to), Some(piece)) = (from_square, to_square, moving_piece) {
            Some((from, to, piece, captured_piece))
        } else {
            None
        }
    }

    fn setup_initial_positions(&mut self) {
        use PieceKind::*;

        let back_rank = [
            Chariot, Horse, Elephant, Guard, General, Guard, Elephant, Horse, Chariot,
        ];

        for (file, kind) in back_rank.iter().enumerate() {
            self.set_piece(
                Square::new(file as u8, 0),
                Some(Piece {
                    owner: PlayerSide::Blue,
                    kind: *kind,
                }),
            );
            self.set_piece(
                Square::new(file as u8, self.height - 1),
                Some(Piece {
                    owner: PlayerSide::Red,
                    kind: *kind,
                }),
            );
        }

        // Guards occupy palace centers (already set above).

        // Cannons
        let cannon_files = [1u8, 7u8];
        for &file in &cannon_files {
            self.set_piece(
                Square::new(file, 2),
                Some(Piece {
                    owner: PlayerSide::Blue,
                    kind: Cannon,
                }),
            );
            self.set_piece(
                Square::new(file, self.height - 3),
                Some(Piece {
                    owner: PlayerSide::Red,
                    kind: Cannon,
                }),
            );
        }

        // Soldiers
        let soldier_files = [0u8, 2, 4, 6, 8];
        for &file in &soldier_files {
            self.set_piece(
                Square::new(file, 3),
                Some(Piece {
                    owner: PlayerSide::Blue,
                    kind: Soldier,
                }),
            );
            self.set_piece(
                Square::new(file, self.height - 4),
                Some(Piece {
                    owner: PlayerSide::Red,
                    kind: Soldier,
                }),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_bounds() {
        let board = BoardState::empty();
        let valid = Square::new(2, 3);
        let invalid = Square::new(9, 9);
        assert!(board.index(valid).is_some());
        assert!(board.index(invalid).is_none());
    }

    #[test]
    fn opponent_switch() {
        assert_eq!(PlayerSide::Blue.opponent(), PlayerSide::Red);
        assert_eq!(PlayerSide::Red.opponent(), PlayerSide::Blue);
    }

    #[test]
    fn initial_board_setup() {
        let board = BoardState::initial();
        // Blue General at (4,0)
        let general = board.piece_at(Square::new(4, 0)).expect("general present");
        assert_eq!(general.owner, PlayerSide::Blue);
        assert_eq!(general.kind, PieceKind::General);

        // Red Cannon at (1,7)
        let cannon = board
            .piece_at(Square::new(1, board.height - 3))
            .expect("cannon present");
        assert_eq!(cannon.owner, PlayerSide::Red);
        assert_eq!(cannon.kind, PieceKind::Cannon);

        // Soldier positions
        assert!(board
            .piece_at(Square::new(0, 3))
            .filter(|p| p.kind == PieceKind::Soldier)
            .is_some());
        assert!(board
            .piece_at(Square::new(8, board.height - 4))
            .filter(|p| p.owner == PlayerSide::Red && p.kind == PieceKind::Soldier)
            .is_some());
    }

    #[test]
    fn board_differences_detect_changes() {
        let a = BoardState::initial();
        let mut b = a.clone();
        let from = Square::new(0, 3);
        let to = Square::new(0, 4);
        b.move_piece(from, to).unwrap();
        let diffs = a.differences(&b);
        assert_eq!(diffs.len(), 2);
        let inferred = BoardState::infer_move_from_diffs(&diffs).expect("infer move");
        assert_eq!(inferred.0, from);
        assert_eq!(inferred.1, to);
    }
}
