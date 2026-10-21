use crate::{Board, GameAction, PlayerId};

/// The result must describe the reason without binding the shared core to standard chess.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultReason {
    RoyalCaptured,
    Checkmate,
    Stalemate,
    RuleDefined(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameResult {
    Ongoing,
    Win { winner: PlayerId, reason: ResultReason },
    Draw { reason: ResultReason },
}

/// Shared turn and result only. Each ruleset owns its additional state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameState {
    pub board: Board,
    pub turn: PlayerId,
    pub result: GameResult,
}

/// Rule-dependent definitions and state are associated types, not a universal schema.
/// The engine core does not dictate what makes a move legal or how a run ends.
pub trait RuleModule {
    type State;
    type Definition;

    fn piece_definitions(&self) -> &[Self::Definition];
    fn board_boundary(&self) -> Board;
    fn initial_position(&self) -> Self::State;
    fn legal_actions(&self, state: &mut Self::State) -> Vec<GameAction>;
    fn apply_action(&self, state: &mut Self::State, action: &GameAction) -> bool;
    fn victory_condition(&self, state: &mut Self::State) -> GameResult;
}
