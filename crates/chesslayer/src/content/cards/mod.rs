//! Card IDs, explicit registration and shared acquisition lists.
mod fleeting_ward;
mod gun;
mod jeans;
pub mod ward;

use crate::{cards::Card, Battle, PieceId};
use serde::Serialize;

pub use gun::{GUN_ADJACENT_FORMATION, GUN_HALF_WIDTH, GUN_HITS_ALLIES};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CardKind {
    Jeans,
    Ward,
    Gun,
    FleetingWard,
}
impl CardKind {
    pub fn definition(self) -> CardDefinition {
        match self {
            Self::Jeans => jeans::definition(),
            Self::Ward => ward::definition(),
            Self::Gun => gun::definition(),
            Self::FleetingWard => fleeting_ward::definition(),
        }
    }
}

/// Effects validate before mutation. Some(count) adds the shared removal log suffix.
pub type CardEffect = fn(&mut Battle, Option<PieceId>) -> Result<Option<usize>, String>;
pub type CardTargets = fn(&Battle) -> Vec<PieceId>;

#[derive(Clone, Serialize)]
pub struct CardDefinition {
    pub kind: CardKind,
    pub name: &'static str,
    pub cost: u8,
    pub description: &'static str,
    pub targeted: bool,
    pub exhaust: bool,
    pub retain: bool,
    pub innate: bool,
    pub ethereal: bool,
    #[serde(skip)]
    pub effect: CardEffect,
    #[serde(skip)]
    pub target_candidates: CardTargets,
}
impl CardDefinition {
    /// Keywords default to false; untargeted cards have no target candidates.
    pub fn new(
        kind: CardKind,
        name: &'static str,
        cost: u8,
        description: &'static str,
        effect: CardEffect,
    ) -> Self {
        Self {
            kind,
            name,
            cost,
            description,
            effect,
            targeted: false,
            exhaust: false,
            retain: false,
            innate: false,
            ethereal: false,
            target_candidates: |_| vec![],
        }
    }
    pub fn with_targets(mut self, targets: CardTargets) -> Self {
        self.targeted = true;
        self.target_candidates = targets;
        self
    }
}

/// Preserve reward indices: saved command fixtures use these positions.
pub const REWARD_POOL: &[CardKind] = &[CardKind::Ward, CardKind::Jeans, CardKind::Gun];

pub fn starter_deck() -> Vec<Card> {
    use CardKind::*;
    [
        Jeans,
        Ward,
        Ward,
        Ward,
        Ward,
        Ward,
        Ward,
        Gun,
        FleetingWard,
        FleetingWard,
    ]
    .into_iter()
    .enumerate()
    .map(|(i, kind)| Card {
        id: i as u32 + 1,
        kind,
    })
    .collect()
}
