//! 2F-N02: 유니콘.
use super::super::{EncounterDefinition, EncounterTier, Placement, RoyalSelection};
use crate::content::pieces::*;

pub const DEFINITION: EncounterDefinition = EncounterDefinition {
    index: 10,
    id: "2F-N02",
    name: "유니콘",
    description: "e8의 로얄 유니콘 1기. 나이트 행마와 빈칸 순간이동을 사용합니다.",
    floor: 2,
    tier: EncounterTier::Normal,
    placements: &[Placement::new(UNICORN, 4, 0)],
    royal_id: 100,
    additional_royal_ids: &[],
    royal_selection: RoyalSelection::Fixed,
    start_rules: &[],
};
