//! 1F-B01: 구행.
use super::super::{EncounterDefinition, EncounterTier, Placement, RoyalSelection};
use crate::content::pieces::*;

pub const DEFINITION: EncounterDefinition = EncounterDefinition {
    index: 8,
    id: "1F-B01",
    name: "구행",
    description: "e8의 로얄 구행 1기. 직선 경로의 빈 칸에서 한 번 직각으로 꺾을 수 있습니다.",
    floor: 1,
    tier: EncounterTier::Boss,
    placements: &[Placement::new(HOOK, 4, 0)],
    royal_id: 100,
    additional_royal_ids: &[],
    royal_selection: RoyalSelection::Fixed,
    start_rules: &[],
};
