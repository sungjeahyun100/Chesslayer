use super::{movement, PieceDefinition};
use crate::PieceType;

pub const BISHOP: PieceType = 3;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 30,
    glyphs: ["♗", "♝"],
    ..PieceDefinition::new(BISHOP, "bishop", "비숍", movement)
};

fn movement() -> String {
    movement::slides(movement::DIAGONAL)
}
