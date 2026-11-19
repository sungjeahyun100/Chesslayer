//! 1F-N02: 왕관을 쓴 기사.
use super::super::{EncounterDefinition, EncounterTier, Placement, RoyalSelection};
use crate::content::pieces::*;

pub const DEFINITION: EncounterDefinition = EncounterDefinition {
    index: 1,
    id: "1F-N02",
    name: "왕관을 쓴 기사",
    description: "e8의 로얄 나이트 1기.",
    floor: 1,
    tier: EncounterTier::Normal,
    placements: &[Placement::new(KNIGHT, 4, 0)],
    royal_id: 100,
    additional_royal_ids: &[],
    royal_selection: RoyalSelection::Fixed,
    start_rules: &[],
};
