use super::PieceDefinition;
use crate::PieceType;

pub const DRONE: PieceType = 72;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 45,
    description:
        "나이트 도약과 상하좌우 한 칸 이동·포획. 전방위 세 칸 이내 명중률 90% 사격, 탄약 1 소모.",
    ..PieceDefinition::new(DRONE, "drone", "드론", movement)
};

// Each piece keeps its complete movement script here. Coordinates are owner-relative.
fn movement() -> String {
    "take-move(1,2);\ntake-move(2,1);\ntake-move(2,-1);\ntake-move(1,-2);\ntake-move(-1,-2);\ntake-move(-2,-1);\ntake-move(-2,1);\ntake-move(-1,2);\ntake-move(1,0);\ntake-move(-1,0);\ntake-move(0,-1);\ntake-move(0,1)".into()
}
