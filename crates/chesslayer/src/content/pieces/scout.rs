use super::PieceDefinition;
use crate::PieceType;

pub const SCOUT: PieceType = 76;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 40,
    description: "나이트 도약. 매 전투 시작 시 민첩 1을 얻습니다.",
    upgrades: &[super::UpgradeDefinition {
        target: super::PHANTOM_SCOUT,
        key: "phantom_scout",
        effect: "유령 정찰병(으)로 영구 강화합니다.",
        requirement: "살아 있는 기물 · 쉼터 행동 1회",
    }],
    family: super::PieceFamily::Knight,
    ..PieceDefinition::new(SCOUT, "scout", "정찰병", movement)
};

// Each piece keeps its complete movement script here. Coordinates are owner-relative.
fn movement() -> String {
    "take-move(1,2);\ntake-move(2,1);\ntake-move(2,-1);\ntake-move(1,-2);\ntake-move(-1,-2);\ntake-move(-2,-1);\ntake-move(-2,1);\ntake-move(-1,2)".into()
}
