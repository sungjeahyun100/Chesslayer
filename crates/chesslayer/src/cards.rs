//! Shared card piles, mana and keyword lifecycle. Definitions live in content/cards.
pub use crate::content::cards::{starter_deck, CardDefinition, CardKind};
use crate::content::{Rng, BASE_MANA, DRAW_PER_TURN, HAND_LIMIT};
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Card {
    pub id: u32,
    pub kind: CardKind,
}
#[derive(Clone, Default)]
pub struct CardCombat {
    pub draw: Vec<Card>,
    pub hand: Vec<Card>,
    pub discard: Vec<Card>,
    pub exhausted: Vec<Card>,
    pub mana: u8,
}
impl CardCombat {
    pub fn start(deck: &[Card], rng: &mut Rng) -> Self {
        let mut draw = deck.to_vec();
        rng.shuffle(&mut draw);
        // pop() draws from the end. Stable partition puts innate cards first.
        draw.sort_by_key(|c| c.kind.definition().innate);
        let mut state = Self {
            draw,
            ..Self::default()
        };
        state.begin_turn(rng);
        state
    }
    pub fn begin_turn(&mut self, rng: &mut Rng) {
        self.mana = BASE_MANA;
        for _ in 0..DRAW_PER_TURN {
            if self.hand.len() >= HAND_LIMIT {
                break;
            }
            if self.draw.is_empty() {
                self.draw = std::mem::take(&mut self.discard);
                rng.shuffle(&mut self.draw);
            }
            if let Some(card) = self.draw.pop() {
                self.hand.push(card);
            } else {
                break;
            }
        }
    }
    pub fn end_turn(&mut self) {
        let mut retained = vec![];
        for card in self.hand.drain(..) {
            let d = card.kind.definition();
            if d.ethereal {
                self.exhausted.push(card);
            } else if d.retain {
                retained.push(card);
            } else {
                self.discard.push(card);
            }
        }
        self.hand = retained;
    }
    pub fn spend(&mut self, index: usize) {
        let card = self.hand.remove(index);
        let d = card.kind.definition();
        self.mana -= d.cost;
        if d.exhaust {
            self.exhausted.push(card);
        } else {
            self.discard.push(card);
        }
    }
}
