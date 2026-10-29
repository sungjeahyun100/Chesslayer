use super::PieceDefinition;
use crate::PieceType;

pub const TANK: PieceType = 73;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 80,
    description: "킹 행마와 직선 두 칸 이동·포획. 직선으로 여섯 칸 이내 명중률 90% 사격, 탄약 2 소모. 포병 승패는 전차가 아닌 창고로 결정합니다.",
    ..PieceDefinition::new(TANK, "tank", "전차", movement)
};

// Each piece keeps its complete movement script here. Coordinates are owner-relative.
fn movement() -> String {
    "take-move(1,0);\ntake-move(-1,0);\ntake-move(0,-1);\ntake-move(0,1);\ntake-move(1,1);\ntake-move(1,-1);\ntake-move(-1,1);\ntake-move(-1,-1);\ntake-move(1,0) take-move(1,0);\ntake-move(-1,0) take-move(-1,0);\ntake-move(0,-1) take-move(0,-1);\ntake-move(0,1) take-move(0,1)".into()
}
