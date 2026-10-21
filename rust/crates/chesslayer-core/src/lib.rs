//! Shared, rule-agnostic types for the Chesslayer engine.
//! Game-specific movement and victory conditions do not belong in this crate.

mod action;
mod board;
mod rules;

pub use action::{
    apply_default_action, ActionError, ActionStep, DefaultAction, Effect, GameAction, Move,
};
pub use board::{Board, BoardError, Piece, PieceId, PieceType, PlayerId, Position};
pub use rules::{GameResult, GameState, ResultReason, RuleModule};
