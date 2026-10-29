use super::PieceDefinition;
use crate::PieceType;

pub const ELITE_COMMANDO: PieceType = 75;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 70,
    description: "킹·나이트 행마. 돌격 특전대원 사격을 유지하며 기본 명중률이 100%로 증가합니다.",
    ..PieceDefinition::new(ELITE_COMMANDO, "elite_commando", "정예 특전대원", movement)
};

// Each piece keeps its complete movement script here. Coordinates are owner-relative.
fn movement() -> String {
    "take-move(1,0);\ntake-move(-1,0);\ntake-move(0,-1);\ntake-move(0,1);\ntake-move(1,1);\ntake-move(1,-1);\ntake-move(-1,1);\ntake-move(-1,-1);\ntake-move(1,2);\ntake-move(2,1);\ntake-move(2,-1);\ntake-move(1,-2);\ntake-move(-1,-2);\ntake-move(-2,-1);\ntake-move(-2,1);\ntake-move(-1,2)".into()
}
