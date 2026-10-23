use super::PieceDefinition;
use crate::PieceType;

pub const TEMPEST_PAWN: PieceType = 8;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 10,
    description: "폰의 행마에 좌우 한 칸 이동·포획과 후방 한 칸 비포획 이동이 추가됩니다. 처음 두 칸 전진 조건은 유지됩니다.",
    initial_double_step: true,
    ..PieceDefinition::new(TEMPEST_PAWN, "tempest_pawn", "템페스트 폰", movement)
};

fn movement() -> String {
    format!(
        "{}; take-move(-1,0); take-move(1,0); move(0,-1)",
        super::pawn::PAWN_MOVEMENT
    )
}
