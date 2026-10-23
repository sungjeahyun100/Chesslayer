//! Stable piece IDs and their definitions. List each new definition once in ALL.
mod amazon;
mod bishop;
mod bouncing_bishop;
mod bouncing_pawn;
mod bouncing_queen;
mod bouncing_rook;
mod king;
mod knight;
pub mod movement;
mod pawn;
mod queen;
mod rook;
mod tempest_bishop;
mod tempest_knight;
mod tempest_pawn;
mod tempest_queen;
mod tempest_rook;

use crate::PieceType;

pub use amazon::AMAZON;
pub use bishop::BISHOP;
pub use bouncing_bishop::BOUNCING_BISHOP;
pub use bouncing_pawn::BOUNCING_PAWN;
pub use bouncing_queen::BOUNCING_QUEEN;
pub use bouncing_rook::BOUNCING_ROOK;
pub use king::KING;
pub use knight::KNIGHT;
pub use pawn::PAWN;
pub use queen::QUEEN;
pub use rook::ROOK;
pub use tempest_bishop::TEMPEST_BISHOP;
pub use tempest_knight::TEMPEST_KNIGHT;
pub use tempest_pawn::TEMPEST_PAWN;
pub use tempest_queen::TEMPEST_QUEEN;
pub use tempest_rook::TEMPEST_ROOK;

pub struct UpgradeDefinition {
    pub target: PieceType,
    pub key: &'static str,
    pub effect: &'static str,
    pub requirement: &'static str,
}

pub struct PieceDefinition {
    pub kind: PieceType,
    pub key: &'static str,
    pub name: &'static str,
    pub movement: fn() -> String,
    pub value: i32,
    pub description: &'static str,
    pub upgrades: &'static [UpgradeDefinition],
    pub promotions: &'static [PieceType],
    pub initial_double_step: bool,
}
impl PieceDefinition {
    pub const fn new(
        kind: PieceType,
        key: &'static str,
        name: &'static str,
        movement: fn() -> String,
    ) -> Self {
        Self {
            kind,
            key,
            name,
            movement,
            value: 10,
            description: "",
            upgrades: &[],
            promotions: &[],
            initial_double_step: false,
        }
    }
}

pub const ALL: &[PieceDefinition] = &[
    pawn::DEFINITION,
    knight::DEFINITION,
    bishop::DEFINITION,
    rook::DEFINITION,
    queen::DEFINITION,
    king::DEFINITION,
    amazon::DEFINITION,
    tempest_pawn::DEFINITION,
    tempest_rook::DEFINITION,
    tempest_bishop::DEFINITION,
    tempest_knight::DEFINITION,
    tempest_queen::DEFINITION,
    bouncing_pawn::DEFINITION,
    bouncing_rook::DEFINITION,
    bouncing_bishop::DEFINITION,
    bouncing_queen::DEFINITION,
];

pub fn definition(kind: PieceType) -> Option<&'static PieceDefinition> {
    ALL.iter().find(|d| d.kind == kind)
}

/// Board and Chesssembly index catalog names by numeric kind; zero is reserved.
pub(crate) fn catalog() -> Vec<String> {
    let max = ALL.iter().map(|d| d.kind).max().unwrap_or(0);
    let mut names = vec![String::new(); usize::from(max) + 1];
    for d in ALL {
        names[usize::from(d.kind)] = d.key.into();
    }
    names
}

pub fn piece_name(kind: PieceType) -> &'static str {
    definition(kind).map_or("기물", |d| d.name)
}
