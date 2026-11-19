//! 4F-N02: 교차 사선.
use super::super::{EncounterDefinition, EncounterTier, Placement, RoyalSelection};
use crate::content::pieces::*;

pub const DEFINITION: EncounterDefinition = EncounterDefinition {
    index: 31,
    id: "4F-N02",
    name: "교차 사선",
    description: "서로 다른 사선을 지키는 저격수 세 기.",
    floor: 4,
    tier: EncounterTier::Normal,
    placements: &[
        Placement::new(SNIPER, 4, 0),
        Placement::new(SNIPER, 0, 1),
        Placement::new(SNIPER, 7, 1),
    ],
    royal_id: 100,
    additional_royal_ids: &[],
    royal_selection: RoyalSelection::Fixed,
    start_rules: &[],
};
