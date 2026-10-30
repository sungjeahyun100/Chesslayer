//! Playable troops with independent rosters, card pools and relics.
mod artillery;
mod guardian;
mod heretic;
mod special;
mod vanguard;

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
pub const ALL: &[Character] = &[
    guardian::DEFINITION,
    vanguard::DEFINITION,
    heretic::DEFINITION,
    artillery::DEFINITION,
    special::DEFINITION,
];

pub fn characters() -> Vec<Character> {
    ALL.to_vec()
}

pub fn roster(character: &str) -> Vec<crate::PlayerPiece> {
    use crate::content::pieces::*;
    use crate::{Battle, PlayerPiece, Position};
    use std::collections::BTreeMap;
    let (back, pawn): ([crate::PieceType; 8], Option<crate::PieceType>) = match character {
        "vanguard" => (
            [
                SPEAR_INFANTRY,
                AXE_INFANTRY,
                SWORD_INFANTRY,
                VANGUARD_OFFICER,
                VANGUARD_COMMANDER,
                SWORD_INFANTRY,
                AXE_INFANTRY,
                SPEAR_INFANTRY,
            ],
            Some(AXE_INFANTRY),
        ),
        "heretic" => (
            [
                BUTCHER_CART,
                FRENZIED_DANCER,
                INTOXICATED_PRIEST,
                MASKED_LADY,
                SMILING_KING,
                INTOXICATED_PRIEST,
                FRENZIED_DANCER,
                BUTCHER_CART,
            ],
            Some(PROCESSION_FOLLOWER),
        ),
        "artillery" => (
            [
                MORTAR, DRONE, WAREHOUSE, TANK, TANK, WAREHOUSE, DRONE, MORTAR,
            ],
            Some(RIFLEMAN),
        ),
        "special" => (
            [
                SNIPER,
                SCOUT,
                COMMANDO,
                COMMANDO,
                SPECIAL_COMMANDER,
                COMMANDO,
                SCOUT,
                SNIPER,
            ],
            None,
        ),
        _ => return Battle::standard_roster(),
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
