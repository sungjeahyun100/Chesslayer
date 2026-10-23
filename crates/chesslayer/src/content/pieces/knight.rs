use super::{movement, PieceDefinition};
use crate::PieceType;

pub const KNIGHT: PieceType = 2;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 30,
    glyphs: ["♘", "♞"],
    ..PieceDefinition::new(KNIGHT, "knight", "나이트", movement)
};

fn movement() -> String {
    movement::steps(movement::KNIGHT)
}
