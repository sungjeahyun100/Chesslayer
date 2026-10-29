//! Shared card piles, mana and keyword lifecycle. Definitions live in content/cards.
pub use crate::content::cards::{CardDefinition, CardKind};
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
    pub mana: u32,
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
        state.begin_turn(rng, false);
        state
    }
    pub fn begin_turn(&mut self, rng: &mut Rng, accumulate_mana: bool) {
        self.mana = if accumulate_mana {
            self.mana.saturating_add(BASE_MANA)
        } else {
            BASE_MANA
        };
        self.draw_cards(DRAW_PER_TURN, rng);
    }
    pub fn draw_cards(&mut self, count: usize, rng: &mut Rng) {
        for _ in 0..count {
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
        // Count the original hand, before ethereal cards are exhausted.
        let discard_hand = self.hand.len() >= 5;
        let mut retained = vec![];
        for card in self.hand.drain(..) {
            let d = card.kind.definition();
            if d.ethereal {
                self.exhausted.push(card);
            } else if d.retain || !discard_hand {
                retained.push(card);
            } else {
                self.discard.push(card);
            }
        }
        self.hand = retained;
    }
    pub fn spend_with_cost(&mut self, index: usize, definition: &CardDefinition, cost: u32) {
        let card = self.hand.remove(index);
        self.mana -= cost;
        if definition.exhaust {
            self.exhausted.push(card);
        } else {
            self.discard.push(card);
        }
    }
    pub fn spend(&mut self, index: usize, definition: &CardDefinition) {
        self.spend_with_cost(index, definition, definition.cost);
    }
}
