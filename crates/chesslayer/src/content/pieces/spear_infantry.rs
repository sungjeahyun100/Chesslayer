use super::PieceDefinition;
use crate::PieceType;

pub const SPEAR_INFANTRY: PieceType = 36;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 30,
    description: "상하좌우 한 칸 또는 전방 나이트 네 방향으로 이동·포획합니다.",
    upgrades: &[super::UpgradeDefinition {
        target: super::SWORD_INFANTRY,
        key: "sword_infantry",
        effect: "검으로 무장한 보병(으)로 영구 강화합니다.",
        requirement: "살아 있는 기물 · 쉼터 행동 1회",
    }],
    ..PieceDefinition::new(
        SPEAR_INFANTRY,
        "spear_infantry",
        "장창으로 무장된 보병",
        movement,
    )
};

// Each piece keeps its complete movement script here. Coordinates are owner-relative.
fn movement() -> String {
    "take-move(1,0);\ntake-move(-1,0);\ntake-move(0,-1);\ntake-move(0,1);\ntake-move(1,2);\ntake-move(2,1);\ntake-move(-1,2);\ntake-move(-2,1)".into()
}
