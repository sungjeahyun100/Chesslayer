use super::PieceDefinition;
use crate::PieceType;

pub const BOUNCING_QUEEN: PieceType = 16;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 130,
    description: "바운싱 룩과 바운싱 비숍의 반사 행마를 모두 사용합니다. 양측 바운싱 폰은 포획할 수 없으며 반사벽으로 취급합니다.",
    ..PieceDefinition::new(BOUNCING_QUEEN, "bouncing_queen", "바운싱 퀸", movement)
};

pub(super) fn movement() -> String {
    format!(
        "{}; {}",
        super::bouncing_rook::movement(),
        super::bouncing_bishop::movement()
    )
}
