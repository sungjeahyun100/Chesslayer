use super::{movement, PieceDefinition};
use crate::PieceType;

pub const AMAZON: PieceType = 7;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 90,
    glyphs: ["♕", "♛"],
    badge: Some("♞"),
    ..PieceDefinition::new(AMAZON, "amazon", "아마존", movement)
};

fn movement() -> String {
    format!(
        "{};{}",
        movement::slides(&movement::all_directions()),
        movement::steps(movement::KNIGHT)
    )
}
