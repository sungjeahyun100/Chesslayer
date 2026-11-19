//! 4F-B01: 특수부대.
use super::super::{EncounterDefinition, EncounterTier, Placement, RoyalSelection};
use crate::content::pieces::*;

pub const DEFINITION: EncounterDefinition = EncounterDefinition {
    index: 21,
    id: "4F-B01",
    name: "특수부대",
    description: "여덟 정예대원의 저격·도약 전술. 그림자 지휘관을 직접 포획해야 승리합니다.",
    floor: 4,
    tier: EncounterTier::Boss,
    placements: &[
        Placement::new(SHADOW_COMMANDER, 4, 0),
        Placement::new(MARKSMAN, 0, 0),
        Placement::new(PHANTOM_SCOUT, 1, 0),
        Placement::new(ELITE_COMMANDO, 2, 0),
        Placement::new(ELITE_COMMANDO, 3, 0),
        Placement::new(ELITE_COMMANDO, 5, 0),
        Placement::new(PHANTOM_SCOUT, 6, 0),
        Placement::new(MARKSMAN, 7, 0),
    ],
    royal_id: 100,
    additional_royal_ids: &[],
    royal_selection: RoyalSelection::Fixed,
    start_rules: &[],
};
