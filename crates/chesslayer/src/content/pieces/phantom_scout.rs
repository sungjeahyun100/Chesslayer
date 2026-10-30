use super::PieceDefinition;
use crate::PieceType;

pub const PHANTOM_SCOUT: PieceType = 77;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 60,
    description: "나이트 및 대각선 두 칸 도약. 매 전투 시작 시 민첩 2를 얻습니다.",
    family: super::PieceFamily::Knight,
    ..PieceDefinition::new(PHANTOM_SCOUT, "phantom_scout", "유령 정찰병", movement)
};

// Each piece keeps its complete movement script here. Coordinates are owner-relative.
fn movement() -> String {
    "take-move(1,2);\ntake-move(2,1);\ntake-move(2,-1);\ntake-move(1,-2);\ntake-move(-1,-2);\ntake-move(-2,-1);\ntake-move(-2,1);\ntake-move(-1,2);\ntake-move(2,2);\ntake-move(2,-2);\ntake-move(-2,2);\ntake-move(-2,-2)".into()
}
