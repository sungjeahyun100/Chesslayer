//! 2F-E02: 제왕.
use super::super::{EncounterDefinition, EncounterTier, Placement, RoyalSelection, StartRule};
use crate::content::pieces::*;

pub const DEFINITION: EncounterDefinition = EncounterDefinition {
    index: 14,
    id: "2F-E02",
    name: "제왕",
    description: "e8의 로얄 제왕. 포획한 기물의 행마법을 흡수하며 선공권을 가집니다.",
    floor: 2,
    tier: EncounterTier::Elite,
    placements: &[Placement::new(EMPEROR, 4, 0)],
    royal_id: 100,
    additional_royal_ids: &[],
    royal_selection: RoyalSelection::Fixed,
    start_rules: &[StartRule::EnemyInitiative],
};
