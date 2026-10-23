use super::{movement, PieceDefinition};
use crate::PieceType;

pub const BOUNCING_ROOK: PieceType = 14;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 60,
    description: "직교로 이동합니다. 첫 보드 경계·바운싱 폰 직전에서 양쪽 직교 방향으로 꺾어 직진합니다. 반사 뒤 다음 장애물에서 멈추며, 일반 적은 포획할 수 있습니다.",
    ..PieceDefinition::new(BOUNCING_ROOK, "bouncing_rook", "바운싱 룩", movement)
};

pub(super) fn movement() -> String {
    movement::bouncing(movement::ORTHOGONAL)
}
