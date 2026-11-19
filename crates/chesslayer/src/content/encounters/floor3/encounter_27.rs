//! 3F-E01: 보급기지.
use super::super::{EncounterDefinition, EncounterTier, Placement, RoyalSelection, StartRule};
use crate::content::pieces::*;

pub const DEFINITION: EncounterDefinition = EncounterDefinition {
    index: 27,
    id: "3F-E01",
    name: "보급기지",
    description: "창고 두 개를 지키는 박격포병·드론·소총수. 창고를 모두 파괴하면 승리합니다.",
    floor: 3,
    tier: EncounterTier::Elite,
    placements: &[
        Placement::new(WAREHOUSE, 2, 0),
        Placement::new(WAREHOUSE, 5, 0),
        Placement::new(MORTAR, 0, 0),
        Placement::new(MORTAR, 7, 0),
        Placement::new(DRONE, 1, 1),
        Placement::new(DRONE, 6, 1),
        Placement::new(RIFLEMAN, 3, 1),
        Placement::new(RIFLEMAN, 4, 1),
    ],
    royal_id: 100,
    additional_royal_ids: &[101],
    royal_selection: RoyalSelection::Fixed,
    start_rules: &[StartRule::WarehouseObjective],
};
