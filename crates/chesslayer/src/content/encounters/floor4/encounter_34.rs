//! 4F-E01: 유령 부대.
use super::super::{EncounterDefinition, EncounterTier, Placement, RoyalSelection};
use crate::content::pieces::*;

pub const DEFINITION: EncounterDefinition = EncounterDefinition {
    index: 34,
    id: "4F-E01",
    name: "유령 부대",
    description: "그림자 지휘관과 유령 정찰병 네 기.",
    floor: 4,
    tier: EncounterTier::Elite,
    placements: &[
        Placement::new(SHADOW_COMMANDER, 4, 0),
        Placement::new(PHANTOM_SCOUT, 0, 1),
        Placement::new(PHANTOM_SCOUT, 2, 1),
        Placement::new(PHANTOM_SCOUT, 5, 1),
        Placement::new(PHANTOM_SCOUT, 7, 1),
    ],
    royal_id: 100,
    additional_royal_ids: &[],
    royal_selection: RoyalSelection::Fixed,
    start_rules: &[],
};
