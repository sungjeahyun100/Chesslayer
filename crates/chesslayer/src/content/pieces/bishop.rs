use super::{movement, PieceDefinition};
use crate::PieceType;

pub const BISHOP: PieceType = 3;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    family: super::PieceFamily::Bishop,
    value: 30,
    description: "대각선으로 원하는 만큼 이동·포획합니다. 다른 기물을 통과할 수 없습니다.",
    upgrades: &[
        super::UpgradeDefinition {
            target: super::TEMPEST_BISHOP,
            key: "tempest",
            effect: super::tempest_bishop::DEFINITION.description,
            requirement: "살아 있는 일반 비숍 · 쉼터 행동 1회 · 추가 비용 없음",
        },
        super::UpgradeDefinition {
            target: super::BOUNCING_BISHOP,
            key: "bouncing",
            effect: super::bouncing_bishop::DEFINITION.description,
            requirement: "살아 있는 일반 비숍 · 쉼터 행동 1회 · 추가 비용 없음",
        },
    ],
    ..PieceDefinition::new(BISHOP, "bishop", "비숍", movement)
};

fn movement() -> String {
    movement::slides(movement::DIAGONAL)
}
