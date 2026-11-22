//! Playable troops with independent rosters, card pools and relics.
mod avenger;

use super::Relic;
use serde::Serialize;

#[derive(Clone, Copy, Serialize)]
pub struct Character {
    pub id: &'static str,
    pub name: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub relic: Relic,
    /// Defeat this floor's boss to unlock; zero is available from the beginning.
    pub unlock_floor: u8,
}
pub const ALL: &[Character] = &[avenger::DEFINITION];

/// All content and entry rules needed to register an existing or new squad.
#[derive(Clone, Copy)]
pub enum ResourceRule {
    Mana,
}
#[derive(Clone, Copy)]
pub struct SquadDefinition {
    pub character: Character,
    pub formation: Option<([crate::PieceType; 8], Option<crate::PieceType>)>,
    pub starting_deck: &'static [crate::content::cards::CardKind],
    pub card_pool: &'static [crate::content::cards::CardKind],
    pub relic_pool: &'static [Relic],
    pub turn_rule: crate::cards::CardTurnRule,
    pub resource: ResourceRule,
    pub card_movement: bool,
}
pub const SQUADS: &[SquadDefinition] = &[avenger::SQUAD];
pub fn definition(id: &str) -> Option<&'static SquadDefinition> {
    SQUADS.iter().find(|squad| squad.character.id == id)
}
pub fn characters() -> Vec<Character> {
    ALL.to_vec()
}

pub fn roster(character: &str) -> Vec<crate::PlayerPiece> {
    use crate::{Battle, PlayerPiece, Position};
    use std::collections::BTreeMap;
    let Some((back, pawn)) = definition(character).and_then(|squad| squad.formation) else {
        return Battle::standard_roster();
    };
    (0..if pawn.is_some() { 16 } else { 8 })
        .map(|index| {
            let origin = Position::new(index % 8, if index < 8 { 7 } else { 6 });
            PlayerPiece {
                id: index as u32 + 1,
                kind: if index < 8 {
                    back[index as usize]
                } else {
                    pawn.unwrap()
                },
                origin,
                position: Some(origin),
                moved: false,
                upgrades: vec![],
                persistent: BTreeMap::new(),
            }
        })
        .collect()
}
