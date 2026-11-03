use super::{movement, PieceDefinition};
use crate::PieceType;

pub const TRIPLE_KING: PieceType = 97;
pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 10,
    description:
        "로열나이트의 행마를 유지합니다. 자신의 턴마다 민첩 2를 얻어 두 번 더 행동할 수 있습니다.",
    ..PieceDefinition::new(TRIPLE_KING, "triple_king", "트리플 킹", movement)
};

fn movement() -> String {
    format!(
        "{};{}",
        movement::steps(&movement::all_directions()),
        movement::steps(movement::KNIGHT)
    )
}
