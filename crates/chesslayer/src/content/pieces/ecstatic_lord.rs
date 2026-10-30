use super::PieceDefinition;
use crate::PieceType;

pub const ECSTATIC_LORD: PieceType = 46;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 100,
    description: "로얄. 전투당 1회, 보드 어디서든 아군이 시체를 포획하면 카드 1장, 포획자 민첩 1, 마나 1을 얻습니다.",
    ..PieceDefinition::new(ECSTATIC_LORD, "ecstatic_lord", "환희의 군주", movement)
};

// Each piece keeps its complete movement script here. Coordinates are owner-relative.
fn movement() -> String {
    "take-move(1,0);\ntake-move(-1,0);\ntake-move(0,-1);\ntake-move(0,1);\ntake-move(1,1);\ntake-move(1,-1);\ntake-move(-1,1);\ntake-move(-1,-1)".into()
}
