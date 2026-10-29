use super::PieceDefinition;
use crate::PieceType;

pub const SHADOW_COMMANDER: PieceType = 81;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 100,
    description: "로얄. 킹·나이트 행마. 여덟 방향 다섯 칸 이내 사격, 명중률 95%에서 거리당 4%p 감소. 매 전투 시작 시 가호와 민첩 1을 얻습니다.",
    ..PieceDefinition::new(SHADOW_COMMANDER, "shadow_commander", "그림자 지휘관", movement)
};

// Each piece keeps its complete movement script here. Coordinates are owner-relative.
fn movement() -> String {
    "take-move(1,0);\ntake-move(-1,0);\ntake-move(0,-1);\ntake-move(0,1);\ntake-move(1,1);\ntake-move(1,-1);\ntake-move(-1,1);\ntake-move(-1,-1);\ntake-move(1,2);\ntake-move(2,1);\ntake-move(2,-1);\ntake-move(1,-2);\ntake-move(-1,-2);\ntake-move(-2,-1);\ntake-move(-2,1);\ntake-move(-1,2)".into()
}
