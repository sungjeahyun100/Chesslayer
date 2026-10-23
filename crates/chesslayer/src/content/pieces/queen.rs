use super::{movement, PieceDefinition};
use crate::PieceType;

pub const QUEEN: PieceType = 5;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 90,
    glyphs: ["♕", "♛"],
    ..PieceDefinition::new(QUEEN, "queen", "퀸", movement)
};

fn movement() -> String {
    movement::slides(&movement::all_directions())
}
