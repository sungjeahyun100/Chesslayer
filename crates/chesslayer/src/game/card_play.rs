//! card play rules. The call sites retain their original ordering.
use super::*;

impl Game {
    pub(super) fn play_card(
        &mut self,
        id: u32,
        selection: content::cards::CardSelection,
    ) -> Result<(), String> {
        self.require_player()?;
        let index = self
            .cards
            .hand
            .iter()
            .position(|c| c.id == id)
            .ok_or("손패에 없는 카드입니다.")?;
        if content::cards::avenger::is_avenger(self.cards.hand[index].kind) {
            if !selection.targets.is_empty() || !selection.upgrades.is_empty() {
                return Err("기물 하나를 선택하세요.".into());
            }
            return self.play_avenger_card(index, selection.target, None);
        }
        let d = self.cards.effective_definition(&self.cards.hand[index]);
        self.resolve_selection(index, &selection, d)
    }
    #[cfg(test)]
    pub(super) fn resolve_card(
        &mut self,
        index: usize,
        target: Option<u32>,
        d: CardDefinition,
    ) -> Result<(), String> {
        self.resolve_selection(index, &content::cards::CardSelection::single(target), d)
    }
    pub(super) fn resolve_selection(
        &mut self,
        index: usize,
        selection: &content::cards::CardSelection,
        d: CardDefinition,
    ) -> Result<(), String> {
        let side = self.battle.turn;
        let (cost, removed) = crate::cards::resolve_selection(
            &mut self.battle,
            &mut self.cards,
            index,
            selection,
            d.clone(),
        )?;
        if self.ammo.is_some() {
            self.ammo = Some(self.cards.mana);
        }
        self.log(format!(
            "{} 사용 · {} −{}{}",
            d.name,
            if self.ammo.is_some() {
                "탄약"
            } else {
                "마나"
            },
            cost,
            removed
                .map(|n| if d.kind == CardKind::Volley {
                    format!(" · {n}회 사격")
                } else {
                    format!(" · {n}기 처리")
                })
                .unwrap_or_default()
        ));
        self.events();
        self.check_finish()?;
        if self.phase == RunPhase::Combat && self.battle.turn != side {
            if side == PLAYER {
                self.cards.end_turn();
            } else {
                self.begin_player_turn();
            }
        }
        Ok(())
    }
}
