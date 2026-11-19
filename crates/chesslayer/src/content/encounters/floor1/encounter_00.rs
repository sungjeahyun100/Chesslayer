//! 1F-N01: 홀로 선 왕.
use super::super::{EncounterDefinition, EncounterTier, Placement, RoyalSelection};
use crate::content::pieces::*;

pub const DEFINITION: EncounterDefinition = EncounterDefinition {
    index: 0,
    id: "1F-N01",
    name: "홀로 선 왕",
    description: "e8의 로얄 킹 1기.",
    floor: 1,
    tier: EncounterTier::Normal,
    placements: &[Placement::new(KING, 4, 0)],
    royal_id: 100,
    additional_royal_ids: &[],
    royal_selection: RoyalSelection::Fixed,
    start_rules: &[],
};
