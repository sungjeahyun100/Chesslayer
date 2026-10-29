use super::PieceDefinition;
use crate::PieceType;

pub const FEAST_PRIEST: PieceType = 56;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 45,
    description:
        "비숍 행마. 전투당 1회, 시체 포획 시 두 칸 이내 다른 아군 하나에 민첩 1을 부여합니다.",
    upgrades: &[
        super::UpgradeDefinition {
            target: super::LOCUST_HIGH_PRIEST,
            key: "locust_high_priest",
            effect: "황충의 대사제(으)로 영구 강화합니다.",
            requirement: "살아 있는 기물 · 쉼터 행동 1회",
        },
        super::UpgradeDefinition {
            target: super::HUNGER_PRIEST,
            key: "hunger_priest",
            effect: "허기의 사제(으)로 영구 강화합니다.",
            requirement: "살아 있는 기물 · 쉼터 행동 1회",
        },
    ],
    family: super::PieceFamily::Bishop,
    ..PieceDefinition::new(FEAST_PRIEST, "feast_priest", "향연의 사제", movement)
};

// Each piece keeps its complete movement script here. Coordinates are owner-relative.
fn movement() -> String {
    "take-move(1,1) repeat(1);\ntake-move(1,-1) repeat(1);\ntake-move(-1,1) repeat(1);\ntake-move(-1,-1) repeat(1)".into()
}
