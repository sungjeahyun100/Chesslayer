use super::PieceDefinition;
use crate::PieceType;

pub const SWORD_INFANTRY: PieceType = 37;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 40,
    description: "장창 행마에 전방 대각선 한 칸 이동·포획을 더합니다.",
    upgrades: &[super::UpgradeDefinition {
        target: super::HALBERD_INFANTRY,
        key: "halberd_infantry",
        effect: "장창과 도끼로 무장된 보병(으)로 영구 강화합니다.",
        requirement: "살아 있는 기물 · 쉼터 행동 1회",
    }],
    ..PieceDefinition::new(
        SWORD_INFANTRY,
        "sword_infantry",
        "검으로 무장한 보병",
        movement,
    )
};

// Each piece keeps its complete movement script here. Coordinates are owner-relative.
fn movement() -> String {
    "take-move(1,0);\ntake-move(-1,0);\ntake-move(0,-1);\ntake-move(0,1);\ntake-move(1,2);\ntake-move(2,1);\ntake-move(-1,2);\ntake-move(-2,1);\ntake-move(1,1);\ntake-move(-1,1)".into()
}
