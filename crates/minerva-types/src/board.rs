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
}

/// Piece with its owner and optional promotion metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Piece {
    pub owner: PlayerSide,
    pub kind: PieceKind,
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
}
