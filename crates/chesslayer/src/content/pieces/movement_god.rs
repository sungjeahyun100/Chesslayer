use super::PieceDefinition;
use crate::PieceType;

pub const MOVEMENT_GOD: PieceType = 23;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 160,
    description: "구행의 직교 꺾기와 텐쿠의 대각 꺾기를 모두 사용합니다. 경유 칸에서 멈추거나 이어가며 포획 뒤에는 멈춥니다.",
    ..PieceDefinition::new(MOVEMENT_GOD, "movement_god", "행마의 신", movement)
};

pub(super) fn movement() -> String {
    format!("{}; {}", super::hook::movement(), super::tengu::movement())
}
