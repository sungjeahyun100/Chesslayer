use super::PieceDefinition;
use crate::PieceType;

pub const HALBERD_INFANTRY: PieceType = 38;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 50,
    description: "장창 행마와 전방 대각선 최대 두 칸의 도끼 행마를 함께 사용합니다.",
    upgrades: &[super::UpgradeDefinition {
        target: super::PISTOL_INFANTRY,
        key: "pistol_infantry",
        effect: "권총과 냉병기로 무장된 보병(으)로 영구 강화합니다.",
        requirement: "살아 있는 기물 · 쉼터 행동 1회",
    }],
    ..PieceDefinition::new(
        HALBERD_INFANTRY,
        "halberd_infantry",
        "장창과 도끼로 무장된 보병",
        movement,
    )
};

// Each piece keeps its complete movement script here. Coordinates are owner-relative.
fn movement() -> String {
    "take-move(1,0);\ntake-move(-1,0);\ntake-move(0,-1);\ntake-move(0,1);\ntake-move(1,2);\ntake-move(2,1);\ntake-move(-1,2);\ntake-move(-2,1);\ntake-move(1,1) take-move(1,1);\ntake-move(-1,1) take-move(-1,1)".into()
}
