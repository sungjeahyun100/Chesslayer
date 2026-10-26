use super::PieceDefinition;
use crate::{ActionStep, Board, DefaultAction, Effect, PieceType, Position};
use chessembly::Error as MovementError;
use std::collections::HashSet;

pub const CARDINAL: PieceType = 22;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    family: super::PieceFamily::Bishop,
    value: 100,
    description: "대각선으로 이동하며 보드 경계와 아군 바운싱 폰에서 가로·세로 성분 중 하나를 반전해 횟수 제한 없이 반사합니다. 적을 포획하면 멈춥니다.",
    native_movement: Some(actions),
    ..PieceDefinition::new(CARDINAL, "cardinal", "추기경", movement)
};

// Unlimited branching reflection needs a visited (square, direction) set.
// Chesssembly's finite blocks, slots and repetition cannot retain that set.
fn movement() -> String {
    String::new()
}

pub(super) fn actions(
    board: &Board,
    origin: Position,
) -> Result<Vec<DefaultAction>, MovementError> {
    let actor = board.piece_at(origin)?.ok_or(MovementError::EmptyOrigin)?;
    let facing = i32::from(board.direction(actor.owner)?);
    let mut out = Vec::new();
    for &(dx, dy) in super::movement::DIAGONAL {
        let mut pending = vec![(origin, (dx, dy), vec![origin])];
        let mut visited = HashSet::new();
        while let Some((at, (dx, dy), route)) = pending.pop() {
            if !visited.insert((at, dx, dy)) {
                continue;
            }
            let next = Position::new(
                at.file.saturating_add(dx * facing),
                at.rank.saturating_sub(dy * facing),
            );
            let occupant = if board.contains(next) {
                board.piece_at(next)?
            } else {
                None
            };
            let wall = !board.contains(next)
                || occupant
                    .is_some_and(|p| p.owner == actor.owner && p.kind == super::BOUNCING_PAWN);
            if wall {
                pending.push((at, (-dx, dy), route.clone()));
                pending.push((at, (dx, -dy), route));
            } else if !occupant.is_some_and(|p| p.owner == actor.owner) {
                let mut next_route = route;
                next_route.push(next);
                let mut action =
                    DefaultAction::one(ActionStep::new(origin, next, next, Effect::TakeMove));
                action.route = Some(next_route.clone());
                out.push(action);
                if occupant.is_none() {
                    pending.push((next, (dx, dy), next_route));
                }
            }
        }
    }
    Ok(out)
}
