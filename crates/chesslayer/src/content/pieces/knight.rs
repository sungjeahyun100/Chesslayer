use super::{movement, PieceDefinition};
use crate::PieceType;

pub const KNIGHT: PieceType = 2;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 30,
    description:
        "가로 두 칸·세로 한 칸 또는 가로 한 칸·세로 두 칸을 뜁니다. 중간 기물을 뛰어넘습니다.",
    upgrades: &[super::UpgradeDefinition {
        target: super::TEMPEST_KNIGHT,
        key: "tempest",
        effect: super::tempest_knight::DEFINITION.description,
        requirement: "살아 있는 일반 나이트 · 쉼터 행동 1회 · 추가 비용 없음",
    }],
    ..PieceDefinition::new(KNIGHT, "knight", "나이트", movement)
};

fn movement() -> String {
    movement::steps(movement::KNIGHT)
}
