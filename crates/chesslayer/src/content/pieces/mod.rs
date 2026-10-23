//! Stable piece IDs and their definitions. List each new definition once in ALL.
mod amazon;
mod bishop;
mod king;
mod knight;
pub mod movement;
mod pawn;
mod queen;
mod rook;
mod tempest_pawn;

use crate::PieceType;

pub use amazon::AMAZON;
pub use bishop::BISHOP;
pub use king::KING;
pub use knight::KNIGHT;
pub use pawn::PAWN;
pub use queen::QUEEN;
pub use rook::ROOK;
pub use tempest_pawn::TEMPEST_PAWN;

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
    pub upgrade: Option<UpgradeDefinition>,
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
            upgrade: None,
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
