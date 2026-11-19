//! 3F-E03: 냉병기와 화약.
use super::super::{EncounterDefinition, EncounterTier, Placement, RoyalSelection};
use crate::content::pieces::*;

pub const DEFINITION: EncounterDefinition = EncounterDefinition {
    index: 29,
    id: "3F-E03",
    name: "냉병기와 화약",
    description: "말을 탄 선봉대장과 권총 보병 세 기.",
    floor: 3,
    tier: EncounterTier::Elite,
    placements: &[
        Placement::new(MOUNTED_COMMANDER, 4, 0),
        Placement::new(PISTOL_INFANTRY, 2, 1),
        Placement::new(PISTOL_INFANTRY, 4, 1),
        Placement::new(PISTOL_INFANTRY, 6, 1),
    ],
    royal_id: 100,
    additional_royal_ids: &[],
    royal_selection: RoyalSelection::Fixed,
    start_rules: &[],
};
