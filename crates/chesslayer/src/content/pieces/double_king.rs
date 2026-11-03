use super::{movement, PieceDefinition, UpgradeDefinition};
use crate::PieceType;

pub const DOUBLE_KING: PieceType = 96;
pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 10,
    description:
        "로열나이트의 행마를 유지합니다. 자신의 턴마다 민첩 1을 얻어 한 번 더 행동할 수 있습니다.",
    upgrades: &[UpgradeDefinition {
        target: super::TRIPLE_KING,
        key: "triple_king",
        effect: super::triple_king::DEFINITION.description,
        requirement: "살아 있는 더블 킹 · 쉼터 행동 1회",
    }],
    ..PieceDefinition::new(DOUBLE_KING, "double_king", "더블 킹", movement)
};

fn movement() -> String {
    format!(
        "{};{}",
        movement::steps(&movement::all_directions()),
        movement::steps(movement::KNIGHT)
    )
}
