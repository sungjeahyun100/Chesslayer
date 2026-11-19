//! shop rules. The call sites retain their original ordering.
use super::*;

impl Game {
    pub(super) fn generate_shop_offers(&mut self) {
        self.shop_offers = self.card_pool().to_vec();
        if !self.legacy_content {
            self.shop_offers.sort_unstable();
            self.shop_offers.dedup();
        }
        self.rng.shuffle(&mut self.shop_offers);
        self.shop_offers.truncate(3);
        self.shop_bought.clear();
    }
    /// Remove one card instance from the expedition and every active combat pile.
    pub(super) fn burn_card(&mut self, id: u32) -> Result<Card, String> {
        let index = self
            .deck
            .iter()
            .position(|card| card.id == id)
            .ok_or("소각할 카드가 없습니다.")?;
        let card = self.deck.remove(index);
        self.cards.hand.retain(|other| other.id != id);
        self.cards.draw.retain(|other| other.id != id);
        self.cards.discard.retain(|other| other.id != id);
        self.cards.exhausted.retain(|other| other.id != id);
        self.cards.cost_overrides.remove(&id);
        Ok(card)
    }
    pub fn next_burn_cost(&self) -> u32 {
        self.cards_burned.saturating_add(1).saturating_mul(100)
    }
}
