//! 4F-E03: 침투 작전.
use super::super::{EncounterDefinition, EncounterTier, Placement, RoyalSelection};
use crate::content::pieces::*;

pub const DEFINITION: EncounterDefinition = EncounterDefinition {
    index: 36,
    id: "4F-E03",
    name: "침투 작전",
    description: "그림자 지휘관·명사수·정예 특전대원·유령 정찰병의 합동작전.",
    floor: 4,
    tier: EncounterTier::Elite,
    placements: &[
        Placement::new(SHADOW_COMMANDER, 4, 0),
        Placement::new(MARKSMAN, 0, 0),
        Placement::new(MARKSMAN, 7, 0),
        Placement::new(ELITE_COMMANDO, 2, 1),
        Placement::new(ELITE_COMMANDO, 5, 1),
        Placement::new(PHANTOM_SCOUT, 3, 1),
        Placement::new(PHANTOM_SCOUT, 4, 1),
    ],
    royal_id: 100,
    additional_royal_ids: &[],
    royal_selection: RoyalSelection::Fixed,
    start_rules: &[],
};
