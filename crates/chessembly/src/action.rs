use crate::{Board, Piece, PlayerId, Position};

/// Shared, invariant effects. Rule modules may reject an action but cannot reinterpret its effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    Move,
    TakeMove,
    Take,
    TakeJump,
    Catch,
    Shift,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionStep {
    pub from: Position,
    pub take: Position,
    pub move_to: Position,
    pub effect: Effect,
    pub transition: Option<String>,
}

impl ActionStep {
    pub fn new(from: Position, take: Position, move_to: Position, effect: Effect) -> Self {
        Self {
            from,
            take,
            move_to,
            effect,
            transition: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DefaultAction {
    pub first: ActionStep,
    pub continuation: Vec<ActionStep>,
    /// Confirmed landing points from the DSL evaluator. None means the route is unknown.
    pub route: Option<Vec<Position>>,
}

// A route describes an action; it is not part of its legality or identity.
impl PartialEq for DefaultAction {
    fn eq(&self, other: &Self) -> bool {
        self.first == other.first && self.continuation == other.continuation
    }
}
impl Eq for DefaultAction {}

impl DefaultAction {
    pub fn one(first: ActionStep) -> Self {
        Self {
            first,
            continuation: Vec::new(),
            route: None,
        }
    }

    pub fn steps(&self) -> impl Iterator<Item = &ActionStep> {
        std::iter::once(&self.first).chain(self.continuation.iter())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionError {
    OutsideBoard,
    EmptySource,
    InvalidEffect,
    UnknownTransition,
}

/// Applies an ordered action atomically, without cloning the whole board.
/// Only squares that change are snapshotted, and a failed continuation is rolled back.
pub fn apply_default_action(board: &mut Board, action: &DefaultAction) -> Result<(), ActionError> {
    let mut undo = Vec::<(usize, Option<Piece>)>::new();
    for step in action.steps() {
        if let Err(error) = apply_step(board, step, &mut undo) {
            for (index, previous) in undo.into_iter().rev() {
                board.cells_mut()[index] = previous;
            }
            return Err(error);
        }
    }
    Ok(())
}

fn apply_step(
    board: &mut Board,
    step: &ActionStep,
    undo: &mut Vec<(usize, Option<Piece>)>,
) -> Result<(), ActionError> {
    let from = board.index(step.from).ok_or(ActionError::OutsideBoard)?;
    let take = board.index(step.take).ok_or(ActionError::OutsideBoard)?;
    let to = board.index(step.move_to).ok_or(ActionError::OutsideBoard)?;

    let mut mover = board.cells()[from].ok_or(ActionError::EmptySource)?;
    let victim = board.cells()[take];
    let destination = board.cells()[to];
    let enemy = |piece: Piece, owner: PlayerId| piece.owner != owner;

    let valid = match step.effect {
        Effect::Move => take == to && from != to && destination.is_none(),
        Effect::TakeMove => {
            take == to && from != to && destination.is_none_or(|piece| enemy(piece, mover.owner))
        }
        Effect::Take => {
            take == to && from != to && victim.is_some_and(|piece| enemy(piece, mover.owner))
        }
        Effect::TakeJump => {
            take != to
                && from != to
                && destination.is_none()
                && victim.is_some_and(|piece| enemy(piece, mover.owner))
        }
        Effect::Catch => {
            to == from && take != from && victim.is_some_and(|piece| enemy(piece, mover.owner))
        }
        Effect::Shift => take == to && from != to && destination.is_some(),
    };
    if !valid {
        return Err(ActionError::InvalidEffect);
    }

    if let Some(name) = step.transition.as_deref() {
        mover.kind = board
            .find_piece_type(name)
            .ok_or(ActionError::UnknownTransition)?;
    }

    for index in [from, take, to] {
        if !undo.iter().any(|(previous, _)| *previous == index) {
            undo.push((index, board.cells()[index]));
        }
    }

    match step.effect {
        Effect::Shift => {
            board.cells_mut()[from] = destination;
            board.cells_mut()[to] = Some(mover);
        }
        Effect::Catch => {
            board.cells_mut()[take] = None;
            board.cells_mut()[from] = Some(mover);
        }
        _ => {
            board.cells_mut()[from] = None;
            if step.effect == Effect::TakeJump {
                board.cells_mut()[take] = None;
            }
            board.cells_mut()[to] = Some(mover);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Board {
        let mut board = Board::new(8, 8).unwrap();
        board
            .set_catalog(
                vec!["".into(), "runner".into(), "target".into()],
                vec!["white".into(), "black".into()],
                vec![1, -1],
            )
            .unwrap();
        board
            .set_piece(
                Position::new(2, 2),
                Some(Piece {
                    kind: 1,
                    owner: 0,
                    id: 10,
                }),
            )
            .unwrap();
        board
            .set_piece(
                Position::new(3, 2),
                Some(Piece {
                    kind: 2,
                    owner: 1,
                    id: 20,
                }),
            )
            .unwrap();
        board
    }

    fn step(effect: Effect, to: Position, take: Position) -> ActionStep {
        ActionStep::new(Position::new(2, 2), take, to, effect)
    }

    #[test]
    fn moving_capture_stay_capture_and_swap_preserve_identity() {
        let target = Position::new(3, 2);
        let mut board = sample();
        apply_default_action(
            &mut board,
            &DefaultAction::one(step(Effect::Take, target, target)),
        )
        .unwrap();
        assert_eq!(board.piece_at(target).unwrap().unwrap().id, 10);

        let mut board = sample();
        let home = Position::new(2, 2);
        apply_default_action(
            &mut board,
            &DefaultAction::one(step(Effect::Catch, home, target)),
        )
        .unwrap();
        assert_eq!(board.piece_at(home).unwrap().unwrap().id, 10);
        assert_eq!(board.piece_at(target).unwrap(), None);

        let mut board = sample();
        apply_default_action(
            &mut board,
            &DefaultAction::one(step(Effect::Shift, target, target)),
        )
        .unwrap();
        assert_eq!(board.piece_at(home).unwrap().unwrap().id, 20);
        assert_eq!(board.piece_at(target).unwrap().unwrap().id, 10);
    }

    #[test]
    fn jump_capture_and_transition() {
        let mut board = sample();
        let mut s = step(Effect::TakeJump, Position::new(4, 2), Position::new(3, 2));
        s.transition = Some("target".into());
        apply_default_action(&mut board, &DefaultAction::one(s)).unwrap();
        let moved = board.piece_at(Position::new(4, 2)).unwrap().unwrap();
        assert_eq!((moved.kind, moved.id), (2, 10));
        assert_eq!(board.piece_at(Position::new(3, 2)).unwrap(), None);
    }

    #[test]
    fn failed_continuation_rolls_back_all_changes() {
        let mut board = sample();
        let before = board.clone();
        let mut action =
            DefaultAction::one(step(Effect::Move, Position::new(1, 2), Position::new(1, 2)));
        action.continuation.push(ActionStep::new(
            Position::new(1, 2),
            Position::new(3, 2),
            Position::new(3, 2),
            Effect::Move,
        ));
        assert_eq!(
            apply_default_action(&mut board, &action),
            Err(ActionError::InvalidEffect)
        );
        assert_eq!(board, before);
    }
}
