use super::{movement, PieceDefinition};
use crate::PieceType;

pub const ROOK: PieceType = 4;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 50,
    glyphs: ["♖", "♜"],
    ..PieceDefinition::new(ROOK, "rook", "룩", movement)
};

fn movement() -> String {
    movement::slides(movement::ORTHOGONAL)
}
