use super::*;

impl Game {
    fn effective_avenger_definition(&self, card: &crate::cards::Card) -> CardDefinition {
        let mut definition = self.cards.effective_definition(card);
        if self.legacy_content
            && card.kind == CardKind::BloodOffering
            && !self.cards.cost_overrides.contains_key(&card.id)
        {
            definition.cost = 2;
        }
        definition
    }
    pub(super) fn roll_reward_card(
        &mut self,
        minimum: content::cards::avenger::Rarity,
    ) -> CardKind {
        let pool = self.card_pool();
        self.roll_card_candidates(pool, minimum)
    }
    pub(super) fn roll_card_candidates(
        &mut self,
        pool: &[CardKind],
        minimum: content::cards::avenger::Rarity,
    ) -> CardKind {
        use content::cards::avenger::Rarity::*;
        let roll = self.rng.index(100);
        let rarity = match roll {
            0..=53 => Common,
            54..=83 => Rare,
            84..=98 => Epic,
            _ => Legendary,
        }
        .max(minimum);
        let mut candidates: Vec<_> = pool
            .iter()
            .copied()
            .filter(|k| k.definition().rarity == rarity)
            .collect();
        if candidates.is_empty() {
            candidates = pool
                .iter()
                .copied()
                .filter(|k| k.definition().rarity >= minimum)
                .collect();
        }
        if candidates.is_empty() {
            candidates = pool.to_vec();
        }
        candidates[self.rng.index(candidates.len())]
    }
    pub(super) fn roll_unique_cards(
        &mut self,
        pool: &[CardKind],
        count: usize,
        minimum: content::cards::avenger::Rarity,
    ) -> Vec<CardKind> {
        let mut available = pool.to_vec();
        available.sort_unstable();
        available.dedup();
        let mut chosen = Vec::with_capacity(count.min(available.len()));
        for _ in 0..count.min(available.len()) {
            let kind = self.roll_card_candidates(&available, minimum);
            available.retain(|candidate| *candidate != kind);
            chosen.push(kind);
        }
        chosen
    }
    pub(super) fn configure_expansion(&mut self) -> Result<(), String> {
        if content::characters::definition(&self.squad_id).is_some_and(|squad| squad.card_movement)
        {
            self.battle.enable_card_movement();
        }
        if self
            .relics
            .iter()
            .any(|r| matches!(r, Relic::Sugar | Relic::DevilSugar))
        {
            self.cards.mana = 4;
        }
        if self.relics.contains(&Relic::DesireBag) {
            self.cards.draw_cards(2, &mut self.rng);
        }
        let allies: Vec<_> = self
            .battle
            .board
            .cells()
            .iter()
            .flatten()
            .filter(|p| p.owner == PLAYER)
            .map(|p| p.id)
            .collect();
        if self.relics.contains(&Relic::GhostBoots) && !allies.is_empty() {
            self.battle
                .grant_move_rights(allies[self.rng.index(allies.len())], 2)
                .map_err(engine_error)?;
        }
        for id in &allies {
            if self.relics.contains(&Relic::FullyArmed) {
                self.battle.stats_mut(*id).attack += 1;
            }
            if self.relics.contains(&Relic::Pride) {
                self.battle.stats_mut(*id).score_bonus += 2;
            }
        }
        if self.relics.contains(&Relic::DevilSugar) {
            let targets: Vec<_> = allies
                .into_iter()
                .filter(|id| !self.battle.is_royal_piece(*id))
                .collect();
            if !targets.is_empty() {
                self.battle
                    .force_remove(&[targets[self.rng.index(targets.len())]])
                    .map_err(engine_error)?;
            }
        }
        Ok(())
    }
    pub(super) fn play_avenger_card(
        &mut self,
        index: usize,
        target: Option<PieceId>,
        name: Option<&str>,
    ) -> Result<(), String> {
        use CardKind::*;
        let card = self
            .cards
            .hand
            .get(index)
            .ok_or("손패에 없는 카드입니다.")?
            .clone();
        let d = self.effective_avenger_definition(&card);
        if self.cards.mana < d.cost {
            return Err("마나가 부족합니다.".into());
        }
        if d.targeted
            && !self
                .battle
                .avenger_targets(card.kind)
                .contains(&target.ok_or("대상을 선택하세요.")?)
        {
            return Err("조건을 충족하는 대상을 선택하세요.".into());
        }
        if !d.targeted && target.is_some() {
            return Err("이 카드는 대상을 선택하지 않습니다.".into());
        }
        if card.kind == Bounty && self.gold < 50 {
            return Err("50골드 이상이 필요합니다.".into());
        }
        // Validate and apply to a staged run, including RNG, resources and royal changes.
        let mut next = self.clone();
        next.cards.hand.remove(index);
        next.cards.mana -= d.cost;
        next.cards.cost_overrides.remove(&card.id);
        let id = target.unwrap_or(0);
        next.battle
            .avenger_effect(card.kind, target, name)
            .map_err(engine_error)?;
        match card.kind {
            Divinity => {
                let choices: Vec<_> = next
                    .battle
                    .board
                    .cells()
                    .iter()
                    .flatten()
                    .filter(|p| p.owner == PLAYER)
                    .filter_map(|p| {
                        let options = next.battle.temporary_upgrade_options(p.id);
                        (!options.is_empty())
                            .then(|| (p.id, options[next.rng.index(options.len())]))
                    })
                    .collect();
                if !choices.is_empty() {
                    next.battle
                        .temporary_upgrades(&choices)
                        .map_err(engine_error)?;
                    for (id, target) in choices {
                        if let Some(p) = next.battle.player_pieces().iter().find(|p| p.id == id) {
                            let mut upgrades = p.upgrades.clone();
                            if let Some(path) = content::pieces::definition(p.kind)
                                .and_then(|d| d.upgrades.iter().find(|u| u.target == target))
                            {
                                upgrades.push(path.key.into());
                            }
                            next.battle
                                .set_permanent_upgrade(id, target, upgrades, true)
                                .map_err(engine_error)?;
                        }
                    }
                }
            }
            BadHand | PotOfGreed => {
                next.cards.draw_cards(2, &mut next.rng);
                if card.kind == BadHand {
                    next.cards.draw_locked = true;
                }
            }
            Optimize | ClaudeOptimize => {
                for c in next.cards.hand.clone() {
                    let cost = if card.kind == ClaudeOptimize {
                        0
                    } else {
                        next.effective_avenger_definition(&c).cost.saturating_sub(1)
                    };
                    next.cards.cost_overrides.insert(c.id, cost);
                }
            }
            BloodOffering | WealthOffering => {
                let score = next.battle.piece_score(id).max(0) as u32;
                next.battle.force_remove(&[id]).map_err(engine_error)?;
                if card.kind == BloodOffering {
                    next.cards.mana = next.cards.mana.saturating_add(score);
                } else {
                    next.cards.draw_cards(score as usize, &mut next.rng);
                }
            }
            Bounty => {
                next.battle
                    .grant_move_rights(id, next.gold / 50)
                    .map_err(engine_error)?;
                next.gold = 0;
            }
            _ => {}
        }
        if d.exhaust {
            next.cards.exhausted.push(card.clone());
        } else {
            next.cards.discard.push(card.clone());
        }
        next.debug_event("card_resolved", Some(d.name.into()),
            target.map(|id| format!("기물 #{id}")), "부대 카드 효과 적용 완료");
        next.log(format!("{} 사용 · 마나 −{}", d.name, d.cost));
        if matches!(card.kind, Castling | EnPassant) && next.battle.phase == Phase::Active {
            next.battle.end_turn().map_err(engine_error)?;
            next.cards.end_turn();
        }
        next.events();
        next.check_finish()?;
        *self = next;
        Ok(())
    }
}
