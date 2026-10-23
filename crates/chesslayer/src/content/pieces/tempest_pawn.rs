use super::PieceDefinition;
use crate::PieceType;

pub const TEMPEST_PAWN: PieceType = 8;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 10,
    glyphs: ["♙", "♟"],
    badge: Some("ϟ"),
    initial_double_step: true,
    ..PieceDefinition::new(TEMPEST_PAWN, "tempest_pawn", "템페스트 폰", movement)
};

fn movement() -> String {
    format!(
        "{}; take-move(-1,0); take-move(1,0); move(0,-1)",
        super::pawn::PAWN_MOVEMENT
    )
}
