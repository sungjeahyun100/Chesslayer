//! 1F-B02: 선봉대.
use super::super::{EncounterDefinition, EncounterTier, Placement, RoyalSelection, StartRule};
use crate::content::pieces::*;

pub const DEFINITION: EncounterDefinition = EncounterDefinition {
    index: 18,
    id: "1F-B02",
    name: "선봉대",
    description: "민첩한 보병 16기와 선봉대장. 적 부대는 한 턴에 두 번 행동합니다.",
    floor: 1,
    tier: EncounterTier::Boss,
    placements: &[
        Placement::new(VANGUARD_COMMANDER, 4, 0),
        Placement::new(SPEAR_INFANTRY, 0, 0),
        Placement::new(AXE_INFANTRY, 1, 0),
        Placement::new(SWORD_INFANTRY, 2, 0),
        Placement::new(VANGUARD_OFFICER, 3, 0),
        Placement::new(SWORD_INFANTRY, 5, 0),
        Placement::new(AXE_INFANTRY, 6, 0),
        Placement::new(SPEAR_INFANTRY, 7, 0),
        Placement::new(AXE_INFANTRY, 0, 1),
        Placement::new(AXE_INFANTRY, 1, 1),
        Placement::new(AXE_INFANTRY, 2, 1),
        Placement::new(AXE_INFANTRY, 3, 1),
        Placement::new(AXE_INFANTRY, 4, 1),
        Placement::new(AXE_INFANTRY, 5, 1),
        Placement::new(AXE_INFANTRY, 6, 1),
        Placement::new(AXE_INFANTRY, 7, 1),
    ],
    royal_id: 100,
    additional_royal_ids: &[],
    royal_selection: RoyalSelection::Fixed,
    start_rules: &[StartRule::EnemyDoubleActions],
};
