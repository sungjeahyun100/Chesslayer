//! 1F-E04: 런던.
use super::super::{EncounterDefinition, EncounterTier, Placement, RoyalSelection};
use crate::content::pieces::*;

pub const DEFINITION: EncounterDefinition = EncounterDefinition {
    index: 7,
    id: "1F-E04",
    name: "런던",
    description: "흑 런던 시스템 진형 16기. e8의 킹이 로얄입니다.",
    floor: 1,
    tier: EncounterTier::Elite,
    placements: &[
        Placement::new(KING, 4, 0),
        Placement::new(QUEEN, 3, 0),
        Placement::new(ROOK, 0, 0),
        Placement::new(ROOK, 7, 0),
        Placement::new(BISHOP, 3, 2),
        Placement::new(BISHOP, 5, 3),
        Placement::new(KNIGHT, 3, 1),
        Placement::new(KNIGHT, 5, 2),
        Placement::new(PAWN, 0, 1),
        Placement::new(PAWN, 1, 1),
        Placement::new(PAWN, 2, 2),
        Placement::new(PAWN, 3, 3),
        Placement::new(PAWN, 4, 2),
        Placement::new(PAWN, 5, 1),
        Placement::new(PAWN, 6, 1),
        Placement::new(PAWN, 7, 1),
    ],
    royal_id: 100,
    additional_royal_ids: &[],
    royal_selection: RoyalSelection::Fixed,
    start_rules: &[],
};
