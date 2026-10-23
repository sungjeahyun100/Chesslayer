use super::{movement, PieceDefinition};
use crate::PieceType;

pub const KING: PieceType = 6;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 10,
    glyphs: ["♔", "♚"],
    ..PieceDefinition::new(KING, "king", "킹", movement)
};

fn movement() -> String {
    movement::steps(&movement::all_directions())
}
