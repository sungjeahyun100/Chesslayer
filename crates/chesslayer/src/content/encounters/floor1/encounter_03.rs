//! 1F-N04: 폰의 성벽.
use super::super::{EncounterDefinition, EncounterTier, Placement, RoyalSelection};
use crate::content::pieces::*;

pub const DEFINITION: EncounterDefinition = EncounterDefinition {
    index: 3,
    id: "1F-N04",
    name: "폰의 성벽",
    description: "e8의 로얄 킹과 7랭크의 폰 8기.",
    floor: 1,
    tier: EncounterTier::Normal,
    placements: &[
        Placement::new(KING, 4, 0),
        Placement::new(PAWN, 0, 1),
        Placement::new(PAWN, 1, 1),
        Placement::new(PAWN, 2, 1),
        Placement::new(PAWN, 3, 1),
        Placement::new(PAWN, 4, 1),
        Placement::new(PAWN, 5, 1),
        Placement::new(PAWN, 6, 1),
        Placement::new(PAWN, 7, 1),
    ],
    royal_id: 100,
    additional_royal_ids: &[],
    royal_selection: RoyalSelection::Fixed,
    start_rules: &[],
};
