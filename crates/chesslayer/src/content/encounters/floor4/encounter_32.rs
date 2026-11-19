//! 4F-N03: 돌격 분대.
use super::super::{EncounterDefinition, EncounterTier, Placement, RoyalSelection};
use crate::content::pieces::*;

pub const DEFINITION: EncounterDefinition = EncounterDefinition {
    index: 32,
    id: "4F-N03",
    name: "돌격 분대",
    description: "전방 도약과 사격을 사용하는 특전대원 네 기.",
    floor: 4,
    tier: EncounterTier::Normal,
    placements: &[
        Placement::new(COMMANDO, 4, 0),
        Placement::new(COMMANDO, 1, 1),
        Placement::new(COMMANDO, 3, 1),
        Placement::new(COMMANDO, 6, 1),
    ],
    royal_id: 100,
    additional_royal_ids: &[],
    royal_selection: RoyalSelection::Fixed,
    start_rules: &[],
};
