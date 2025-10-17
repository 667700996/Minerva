//! Search and evaluation engine abstraction.

use std::cmp::Ordering;

use async_trait::async_trait;
use minerva_types::{
    board::{BoardState, Piece, PieceKind, PlayerSide, Square},
    game::{EngineDecision, Move, MoveCandidate, TurnContext},
    MinervaError, Result,
};
use tokio::time::{sleep, Duration};
use tracing::info;

#[async_trait]
pub trait GameEngine: Send + Sync {
    async fn warm_up(&mut self) -> Result<()>;
    async fn evaluate_position(&self, ctx: &TurnContext) -> Result<EngineDecision>;
}

/// Simple deterministic engine focusing on basic move generation.
pub struct RuleBasedEngine;

impl RuleBasedEngine {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl GameEngine for RuleBasedEngine {
    async fn warm_up(&mut self) -> Result<()> {
        info!("Rule-based engine warm-up");
        sleep(Duration::from_millis(10)).await;
        Ok(())
    }

    async fn evaluate_position(&self, ctx: &TurnContext) -> Result<EngineDecision> {
        let mut candidates = generate_candidates(&ctx.snapshot.board, ctx.side);
        candidates.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(Ordering::Equal));
        let best_move = candidates.first().map(|c| c.mv.clone());

        Ok(EngineDecision {
            best_move,
            candidates,
            searched_nodes: 0,
            depth: 1,
            duration_ms: 5,
        })
    }
}

fn generate_candidates(board: &BoardState, side: PlayerSide) -> Vec<MoveCandidate> {
    let mut moves = Vec::new();

    for rank in 0..board.height {
        for file in 0..board.width {
            let square = Square::new(file, rank);
            if let Some(piece) = board.piece_at(square) {
                if piece.owner != side {
                    continue;
                }
                let mut piece_moves = match piece.kind {
                    PieceKind::Soldier => soldier_moves(board, side, square),
                    PieceKind::Chariot => rook_like_moves(board, side, square),
                    PieceKind::Horse => horse_moves(board, side, square),
                    PieceKind::Cannon => cannon_moves(board, side, square),
                    PieceKind::Guard | PieceKind::Elephant | PieceKind::General => {
                        palace_moves(board, side, square, piece.kind)
                    }
                };
                moves.append(&mut piece_moves);
            }
        }
    }

    if moves.is_empty() {
        if let Some(pass_move) = default_hold_move(board, side) {
            moves.push(pass_move);
        }
    }

    moves
}

fn soldier_moves(board: &BoardState, side: PlayerSide, from: Square) -> Vec<MoveCandidate> {
    let mut options = Vec::new();
    let forward = match side {
        PlayerSide::Blue => 1,
        PlayerSide::Red => -1,
    };
    if let Some(to) = from.offset(0, forward) {
        if board.is_empty(to) || board.piece_at(to).map(|p| p.owner != side).unwrap_or(false) {
            options.push(candidate(from, to, board.piece_at(to)));
        }
    }
    // Soldiers can move sideways after crossing river (ranks >=5 for Blue, <=4 for Red).
    let river_rank = (board.height / 2) as u8;
    if (side == PlayerSide::Blue && from.rank >= river_rank)
        || (side == PlayerSide::Red && from.rank <= river_rank.saturating_sub(1))
    {
        for df in [-1, 1] {
            if let Some(to) = from.offset(df, 0) {
                if board.is_empty(to)
                    || board.piece_at(to).map(|p| p.owner != side).unwrap_or(false)
                {
                    options.push(candidate(from, to, board.piece_at(to)));
                }
            }
        }
    }
    options
}

fn rook_like_moves(board: &BoardState, side: PlayerSide, from: Square) -> Vec<MoveCandidate> {
    let mut options = Vec::new();
    let directions = [(1, 0), (-1, 0), (0, 1), (0, -1)];
    for (df, dr) in directions {
        let mut current = from;
        while let Some(next) = current.offset(df, dr) {
            if let Some(piece) = board.piece_at(next) {
                if piece.owner != side {
                    options.push(candidate(from, next, Some(piece)));
                }
                break;
            } else {
                options.push(candidate(from, next, None));
                current = next;
            }
        }
    }
    options
}

fn cannon_moves(board: &BoardState, side: PlayerSide, from: Square) -> Vec<MoveCandidate> {
    let mut options = Vec::new();
    let directions = [(1, 0), (-1, 0), (0, 1), (0, -1)];
    for (df, dr) in directions {
        let mut current = from;
        let mut screen_found = false;
        while let Some(next) = current.offset(df, dr) {
            if let Some(piece) = board.piece_at(next) {
                if !screen_found {
                    screen_found = true;
                } else {
                    if piece.owner != side {
                        options.push(candidate(from, next, Some(piece)));
                    }
                    break;
                }
            } else if !screen_found {
                options.push(candidate(from, next, None));
            }
            current = next;
        }
    }
    options
}

fn horse_moves(board: &BoardState, side: PlayerSide, from: Square) -> Vec<MoveCandidate> {
    let mut options = Vec::new();
    let patterns = [
        ((1, 0), (1, 1)),
        ((1, 0), (1, -1)),
        ((-1, 0), (-1, 1)),
        ((-1, 0), (-1, -1)),
        ((0, 1), (1, 1)),
        ((0, 1), (-1, 1)),
        ((0, -1), (1, -1)),
        ((0, -1), (-1, -1)),
    ];
    for (leg, dest) in patterns {
        if let Some(block) = from.offset(leg.0, leg.1) {
            if board.is_empty(block) {
                if let Some(to) = block.offset(dest.0, dest.1) {
                    if board.is_empty(to)
                        || board.piece_at(to).map(|p| p.owner != side).unwrap_or(false)
                    {
                        options.push(candidate(from, to, board.piece_at(to)));
                    }
                }
            }
        }
    }
    options
}

fn palace_moves(
    board: &BoardState,
    side: PlayerSide,
    from: Square,
    kind: PieceKind,
) -> Vec<MoveCandidate> {
    let palace_files = [3u8, 4, 5];
    let palace_ranks = match side {
        PlayerSide::Blue => [0u8, 1, 2],
        PlayerSide::Red => [board.height - 1, board.height - 2, board.height - 3],
    };

    let mut options = Vec::new();
    let directions = match kind {
        PieceKind::Guard | PieceKind::General => {
            vec![
                (1, 0),
                (-1, 0),
                (0, 1),
                (0, -1),
                (1, 1),
                (-1, 1),
                (1, -1),
                (-1, -1),
            ]
        }
        PieceKind::Elephant => vec![(2, 2), (2, -2), (-2, 2), (-2, -2)],
        _ => vec![],
    };

    for (df, dr) in directions {
        if let Some(to) = from.offset(df, dr) {
            if palace_files.contains(&to.file) && palace_ranks.contains(&to.rank) {
                if board.is_empty(to)
                    || board.piece_at(to).map(|p| p.owner != side).unwrap_or(false)
                {
                    options.push(candidate(from, to, board.piece_at(to)));
                }
            }
        }
    }
    options
}

fn candidate(from: Square, to: Square, capture: Option<Piece>) -> MoveCandidate {
    let score = capture.map(piece_value).unwrap_or(0.1);
    MoveCandidate {
        mv: Move {
            from,
            to,
            promotion: None,
            confidence: Some(score as f32),
        },
        score,
        depth: 1,
    }
}

fn piece_value(piece: Piece) -> f32 {
    match piece.kind {
        PieceKind::General => 1000.0,
        PieceKind::Guard => 3.0,
        PieceKind::Elephant => 5.0,
        PieceKind::Horse => 7.0,
        PieceKind::Chariot => 13.0,
        PieceKind::Cannon => 9.0,
        PieceKind::Soldier => 1.0,
    }
}

fn default_hold_move(board: &BoardState, side: PlayerSide) -> Option<MoveCandidate> {
    for rank in 0..board.height {
        for file in 0..board.width {
            let square = Square::new(file, rank);
            if let Some(piece) = board.piece_at(square) {
                if piece.owner == side {
                    return Some(MoveCandidate {
                        mv: Move {
                            from: square,
                            to: square,
                            promotion: None,
                            confidence: Some(0.0),
                        },
                        score: 0.0,
                        depth: 0,
                    });
                }
            }
        }
    }
    None
}

pub fn engine_error(message: impl Into<String>) -> MinervaError {
    MinervaError::Engine(message.into())
}
