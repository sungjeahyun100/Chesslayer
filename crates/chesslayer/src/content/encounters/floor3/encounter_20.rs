//! 3F-B01: 포병대대.
use super::super::{EncounterDefinition, EncounterTier, Placement, RoyalSelection, StartRule};
use crate::content::pieces::*;

pub const DEFINITION: EncounterDefinition = EncounterDefinition {
    index: 20,
    id: "3F-B01",
    name: "포병대대",
    description: "소총수·박격포·드론·전차와 창고 두 개. 모든 적 창고를 파괴해야 승리합니다.",
    floor: 3,
    tier: EncounterTier::Boss,
    placements: &[
        Placement::new(WAREHOUSE, 2, 0),
        Placement::new(WAREHOUSE, 5, 0),
        Placement::new(MORTAR, 0, 0),
        Placement::new(DRONE, 1, 0),
        Placement::new(TANK, 3, 0),
        Placement::new(TANK, 4, 0),
        Placement::new(DRONE, 6, 0),
        Placement::new(MORTAR, 7, 0),
        Placement::new(RIFLEMAN, 0, 1),
        Placement::new(RIFLEMAN, 1, 1),
        Placement::new(RIFLEMAN, 2, 1),
        Placement::new(RIFLEMAN, 3, 1),
        Placement::new(RIFLEMAN, 4, 1),
        Placement::new(RIFLEMAN, 5, 1),
        Placement::new(RIFLEMAN, 6, 1),
        Placement::new(RIFLEMAN, 7, 1),
    ],
    royal_id: 100,
    additional_royal_ids: &[101],
    royal_selection: RoyalSelection::Fixed,
    start_rules: &[StartRule::WarehouseObjective],
};
