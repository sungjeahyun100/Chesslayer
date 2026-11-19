//! 2F-E03: 몽골군.
use super::super::{EncounterDefinition, EncounterTier, Placement, RoyalSelection};
use crate::content::pieces::*;

pub const DEFINITION: EncounterDefinition = EncounterDefinition {
    index: 15,
    id: "2F-E03",
    name: "몽골군",
    description: "e8의 로얄 켄타우로스와 8·7랭크의 나이트 15기.",
    floor: 2,
    tier: EncounterTier::Elite,
    placements: &[
        Placement::new(CENTAUR, 4, 0),
        Placement::new(KNIGHT, 0, 0),
        Placement::new(KNIGHT, 1, 0),
        Placement::new(KNIGHT, 2, 0),
        Placement::new(KNIGHT, 3, 0),
        Placement::new(KNIGHT, 5, 0),
        Placement::new(KNIGHT, 6, 0),
        Placement::new(KNIGHT, 7, 0),
        Placement::new(KNIGHT, 0, 1),
        Placement::new(KNIGHT, 1, 1),
        Placement::new(KNIGHT, 2, 1),
        Placement::new(KNIGHT, 3, 1),
        Placement::new(KNIGHT, 4, 1),
        Placement::new(KNIGHT, 5, 1),
        Placement::new(KNIGHT, 6, 1),
        Placement::new(KNIGHT, 7, 1),
    ],
    royal_id: 100,
    additional_royal_ids: &[],
    royal_selection: RoyalSelection::Fixed,
    start_rules: &[],
};
