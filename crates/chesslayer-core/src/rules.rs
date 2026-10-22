use crate::{Board, Position};

/// Reasons are descriptive; a module decides when an outcome occurs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultReason {
    RoyalCaptured,
    RuleDefined(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameResult {
    Ongoing,
    Win {
        winner: crate::PlayerId,
        reason: ResultReason,
    },
    Draw {
        reason: ResultReason,
    },
}

/// Shared turn and result only. Each ruleset owns its additional state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameState {
    pub board: Board,
    pub turn: crate::PlayerId,
    pub result: GameResult,
}

/// A rule module supplies its own setup, action, state and outcome types.
/// The core imposes no piece-definition schema or victory logic.
pub trait RuleModule {
    type Setup;
    type State;
    type Action;
    type Outcome;
    type Error;

    fn start(&self, setup: Self::Setup) -> Result<Self::State, Self::Error>;
    fn legal_actions(
        &self,
        state: &Self::State,
        from: Position,
    ) -> Result<Vec<Self::Action>, Self::Error>;
    fn apply_action(
        &self,
        state: &mut Self::State,
        action: &Self::Action,
    ) -> Result<(), Self::Error>;
    fn result(&self, state: &Self::State) -> GameResult;
    fn outcome(&self, state: &Self::State) -> Self::Outcome;
}
