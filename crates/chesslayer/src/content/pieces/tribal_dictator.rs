use super::PieceDefinition;
use crate::PieceType;

pub const TRIBAL_DICTATOR: PieceType = 82;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 150,
    description: "최종 보스 전용 로얄. 킹 행마. 한 턴에 이동·포획 1회와 사격·재장전·제자리 능력 1회를 순서에 관계없이 사용합니다. 제거 판정마다 HP -1(시작 5), 주변 8칸의 죽음마다 HP +1과 행마 습득. 공격자를 갈고리로 지정해 다음 턴에 끌어옵니다. 3갈래 샷건은 2발 장전됩니다.",
    ..PieceDefinition::new(TRIBAL_DICTATOR, "tribal_dictator", "부족의 독재자", movement)
};

// Each piece keeps its complete movement script here. Coordinates are owner-relative.
fn movement() -> String {
    "take-move(1,0);\ntake-move(-1,0);\ntake-move(0,-1);\ntake-move(0,1);\ntake-move(1,1);\ntake-move(1,-1);\ntake-move(-1,1);\ntake-move(-1,-1);".into()
}
