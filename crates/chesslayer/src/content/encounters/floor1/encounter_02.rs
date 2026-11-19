//! 1F-N03: 쌍둥이 주교.
use super::super::{EncounterDefinition, EncounterTier, Placement, RoyalSelection};
use crate::content::pieces::*;

pub const DEFINITION: EncounterDefinition = EncounterDefinition {
    index: 2,
    id: "1F-N03",
    name: "쌍둥이 주교",
    description: "e8의 로얄 킹과 c8·f8의 비숍.",
    floor: 1,
    tier: EncounterTier::Normal,
    placements: &[
        Placement::new(KING, 4, 0),
        Placement::new(BISHOP, 2, 0),
        Placement::new(BISHOP, 5, 0),
    ],
    royal_id: 100,
    additional_royal_ids: &[],
    royal_selection: RoyalSelection::Fixed,
    start_rules: &[],
};
