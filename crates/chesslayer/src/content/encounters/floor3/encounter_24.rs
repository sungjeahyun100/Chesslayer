//! 3F-N02: 드론 순찰.
use super::super::{EncounterDefinition, EncounterTier, Placement, RoyalSelection};
use crate::content::pieces::*;

pub const DEFINITION: EncounterDefinition = EncounterDefinition {
    index: 24,
    id: "3F-N02",
    name: "드론 순찰",
    description: "세 드론이 도약과 근거리 사격으로 전장을 순찰합니다.",
    floor: 3,
    tier: EncounterTier::Normal,
    placements: &[
        Placement::new(DRONE, 4, 0),
        Placement::new(DRONE, 1, 1),
        Placement::new(DRONE, 6, 1),
    ],
    royal_id: 100,
    additional_royal_ids: &[],
    royal_selection: RoyalSelection::Fixed,
    start_rules: &[],
};
