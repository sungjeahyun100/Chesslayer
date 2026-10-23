use super::{movement, PieceDefinition};
use crate::PieceType;

pub const BOUNCING_BISHOP: PieceType = 15;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 70,
    description: "대각선으로 이동합니다. 첫 보드 경계·바운싱 폰 직전에서 가로 또는 세로 방향 성분을 반전해 직진합니다. 반사 뒤 다음 장애물에서 멈춥니다.",
    ..PieceDefinition::new(BOUNCING_BISHOP, "bouncing_bishop", "바운싱 비숍", movement)
};

pub(super) fn movement() -> String {
    movement::bouncing(movement::DIAGONAL)
}
