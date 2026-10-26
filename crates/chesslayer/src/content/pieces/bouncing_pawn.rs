use super::PieceDefinition;
use crate::PieceType;

pub const BOUNCING_PAWN: PieceType = 13;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 20,
    description: "일반 폰처럼 전진·대각 포획합니다. 이동 전 초기 랭크에서는 두 칸 전진합니다. 아군 바운싱 기물은 이 폰 앞에서 반사하며 적군 바운싱 기물은 포획할 수 있습니다.",
    initial_double_step: true,
    promotions: &[super::BOUNCING_ROOK, super::BOUNCING_BISHOP, super::BOUNCING_QUEEN],
    ..PieceDefinition::new(BOUNCING_PAWN, "bouncing_pawn", "바운싱 폰", movement)
};

pub(super) fn movement() -> String {
    "move(0,1); enemy(-1,1) take(-1,1); enemy(1,1) take(1,1)".into()
}
