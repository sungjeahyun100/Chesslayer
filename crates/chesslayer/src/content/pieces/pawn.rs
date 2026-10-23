use super::PieceDefinition;
use crate::PieceType;

pub const PAWN: PieceType = 1;
pub(super) const PAWN_MOVEMENT: &str = "move(0,1); enemy(-1,1) take(-1,1); enemy(1,1) take(1,1)";

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 10,
    glyphs: ["♙", "♟"],
    initial_double_step: true,
    ..PieceDefinition::new(PAWN, "pawn", "폰", movement)
};

fn movement() -> String {
    PAWN_MOVEMENT.into()
}
