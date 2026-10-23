use super::PieceDefinition;
use crate::PieceType;

pub const BOUNCING_TEMPEST_ROOK: PieceType = 25;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 105,
    description: "대각 경유 칸에서 가로 또는 세로로 갈라집니다. 각 실제 경로는 경계나 바운싱 폰 직전에 최대 한 번 반사하며 포획하면 멈춥니다.",
    ..PieceDefinition::new(BOUNCING_TEMPEST_ROOK, "bouncing_tempest_rook", "바운싱 템페스트 룩", movement)
};

pub(super) fn movement() -> String {
    super::movement::bouncing_tempest(
        super::movement::DIAGONAL,
        super::movement::tempest_rook_branches,
    )
}
