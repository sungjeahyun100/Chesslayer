//! 2F-B02: 쌍둥이 신.
use super::super::{EncounterDefinition, EncounterTier, Placement, RoyalSelection};
use crate::content::pieces::*;

pub const DEFINITION: EncounterDefinition = EncounterDefinition {
    index: 17,
    id: "2F-B02",
    name: "쌍둥이 신",
    description: "e8·d8의 행마의 신이 모두 로얄이며 둘 다 포획해야 승리합니다.",
    floor: 2,
    tier: EncounterTier::Boss,
    placements: &[
        Placement::new(MOVEMENT_GOD, 4, 0),
        Placement::new(MOVEMENT_GOD, 3, 0),
    ],
    royal_id: 100,
    additional_royal_ids: &[101],
    royal_selection: RoyalSelection::Fixed,
    start_rules: &[],
};
