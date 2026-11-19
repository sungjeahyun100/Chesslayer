//! 1F-E01: 마하라자.
use super::super::{EncounterDefinition, EncounterTier, Placement, RoyalSelection};
use crate::content::pieces::*;

pub const DEFINITION: EncounterDefinition = EncounterDefinition {
    index: 4,
    id: "1F-E01",
    name: "마하라자",
    description: "e8의 로얄 아마존 1기. 퀸과 나이트의 행마를 사용합니다.",
    floor: 1,
    tier: EncounterTier::Elite,
    placements: &[Placement::new(AMAZON, 4, 0)],
    royal_id: 100,
    additional_royal_ids: &[],
    royal_selection: RoyalSelection::Fixed,
    start_rules: &[],
};
