use super::PieceDefinition;
use crate::PieceType;

pub const INTOXICATED_PRIEST: PieceType = 55;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 35,
    description: "비숍 행마. 시체 섭취의 부정적 효과를 받지 않고 시체 포획 시 무작위 아군 하나에 행마권 1을 부여합니다.",
    upgrades: &[super::UpgradeDefinition {
        target: super::FEAST_PRIEST,
        key: "feast_priest",
        effect: "향연의 사제(으)로 영구 강화합니다.",
        requirement: "살아 있는 기물 · 쉼터 행동 1회",
    }],
    family: super::PieceFamily::Bishop,
    ..PieceDefinition::new(
        INTOXICATED_PRIEST,
        "intoxicated_priest",
        "도취의 사제",
        movement,
    )
};

// Each piece keeps its complete movement script here. Coordinates are owner-relative.
fn movement() -> String {
    "take-move(1,1) repeat(1);\ntake-move(1,-1) repeat(1);\ntake-move(-1,1) repeat(1);\ntake-move(-1,-1) repeat(1)".into()
}
