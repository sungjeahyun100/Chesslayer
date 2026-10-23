use super::PieceDefinition;
use crate::PieceType;

pub const BOUNCING_PAWN: PieceType = 13;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 20,
    description: "일반 폰처럼 전진·대각 포획합니다. 이동 전 초기 랭크에서는 두 칸 전진합니다. 양측 바운싱 기물은 이 폰에 진입·포획하지 않고 직전 칸에서 반사합니다.",
    initial_double_step: true,
    promotions: &[super::BOUNCING_ROOK, super::BOUNCING_BISHOP, super::BOUNCING_QUEEN],
    ..PieceDefinition::new(BOUNCING_PAWN, "bouncing_pawn", "바운싱 폰", movement)
};

pub(super) fn movement() -> String {
    super::pawn::PAWN_MOVEMENT.into()
}
