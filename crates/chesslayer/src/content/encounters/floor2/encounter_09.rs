//! 2F-N01: 아마존.
use super::super::{EncounterDefinition, EncounterTier, Placement, RoyalSelection};
use crate::content::pieces::*;

pub const DEFINITION: EncounterDefinition = EncounterDefinition {
    index: 9,
    id: "2F-N01",
    name: "아마존",
    description: "e8의 로얄 아마존 1기.",
    floor: 2,
    tier: EncounterTier::Normal,
    placements: &[Placement::new(AMAZON, 4, 0)],
    royal_id: 100,
    additional_royal_ids: &[],
    royal_selection: RoyalSelection::Fixed,
    start_rules: &[],
};
