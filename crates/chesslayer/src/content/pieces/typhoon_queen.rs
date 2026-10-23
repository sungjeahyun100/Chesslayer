use super::PieceDefinition;
use crate::PieceType;

pub const TYPHOON_QUEEN: PieceType = 27;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 125,
    description: "템페스트 퀸의 경유·분기 행마에 일반 룩과 비숍의 직선 이동을 더합니다. 포획하면 해당 경로는 멈춥니다.",
    ..PieceDefinition::new(TYPHOON_QUEEN, "typhoon_queen", "태풍 퀸", movement)
};

pub(super) fn movement() -> String {
    format!(
        "{}; {}; {}",
        super::tempest_queen::movement(),
        super::movement::slides(super::movement::ORTHOGONAL),
        super::movement::slides(super::movement::DIAGONAL)
    )
}
