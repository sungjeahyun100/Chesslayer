use super::{movement, PieceDefinition, UpgradeDefinition};
use crate::PieceType;

pub const ROYAL_KNIGHT: PieceType = 95;
pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 10,
    description: "킹의 기본 행마와 나이트 도약을 함께 사용합니다.",
    upgrades: &[
        UpgradeDefinition {
            target: super::TEMPEST_KING,
            key: "tempest_king",
            effect: super::tempest_king::DEFINITION.description,
            requirement: "살아 있는 로열나이트 · 쉼터 행동 1회",
        },
        UpgradeDefinition {
            target: super::DOUBLE_KING,
            key: "double_king",
            effect: super::double_king::DEFINITION.description,
            requirement: "살아 있는 로열나이트 · 쉼터 행동 1회",
        },
    ],
    ..PieceDefinition::new(ROYAL_KNIGHT, "royal_knight", "로열나이트", movement)
};

fn movement() -> String {
    format!(
        "{};{}",
        movement::steps(&movement::all_directions()),
        movement::steps(movement::KNIGHT)
    )
}
