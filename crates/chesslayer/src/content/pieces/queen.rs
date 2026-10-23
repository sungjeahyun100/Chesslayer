use super::{movement, PieceDefinition};
use crate::PieceType;

pub const QUEEN: PieceType = 5;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 90,
    description:
        "가로·세로·대각선으로 원하는 만큼 이동·포획합니다. 다른 기물을 통과할 수 없습니다.",
    upgrades: &[
        super::UpgradeDefinition {
            target: super::TEMPEST_QUEEN,
            key: "tempest",
            effect: super::tempest_queen::DEFINITION.description,
            requirement: "살아 있는 일반 퀸 · 쉼터 행동 1회 · 추가 비용 없음",
        },
        super::UpgradeDefinition {
            target: super::BOUNCING_QUEEN,
            key: "bouncing",
            effect: super::bouncing_queen::DEFINITION.description,
            requirement: "살아 있는 일반 퀸 · 쉼터 행동 1회 · 추가 비용 없음",
        },
    ],
    ..PieceDefinition::new(QUEEN, "queen", "퀸", movement)
};

fn movement() -> String {
    movement::slides(&movement::all_directions())
}
