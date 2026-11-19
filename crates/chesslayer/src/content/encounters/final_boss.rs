//! FINAL-B01: 부족의 독재자.
use super::{EncounterDefinition, EncounterTier, Placement, RoyalSelection, StartRule};
use crate::content::pieces::*;

pub const DEFINITION: EncounterDefinition = EncounterDefinition {
    index: 22,
    id: "FINAL-B01",
    name: "부족의 독재자",
    description: "4층 완료 후 별도로 도전하는 최종 결전. 독재자 한 기와 싸웁니다.",
    floor: 4,
    tier: EncounterTier::Boss,
    placements: &[Placement::new(TRIBAL_DICTATOR, 4, 0)],
    royal_id: 100,
    additional_royal_ids: &[],
    royal_selection: RoyalSelection::Fixed,
    start_rules: &[StartRule::FinalBoss],
};
