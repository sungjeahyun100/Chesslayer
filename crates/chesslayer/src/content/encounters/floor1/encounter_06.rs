//! 1F-E03: 건어물.
use super::super::{EncounterDefinition, EncounterTier, Placement, RoyalSelection};
use crate::content::pieces::*;

pub const DEFINITION: EncounterDefinition = EncounterDefinition {
    index: 6,
    id: "1F-E03",
    name: "건어물",
    description: "표준 체스 초기 배치의 흑 16기. e8의 킹이 로얄입니다.",
    floor: 1,
    tier: EncounterTier::Elite,
    placements: &[
        Placement::new(KING, 4, 0),
        Placement::new(ROOK, 0, 0),
        Placement::new(KNIGHT, 1, 0),
        Placement::new(BISHOP, 2, 0),
        Placement::new(QUEEN, 3, 0),
        Placement::new(BISHOP, 5, 0),
        Placement::new(KNIGHT, 6, 0),
        Placement::new(ROOK, 7, 0),
        Placement::new(PAWN, 0, 1),
        Placement::new(PAWN, 1, 1),
        Placement::new(PAWN, 2, 1),
        Placement::new(PAWN, 3, 1),
        Placement::new(PAWN, 4, 1),
        Placement::new(PAWN, 5, 1),
        Placement::new(PAWN, 6, 1),
        Placement::new(PAWN, 7, 1),
    ],
    royal_id: 100,
    additional_royal_ids: &[],
    royal_selection: RoyalSelection::Fixed,
    start_rules: &[],
};
