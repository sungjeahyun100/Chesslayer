use super::PieceDefinition;
use crate::PieceType;

pub const MARKSMAN: PieceType = 79;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 90,
    description: "킹 행마. 여덟 방향 일곱 칸 사격, 기본 명중률 100%, 거리당 2%p 감소.",
    ..PieceDefinition::new(MARKSMAN, "marksman", "명사수", movement)
};

// Each piece keeps its complete movement script here. Coordinates are owner-relative.
fn movement() -> String {
    "take-move(1,0);\ntake-move(-1,0);\ntake-move(0,-1);\ntake-move(0,1);\ntake-move(1,1);\ntake-move(1,-1);\ntake-move(-1,1);\ntake-move(-1,-1)".into()
}
