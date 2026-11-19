//! 4F-E02: 명사수.
use super::super::{EncounterDefinition, EncounterTier, Placement, RoyalSelection};
use crate::content::pieces::*;

pub const DEFINITION: EncounterDefinition = EncounterDefinition {
    index: 35,
    id: "4F-E02",
    name: "명사수",
    description: "그림자 지휘관과 명사수 네 기.",
    floor: 4,
    tier: EncounterTier::Elite,
    placements: &[
        Placement::new(SHADOW_COMMANDER, 4, 0),
        Placement::new(MARKSMAN, 0, 1),
        Placement::new(MARKSMAN, 2, 1),
        Placement::new(MARKSMAN, 5, 1),
        Placement::new(MARKSMAN, 7, 1),
    ],
    royal_id: 100,
    additional_royal_ids: &[],
    royal_selection: RoyalSelection::Fixed,
    start_rules: &[],
};
