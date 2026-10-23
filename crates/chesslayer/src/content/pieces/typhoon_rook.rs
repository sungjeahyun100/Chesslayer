use super::PieceDefinition;
use crate::PieceType;

pub const TYPHOON_ROOK: PieceType = 28;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 90,
    description: "템페스트 룩의 대각 경유·분기 행마와 일반 룩의 직교 직선 이동을 모두 사용합니다.",
    ..PieceDefinition::new(TYPHOON_ROOK, "typhoon_rook", "태풍 룩", movement)
};

pub(super) fn movement() -> String {
    format!(
        "{}; {}",
        super::tempest_rook::movement(),
        super::movement::slides(super::movement::ORTHOGONAL)
    )
}
