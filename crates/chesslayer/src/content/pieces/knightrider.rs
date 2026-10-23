use super::{movement, PieceDefinition};
use crate::PieceType;

pub const KNIGHTRIDER: PieceType = 18;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    family: super::PieceFamily::Knight,
    value: 55,
    description: "나이트의 여덟 방향 중 하나를 골라 같은 점프를 보드 안에서 반복합니다. 각 착지 칸에서 멈추거나 포획할 수 있습니다.",
    upgrades: &[
        super::UpgradeDefinition {
            target: super::UNICORN,
            key: "unicorn",
            effect: super::unicorn::DEFINITION.description,
            requirement: "살아 있는 나이트라이더 · 쉼터 행동 1회 · 추가 비용 없음",
        },
        super::UpgradeDefinition {
            target: super::BOUNCING_KNIGHTRIDER,
            key: "bouncing_knightrider",
            effect: super::bouncing_knightrider::DEFINITION.description,
            requirement: "살아 있는 나이트라이더 · 쉼터 행동 1회 · 추가 비용 없음",
        },
    ],
    ..PieceDefinition::new(KNIGHTRIDER, "knightrider", "나이트라이더", movement)
};

fn movement() -> String {
    movement::slides(movement::KNIGHT)
}
