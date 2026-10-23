//! The current characters differ only by their starting relic.
mod guardian;
mod vanguard;
mod wanderer;

use super::Relic;
use serde::Serialize;

#[derive(Clone, Copy, Serialize)]
pub struct Character {
    pub id: &'static str,
    pub name: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub relic: Relic,
}
pub const ALL: &[Character] = &[
    vanguard::DEFINITION,
    wanderer::DEFINITION,
    guardian::DEFINITION,
];

pub fn characters() -> Vec<Character> {
    ALL.to_vec()
}
