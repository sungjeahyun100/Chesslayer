//! 3F-E02: 기동 포대.
use super::super::{EncounterDefinition, EncounterTier, Placement, RoyalSelection};
use crate::content::pieces::*;

pub const DEFINITION: EncounterDefinition = EncounterDefinition {
    index: 28,
    id: "3F-E02",
    name: "기동 포대",
    description: "로얄 전차와 드론 네 기가 기동전을 펼칩니다.",
    floor: 3,
    tier: EncounterTier::Elite,
    placements: &[
        Placement::new(TANK, 4, 0),
        Placement::new(DRONE, 1, 1),
        Placement::new(DRONE, 2, 1),
        Placement::new(DRONE, 5, 1),
        Placement::new(DRONE, 6, 1),
    ],
    royal_id: 100,
    additional_royal_ids: &[],
    royal_selection: RoyalSelection::Fixed,
    start_rules: &[],
};
