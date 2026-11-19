//! 4F-N01: 정찰조.
use super::super::{EncounterDefinition, EncounterTier, Placement, RoyalSelection};
use crate::content::pieces::*;

pub const DEFINITION: EncounterDefinition = EncounterDefinition {
    index: 30,
    id: "4F-N01",
    name: "정찰조",
    description: "세 정찰병의 나이트 도약.",
    floor: 4,
    tier: EncounterTier::Normal,
    placements: &[
        Placement::new(SCOUT, 4, 0),
        Placement::new(SCOUT, 1, 1),
        Placement::new(SCOUT, 6, 1),
    ],
    royal_id: 100,
    additional_royal_ids: &[],
    royal_selection: RoyalSelection::Fixed,
    start_rules: &[],
};
