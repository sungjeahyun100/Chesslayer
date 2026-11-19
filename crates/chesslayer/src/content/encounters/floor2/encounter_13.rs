//! 2F-E01: 템페스트 건어물.
use super::super::{EncounterDefinition, EncounterTier, Placement, RoyalSelection};
use crate::content::pieces::*;

pub const DEFINITION: EncounterDefinition = EncounterDefinition {
    index: 13,
    id: "2F-E01",
    name: "템페스트 건어물",
    description: "흑 기본 진형의 모든 기물이 템페스트 버전입니다. e8의 템페스트 킹이 로얄입니다.",
    floor: 2,
    tier: EncounterTier::Elite,
    placements: &[
        Placement::new(TEMPEST_KING, 4, 0),
        Placement::new(TEMPEST_ROOK, 0, 0),
        Placement::new(TEMPEST_KNIGHT, 1, 0),
        Placement::new(TEMPEST_BISHOP, 2, 0),
        Placement::new(TEMPEST_QUEEN, 3, 0),
        Placement::new(TEMPEST_BISHOP, 5, 0),
        Placement::new(TEMPEST_KNIGHT, 6, 0),
        Placement::new(TEMPEST_ROOK, 7, 0),
        Placement::new(TEMPEST_PAWN, 0, 1),
        Placement::new(TEMPEST_PAWN, 1, 1),
        Placement::new(TEMPEST_PAWN, 2, 1),
        Placement::new(TEMPEST_PAWN, 3, 1),
        Placement::new(TEMPEST_PAWN, 4, 1),
        Placement::new(TEMPEST_PAWN, 5, 1),
        Placement::new(TEMPEST_PAWN, 6, 1),
        Placement::new(TEMPEST_PAWN, 7, 1),
    ],
    royal_id: 100,
    additional_royal_ids: &[],
    royal_selection: RoyalSelection::Fixed,
    start_rules: &[],
};
