//! 2F-N04: 템페스트 폰.
use super::super::{EncounterDefinition, EncounterTier, Placement, RoyalSelection};
use crate::content::pieces::*;

pub const DEFINITION: EncounterDefinition = EncounterDefinition {
    index: 12,
    id: "2F-N04",
    name: "템페스트 폰",
    description: "7랭크의 템페스트 폰 8기 중 전투마다 1기가 로얄입니다.",
    floor: 2,
    tier: EncounterTier::Normal,
    placements: &[
        Placement::new(TEMPEST_PAWN, 0, 1),
        Placement::new(TEMPEST_PAWN, 1, 1),
        Placement::new(TEMPEST_PAWN, 2, 1),
        Placement::new(TEMPEST_PAWN, 3, 1),
        Placement::new(TEMPEST_PAWN, 4, 1),
        Placement::new(TEMPEST_PAWN, 5, 1),
        Placement::new(TEMPEST_PAWN, 6, 1),
        Placement::new(TEMPEST_PAWN, 7, 1),
    ],
    royal_id: 100,
    additional_royal_ids: &[],
    royal_selection: RoyalSelection::UniformDeployed,
    start_rules: &[],
};
