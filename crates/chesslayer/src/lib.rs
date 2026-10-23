//! The authoritative Chesslayer game: battle, cards, content and run progression.
mod battle;
pub mod cards;
pub mod content;
pub mod game;
pub mod map;
pub use battle::*;
pub use chessembly::{
    apply_default_action, ActionError, ActionStep, Board, BoardError, DefaultAction, Effect, Piece,
    PieceId, PieceType, PlayerId, Position,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultReason {
    RoyalCaptured,
    Counterattack,
    SelfDefense,
    HardDisk,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameResult {
    Ongoing,
    Win {
        winner: PlayerId,
        reason: ResultReason,
    },
}
