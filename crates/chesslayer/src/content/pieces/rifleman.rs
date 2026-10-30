use super::PieceDefinition;
use crate::PieceType;

pub const RIFLEMAN: PieceType = 70;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 25,
    description: "상하좌우 한 칸 이동·포획. 전방 세 방향 6칸 이내 제자리 사격, 명중 95%에서 추가 거리마다 5%p 감소, 탄약 1 소모.",
    ..PieceDefinition::new(RIFLEMAN, "rifleman", "소총수", movement)
};

// Each piece keeps its complete movement script here. Coordinates are owner-relative.
fn movement() -> String {
    "take-move(1,0);\ntake-move(-1,0);\ntake-move(0,-1);\ntake-move(0,1)".into()
}
