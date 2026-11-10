use super::{PieceDefinition, UpgradeDefinition};
use crate::PieceType;

pub const DOUBLE_KING: PieceType = 96;
pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 10,
    description: "한 행동으로 킹 이동을 최대 2단계 수행합니다. 두 번째 단계는 직진 또는 좌우 45도 방향이며, 각 단계에서 멈출 수 있습니다. 아군을 통과할 수 없고 포획하면 이동을 마칩니다.",
    upgrades: &[UpgradeDefinition {
        target: super::TRIPLE_KING,
        key: "triple_king",
        effect: super::triple_king::DEFINITION.description,
        requirement: "살아 있는 더블 킹 · 쉼터 행동 1회",
    }],
    ..PieceDefinition::new(DOUBLE_KING, "double_king", "더블 킹", movement)
};

fn movement() -> String {
    (super::vanguard_commander::DEFINITION.movement)()
}
