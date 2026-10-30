use super::PieceDefinition;
use crate::PieceType;

pub const TRIBAL_DICTATOR: PieceType = 82;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 150,
    description: "최종 보스 전용 로얄. 킹 행마. 제거 판정마다 HP -1(시작 5), 주변 8칸의 죽음마다 HP +1과 행마 습득. 공격자를 갈고리로 지정해 다음 턴에 끌어옵니다. 3갈래 샷건은 2발 장전되며 재장전에 행동 1회를 씁니다.",
    ..PieceDefinition::new(TRIBAL_DICTATOR, "tribal_dictator", "부족의 독재자", movement)
};

// Each piece keeps its complete movement script here. Coordinates are owner-relative.
fn movement() -> String {
    "take-move(1,0) repeat(1);\ntake-move(-1,0) repeat(1);\ntake-move(0,-1) repeat(1);\ntake-move(0,1) repeat(1);\ntake-move(1,1) repeat(1);\ntake-move(1,-1) repeat(1);\ntake-move(-1,1) repeat(1);\ntake-move(-1,-1) repeat(1)".into()
}
