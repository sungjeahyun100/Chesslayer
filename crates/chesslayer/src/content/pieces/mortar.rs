use super::PieceDefinition;
use crate::PieceType;

pub const MORTAR: PieceType = 71;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 60,
    description: "상하좌우 한 칸 이동·포획. 2~5칸 거리 포격은 중간 기물을 뛰어넘으며 명중률 80%, 탄약 2 소모.",
    ..PieceDefinition::new(MORTAR, "mortar", "박격포병", movement)
};

// Each piece keeps its complete movement script here. Coordinates are owner-relative.
fn movement() -> String {
    "take-move(1,0);\ntake-move(-1,0);\ntake-move(0,-1);\ntake-move(0,1)".into()
}
