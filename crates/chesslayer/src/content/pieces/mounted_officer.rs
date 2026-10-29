use super::PieceDefinition;
use crate::PieceType;

pub const MOUNTED_OFFICER: PieceType = 41;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 65,
    description: "킹과 나이트 행마를 모두 사용하며 나이트 이동은 기물을 뛰어넘습니다.",
    ..PieceDefinition::new(
        MOUNTED_OFFICER,
        "mounted_officer",
        "말을 탄 선봉 부대장",
        movement,
    )
};

// Each piece keeps its complete movement script here. Coordinates are owner-relative.
fn movement() -> String {
    "take-move(1,0);\ntake-move(-1,0);\ntake-move(0,-1);\ntake-move(0,1);\ntake-move(1,1);\ntake-move(1,-1);\ntake-move(-1,1);\ntake-move(-1,-1);\ntake-move(1,2);\ntake-move(2,1);\ntake-move(2,-1);\ntake-move(1,-2);\ntake-move(-1,-2);\ntake-move(-2,-1);\ntake-move(-2,1);\ntake-move(-1,2)".into()
}
