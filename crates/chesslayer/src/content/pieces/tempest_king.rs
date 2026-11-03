use super::{movement, PieceDefinition, UpgradeDefinition};
use crate::PieceType;

pub const TEMPEST_KING: PieceType = 34;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 10,
    description: "킹의 기본 행마를 사용합니다. 특수 능력으로 적 기물 2기의 위치를 교환합니다.",
    upgrades: &[UpgradeDefinition {
        target: super::TYPHOON_KING,
        key: "typhoon_king",
        effect: super::typhoon_king::DEFINITION.description,
        requirement: "살아 있는 템페스트 킹 · 쉼터 행동 1회",
    }],
    ..PieceDefinition::new(TEMPEST_KING, "tempest_king", "템페스트 킹", movement)
};

fn movement() -> String {
    movement::steps(&movement::all_directions())
}
