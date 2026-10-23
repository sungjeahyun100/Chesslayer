use super::{movement, PieceDefinition};
use crate::PieceType;

pub const ROOK: PieceType = 4;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 50,
    description: "가로·세로로 원하는 만큼 이동·포획합니다. 다른 기물을 통과할 수 없습니다.",
    upgrades: &[
        super::UpgradeDefinition {
            target: super::TEMPEST_ROOK,
            key: "tempest",
            effect: super::tempest_rook::DEFINITION.description,
            requirement: "살아 있는 일반 룩 · 쉼터 행동 1회 · 추가 비용 없음",
        },
        super::UpgradeDefinition {
            target: super::BOUNCING_ROOK,
            key: "bouncing",
            effect: super::bouncing_rook::DEFINITION.description,
            requirement: "살아 있는 일반 룩 · 쉼터 행동 1회 · 추가 비용 없음",
        },
    ],
    ..PieceDefinition::new(ROOK, "rook", "룩", movement)
};

fn movement() -> String {
    movement::slides(movement::ORTHOGONAL)
}
