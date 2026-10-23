use super::PieceDefinition;
use crate::PieceType;

pub const BOUNCING_TEMPEST_QUEEN: PieceType = 24;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 150,
    description: "템페스트 퀸의 경유 칸과 분기를 유지하며 각 경로에서 경계나 바운싱 폰 직전에 최대 한 번 반사합니다. 포획하면 멈춥니다.",
    ..PieceDefinition::new(BOUNCING_TEMPEST_QUEEN, "bouncing_tempest_queen", "바운싱 템페스트 퀸", movement)
};

pub(super) fn movement() -> String {
    format!(
        "{}; {}",
        super::bouncing_tempest_rook::movement(),
        super::bouncing_tempest_bishop::movement()
    )
}
