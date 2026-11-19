//! 2F-N03: 랍스타.
use super::super::{EncounterDefinition, EncounterTier, Placement, RoyalSelection};
use crate::content::pieces::*;

pub const DEFINITION: EncounterDefinition = EncounterDefinition {
    index: 11,
    id: "2F-N03",
    name: "랍스타",
    description: "a8·b8·g8·h8의 랍스타 4기 중 전투마다 1기가 로얄입니다.",
    floor: 2,
    tier: EncounterTier::Normal,
    placements: &[
        Placement::new(LOBSTER, 0, 0),
        Placement::new(LOBSTER, 1, 0),
        Placement::new(LOBSTER, 6, 0),
        Placement::new(LOBSTER, 7, 0),
    ],
    royal_id: 100,
    additional_royal_ids: &[],
    royal_selection: RoyalSelection::UniformDeployed,
    start_rules: &[],
};
