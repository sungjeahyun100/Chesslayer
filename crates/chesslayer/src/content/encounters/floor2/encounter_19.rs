//! 2F-B03: 이단 부대 · 영원한 주빈.
use super::super::{EncounterDefinition, EncounterTier, Placement, RoyalSelection};
use crate::content::pieces::*;

pub const DEFINITION: EncounterDefinition = EncounterDefinition {
    index: 19,
    id: "2F-B03",
    name: "이단 부대 · 영원한 주빈",
    description: "강화된 살아 있는 기물 16기와 d6·f6의 시체 2기. 주빈을 직접 포획하면 승리합니다.",
    floor: 2,
    tier: EncounterTier::Boss,
    placements: &[
        Placement::new(ETERNAL_GUEST, 4, 0),
        Placement::new(FUNERAL_CART, 0, 0),
        Placement::new(MASKED_DANCER, 1, 0),
        Placement::new(INTOXICATED_PRIEST, 2, 0),
        Placement::new(BANQUET_LADY, 3, 0),
        Placement::new(FEAST_PRIEST, 5, 0),
        Placement::new(TEMPTING_DANCER, 6, 0),
        Placement::new(FUNERAL_CART, 7, 0),
        Placement::new(PROCESSION_FOLLOWER, 0, 1),
        Placement::new(HUNGRY_FOLLOWER, 1, 1),
        Placement::new(PROCESSION_FOLLOWER, 2, 1),
        Placement::new(INVITING_FOLLOWER, 3, 1),
        Placement::new(PROCESSION_FOLLOWER, 4, 1),
        Placement::new(HUNGRY_FOLLOWER, 5, 1),
        Placement::new(PROCESSION_FOLLOWER, 6, 1),
        Placement::new(PROCESSION_FOLLOWER, 7, 1),
        Placement::new(CORPSE, 3, 2),
        Placement::new(CORPSE, 5, 2),
    ],
    royal_id: 100,
    additional_royal_ids: &[],
    royal_selection: RoyalSelection::Fixed,
    start_rules: &[],
};
