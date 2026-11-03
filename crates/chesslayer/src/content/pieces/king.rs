use super::{movement, PieceDefinition, UpgradeDefinition};
use crate::PieceType;

pub const KING: PieceType = 6;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 10,
    description: "모든 방향으로 한 칸 이동·포획합니다. 아군 킹을 잃으면 원정이 끝납니다.",
    upgrades: &[
        UpgradeDefinition {
            target: super::RACING_KING,
            key: "racing_king",
            effect: super::racing_king::DEFINITION.description,
            requirement: "살아 있는 킹 · 쉼터 행동 1회",
        },
        UpgradeDefinition {
            target: super::ROYAL_KNIGHT,
            key: "royal_knight",
            effect: super::royal_knight::DEFINITION.description,
            requirement: "살아 있는 킹 · 쉼터 행동 1회",
        },
    ],
    ..PieceDefinition::new(KING, "king", "킹", movement)
};

fn movement() -> String {
    movement::steps(&movement::all_directions())
}
