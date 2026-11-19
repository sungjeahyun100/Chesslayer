//! 3F-N01: 소총 초소.
use super::super::{EncounterDefinition, EncounterTier, Placement, RoyalSelection};
use crate::content::pieces::*;

pub const DEFINITION: EncounterDefinition = EncounterDefinition {
    index: 23,
    id: "3F-N01",
    name: "소총 초소",
    description: "세 소총수가 전방 사격으로 진입로를 지킵니다.",
    floor: 3,
    tier: EncounterTier::Normal,
    placements: &[
        Placement::new(RIFLEMAN, 4, 1),
        Placement::new(RIFLEMAN, 2, 1),
        Placement::new(RIFLEMAN, 6, 1),
    ],
    royal_id: 100,
    additional_royal_ids: &[],
    royal_selection: RoyalSelection::Fixed,
    start_rules: &[],
};
