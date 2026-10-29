use super::PieceDefinition;
use crate::PieceType;

pub const TRIBAL_DICTATOR: PieceType = 82;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 150,
    description: "최종 보스 전용 로얄. 퀸·나이트 행마. 매 세 번째 적 턴에 가장 가까운 아군 하나에 민첩 1을 부여합니다. 직접 포획해야 승리합니다.",
    ..PieceDefinition::new(TRIBAL_DICTATOR, "tribal_dictator", "부족의 독재자", movement)
};

// Each piece keeps its complete movement script here. Coordinates are owner-relative.
fn movement() -> String {
    "take-move(1,0) repeat(1);\ntake-move(-1,0) repeat(1);\ntake-move(0,-1) repeat(1);\ntake-move(0,1) repeat(1);\ntake-move(1,1) repeat(1);\ntake-move(1,-1) repeat(1);\ntake-move(-1,1) repeat(1);\ntake-move(-1,-1) repeat(1);\ntake-move(1,2);\ntake-move(2,1);\ntake-move(2,-1);\ntake-move(1,-2);\ntake-move(-1,-2);\ntake-move(-2,-1);\ntake-move(-2,1);\ntake-move(-1,2)".into()
}
