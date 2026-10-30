//! Shared card piles, mana and keyword lifecycle. Definitions live in content/cards.
pub use crate::content::cards::{starter_deck, CardDefinition, CardKind};
use crate::content::cards::{CardSelection, SelectionMode};
use crate::content::{Rng, BASE_MANA, DRAW_PER_TURN, HAND_LIMIT};
use crate::Battle;
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

/// The same atomic card validation, effect, payment and pile transfer for runs and sandboxes.
pub fn resolve_selection(
    battle: &mut Battle,
    cards: &mut CardCombat,
    index: usize,
    selection: &CardSelection,
    d: CardDefinition,
) -> Result<(u32, Option<usize>), String> {
    match d.selection_mode {
        SelectionMode::None
            if selection.target.is_some()
                || !selection.targets.is_empty()
                || !selection.upgrades.is_empty() =>
        {
            return Err("이 카드는 대상을 선택하지 않습니다.".into());
        }
        SelectionMode::Single
            if !selection.targets.is_empty() || !selection.upgrades.is_empty() =>
        {
            return Err("기물 하나만 선택하세요.".into());
        }
        SelectionMode::Multiple if selection.target.is_some() || !selection.upgrades.is_empty() => {
            return Err("복수 기물 선택이 올바르지 않습니다.".into());
        }
        SelectionMode::Upgrade if selection.target.is_some() || !selection.targets.is_empty() => {
            return Err("강화 경로 선택이 올바르지 않습니다.".into());
        }
        _ => {}
    }
    let mut cost = d.selected_cost(selection)?;
    if cards.mana < cost {
        return Err("마나가 부족합니다.".into());
    }
    let mut next = battle.clone();
    let removed = if let Some(effect) = d.selection_effect {
        effect(&mut next, selection)?
    } else {
        (d.effect)(&mut next, selection.target)?
    };
    if d.kind == CardKind::Volley {
        cost = removed.unwrap_or(0) as u32;
        if cards.mana < cost {
            return Err(format!("일제사격에 필요한 탄약이 부족합니다: {cost}"));
        }
    }
    if cards.mana < cost.saturating_add(next.pending_ammo_spend()) {
        return Err("카드 비용과 사격 탄약이 부족합니다.".into());
    }
    *battle = next;
    cards.spend_with_cost(index, &d, cost);
    Ok((cost, removed))
}
