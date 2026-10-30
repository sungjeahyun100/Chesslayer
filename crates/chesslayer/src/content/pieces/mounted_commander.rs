use super::PieceDefinition;
use crate::PieceType;

pub const MOUNTED_COMMANDER: PieceType = 43;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 100,
    description: "로얄. 두 칸 이내 모든 칸으로 도약해 이동·포획합니다.",
    ..PieceDefinition::new(
        MOUNTED_COMMANDER,
        "mounted_commander",
        "말을 탄 선봉대장",
        movement,
    )
};

// Each piece keeps its complete movement script here. Coordinates are owner-relative.
fn movement() -> String {
    "take-move(-2,-2);\ntake-move(-2,-1);\ntake-move(-2,0);\ntake-move(-2,1);\ntake-move(-2,2);\ntake-move(-1,-2);\ntake-move(-1,-1);\ntake-move(-1,0);\ntake-move(-1,1);\ntake-move(-1,2);\ntake-move(0,-2);\ntake-move(0,-1);\ntake-move(0,1);\ntake-move(0,2);\ntake-move(1,-2);\ntake-move(1,-1);\ntake-move(1,0);\ntake-move(1,1);\ntake-move(1,2);\ntake-move(2,-2);\ntake-move(2,-1);\ntake-move(2,0);\ntake-move(2,1);\ntake-move(2,2)".into()
}
