//! 3F-N03: 박격포 진지.
use super::super::{EncounterDefinition, EncounterTier, Placement, RoyalSelection};
use crate::content::pieces::*;

pub const DEFINITION: EncounterDefinition = EncounterDefinition {
    index: 25,
    id: "3F-N03",
    name: "박격포 진지",
    description: "박격포병과 소총수 두 기가 장애물 너머를 포격합니다.",
    floor: 3,
    tier: EncounterTier::Normal,
    placements: &[
        Placement::new(MORTAR, 4, 0),
        Placement::new(RIFLEMAN, 3, 1),
        Placement::new(RIFLEMAN, 5, 1),
    ],
    royal_id: 100,
    additional_royal_ids: &[],
    royal_selection: RoyalSelection::Fixed,
    start_rules: &[],
};
