use super::{movement, PieceDefinition, UpgradeDefinition};
use crate::PieceType;

pub const RACING_KING: PieceType = 91;
pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 10,
    description: "킹의 기본 행마를 사용합니다. 상대 진영의 끝 랭크에 도달하면 승리합니다.",
    upgrades: &[
        UpgradeDefinition {
            target: super::GAMBLER,
            key: "gambler",
            effect: super::gambler::DEFINITION.description,
            requirement: "살아 있는 레이싱 킹 · 쉼터 행동 1회",
        },
        UpgradeDefinition {
            target: super::HILL_KING,
            key: "hill_king",
            effect: super::hill_king::DEFINITION.description,
            requirement: "살아 있는 레이싱 킹 · 쉼터 행동 1회",
        },
    ],
    ..PieceDefinition::new(RACING_KING, "racing_king", "레이싱 킹", movement)
};

fn movement() -> String {
    movement::steps(&movement::all_directions())
}
