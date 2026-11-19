//! 4F-N04: 비밀 지휘소.
use super::super::{EncounterDefinition, EncounterTier, Placement, RoyalSelection};
use crate::content::pieces::*;

pub const DEFINITION: EncounterDefinition = EncounterDefinition {
    index: 33,
    id: "4F-N04",
    name: "비밀 지휘소",
    description: "가호를 가진 로얄 지휘관과 두 정찰병.",
    floor: 4,
    tier: EncounterTier::Normal,
    placements: &[
        Placement::new(SPECIAL_COMMANDER, 4, 0),
        Placement::new(SCOUT, 1, 1),
        Placement::new(SCOUT, 6, 1),
    ],
    royal_id: 100,
    additional_royal_ids: &[],
    royal_selection: RoyalSelection::Fixed,
    start_rules: &[],
};
