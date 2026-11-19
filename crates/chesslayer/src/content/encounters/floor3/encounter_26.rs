//! 3F-N04: 전차 호위.
use super::super::{EncounterDefinition, EncounterTier, Placement, RoyalSelection};
use crate::content::pieces::*;

pub const DEFINITION: EncounterDefinition = EncounterDefinition {
    index: 26,
    id: "3F-N04",
    name: "전차 호위",
    description: "로얄 전차와 소총수 호위 두 기.",
    floor: 3,
    tier: EncounterTier::Normal,
    placements: &[
        Placement::new(TANK, 4, 0),
        Placement::new(RIFLEMAN, 3, 1),
        Placement::new(RIFLEMAN, 5, 1),
    ],
    royal_id: 100,
    additional_royal_ids: &[],
    royal_selection: RoyalSelection::Fixed,
    start_rules: &[],
};
