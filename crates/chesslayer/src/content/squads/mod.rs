//! Playable squads own their starting state and player content.
mod guardian;

use super::{
    cards::{self, CardKind},
    pieces,
    relics::Relic,
};
use crate::{battle::PlayerPiece, PieceType};
use serde::Serialize;

/// Add a variant and handle it at the relevant Game event when a squad gains a rule.
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SquadRules {
    Standard,
}

#[derive(Clone, Copy, Serialize)]
pub struct Squad {
    pub id: &'static str,
    pub name: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub relic: Relic,
    #[serde(skip)]
    pub starting_roster: fn() -> Vec<PlayerPiece>,
    #[serde(skip)]
    pub starting_deck: &'static [CardKind],
    #[serde(skip)]
    pub reward_cards: &'static [CardKind],
    #[serde(skip)]
    pub available_pieces: &'static [PieceType],
    #[serde(skip)]
    pub upgrade_paths: &'static [(PieceType, PieceType)],
    #[serde(skip)]
    pub duplicate_relic_card: CardKind,
    pub rules: SquadRules,
    pub unlock_floor: u8,
    #[serde(skip)]
    pub reward_relics: &'static [Relic],
}

impl Squad {
    pub fn validate(self, roster: &[PlayerPiece]) -> Result<(), String> {
        let owns_card = |card: CardKind| card.affiliation().available_to(self.id);
        if !self.starting_deck.iter().copied().all(owns_card)
            || !self.reward_cards.iter().copied().all(owns_card)
            || !owns_card(self.duplicate_relic_card)
            || !cards::SHARED_REWARD_POOL.iter().copied().all(owns_card)
        {
            return Err(format!("{} 부대에 소속되지 않은 카드가 있습니다.", self.id));
        }
        if self.reward_cards.is_empty() {
            return Err(format!("{} 부대의 카드 보상 풀이 너무 작습니다.", self.id));
        }
        if !roster.iter().all(|p| self.allows_piece(p.kind))
            || !self
                .available_pieces
                .iter()
                .all(|&kind| pieces::definition(kind).is_some())
            || !self.upgrade_paths.iter().all(|&(from, to)| {
                self.allows_piece(from)
                    && self.allows_piece(to)
                    && pieces::definition(from)
                        .is_some_and(|d| d.upgrades.iter().any(|u| u.target == to))
            })
        {
            return Err(format!(
                "{} 부대의 기물 또는 강화 경로가 올바르지 않습니다.",
                self.id
            ));
        }
        Ok(())
    }

    pub fn allows_piece(self, kind: PieceType) -> bool {
        self.available_pieces.contains(&kind)
    }
    pub fn allows_upgrade(self, from: PieceType, to: PieceType) -> bool {
        self.allows_piece(from) && self.allows_piece(to) && self.upgrade_paths.contains(&(from, to))
    }
}

pub const ALL: &[Squad] = &[
    guardian::DEFINITION,
    vanguard::DEFINITION,
    heretic::DEFINITION,
    artillery::DEFINITION,
    special::DEFINITION,
];

pub fn squads() -> Vec<Squad> {
    ALL.to_vec()
}

mod artillery;
mod heretic;
mod special;
mod vanguard;

pub fn formation(back: &[PieceType], pawn: Option<PieceType>) -> Vec<PlayerPiece> {
    let mut roster = crate::Battle::standard_roster();
    if pawn.is_none() {
        roster.truncate(8);
    }
    for (i, p) in roster.iter_mut().enumerate() {
        p.kind = if i < 8 { back[i] } else { pawn.unwrap() };
    }
    roster
}
