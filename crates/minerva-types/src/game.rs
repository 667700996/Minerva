use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::board::{BoardState, PlayerSide, Square};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub promotion: Option<String>,
    pub confidence: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveCandidate {
    pub mv: Move,
    pub score: f32,
    pub depth: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSnapshot {
    pub board: BoardState,
    pub ply: u32,
    pub last_move: Option<Move>,
    pub phase: GamePhase,
    pub clocks: GameClocks,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct GameClocks {
    pub blue_ms: u64,
    pub red_ms: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum GamePhase {
    Opening,
    Midgame,
    Endgame,
}

impl Default for GamePhase {
    fn default() -> Self {
        GamePhase::Opening
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineDecision {
    pub best_move: Option<Move>,
    pub candidates: Vec<MoveCandidate>,
    pub searched_nodes: u64,
    pub depth: u8,
    pub duration_ms: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnContext {
    pub snapshot: GameSnapshot,
    pub side: PlayerSide,
}

impl Default for GameSnapshot {
    fn default() -> Self {
        Self {
            board: BoardState::initial(),
            ply: 0,
            last_move: None,
            phase: GamePhase::Opening,
            clocks: GameClocks::default(),
            created_at: Utc::now(),
        }
    }
}

impl GameSnapshot {
    pub fn apply_move(&mut self, side: PlayerSide, mv: &Move) -> Result<(), String> {
        let moving_piece = self.board.piece_at(mv.from).ok_or_else(|| {
            format!(
                "원점에 기물이 없습니다: ({},{})",
                mv.from.file, mv.from.rank
            )
        })?;
        if moving_piece.owner != side {
            return Err("선택한 말이 현재 플레이어의 것이 아닙니다".into());
        }
        self.board.move_piece(mv.from, mv.to)?;
        self.board.side_to_move = side.opponent();
        self.last_move = Some(mv.clone());
        self.ply += 1;
        Ok(())
    }
}
