use super::*;
impl Game {
    pub(super) fn flush_mechanics(&mut self) {
        let count = std::mem::take(&mut self.battle.mechanics.draws);
        self.cards.draw_cards(count, &mut self.rng);
        self.cards.mana = self
            .cards
            .mana
            .saturating_add(std::mem::take(&mut self.battle.mechanics.mana));
        if self.squad_definition.id == "artillery" {
            self.cards.mana = self.battle.mechanics.ammo;
        }
    }
    pub(super) fn play_tactical(
        &mut self,
        id: u32,
        targets: &[PieceId],
        cells: &[usize],
    ) -> Result<(), String> {
        self.require_player()?;
        let index = self
            .cards
            .hand
            .iter()
            .position(|c| c.id == id)
            .ok_or("손패에 없는 카드입니다.")?;
        let kind = self.cards.hand[index].kind;
        let d = kind.definition();
        if !d.affiliation.available_to(self.squad_definition.id)
            || self.squad_definition.id == "guardian"
        {
            return Err("이 부대의 전술 카드가 아닙니다.".into());
        }
        if !matches!(
            d.selection_mode,
            content::cards::SelectionMode::Square | content::cards::SelectionMode::Placement
        ) && !cells.is_empty()
        {
            return Err("이 카드는 칸을 지정하지 않습니다.".into());
        }
        if d.selection_mode == content::cards::SelectionMode::None && !targets.is_empty() {
            return Err("대상이 없는 카드입니다.".into());
        }
        let mut next = self.battle.clone();
        let cost = next.tactical_card(kind, targets, cells)?;
        let balance = if self.squad_definition.id == "artillery" {
            self.battle.mechanics.ammo
        } else {
            self.cards.mana
        };
        if balance < cost {
            return Err("사용 비용이 부족합니다.".into());
        }
        if self.squad_definition.id == "artillery" {
            next.mechanics.ammo = next.mechanics.ammo.saturating_sub(cost);
        }
        self.battle = next;
        self.cards.mana = balance;
        self.cards.spend_with_cost(index, &d, cost);
        self.log(format!(
            "{} 사용 · {} −{}",
            d.name,
            if self.squad_definition.id == "artillery" {
                "탄약"
            } else {
                "마나"
            },
            cost
        ));
        self.events();
        self.check_finish()?;
        Ok(())
    }
    pub(super) fn prepare_aftermath(&mut self, won: bool) -> Result<(), String> {
        self.battle.mechanics.choices.clear();
        let roster = self.battle.outcome();
        let hungry = roster
            .iter()
            .filter(|p| p.position.is_some() && self.battle.status(p.id).hunger > 0)
            .map(|p| p.id)
            .collect::<Vec<_>>();
        if won && self.relics.contains(&Relic::LastPlate) && !hungry.is_empty() {
            self.aftermath.push(ChoiceView {
                prompt: "마지막 접시: 허기 악화를 막을 아군".into(),
                kind: "last_plate".into(),
                targets: hungry,
                cells: vec![],
                amount: 0,
            });
        } else {
            for id in hungry {
                self.battle.worsen_hunger(id);
            }
        }
        if won && self.relics.contains(&Relic::Reinforcements) {
            let dead = roster
                .iter()
                .filter(|p| {
                    p.position.is_none()
                        && !content::pieces::is_player_royal(p.kind)
                        && p.persistent.get("defected") != Some(&1)
                })
                .map(|p| p.id)
                .collect::<Vec<_>>();
            if !dead.is_empty() {
                self.aftermath.push(ChoiceView {
                    prompt: "충원: 초기 종류로 부활할 기물".into(),
                    kind: "reinforcements".into(),
                    targets: dead,
                    cells: vec![],
                    amount: 0,
                });
            }
        }
        Ok(())
    }
    pub(super) fn resolve_aftermath(&mut self, target: u32) -> Result<(), String> {
        if !matches!(self.phase, RunPhase::Reward | RunPhase::Victory) {
            return Err("전투가 끝난 뒤에 선택하세요.".into());
        }
        let choice = self
            .aftermath
            .first()
            .cloned()
            .ok_or("남은 효과가 없습니다.")?;
        if !choice.targets.contains(&target) {
            return Err("표시된 기물 중 선택하세요.".into());
        }
        let mut roster = self.battle.outcome();
        if choice.kind == "last_plate" {
            for p in &mut roster {
                if p.id != target && p.position.is_some() {
                    let h = p.persistent.entry("hunger".into()).or_default();
                    if *h > 0 {
                        *h = (*h + 1).min(3);
                    }
                }
            }
        } else {
            let index = roster
                .iter()
                .position(|p| p.id == target)
                .ok_or("잘못된 기물입니다.")?;
            let origin = roster[index].origin;
            if roster.iter().any(|p| p.position == Some(origin)) {
                return Err("시작 칸이 점유되어 있습니다.".into());
            }
            let p = &mut roster[index];
            p.kind = content::pieces::base_kind(p.kind);
            p.position = Some(origin);
            p.upgrades.clear();
            p.moved = false;
        }
        self.battle
            .replace_roster_at_rest(roster)
            .map_err(engine_error)?;
        self.aftermath.remove(0);
        Ok(())
    }
}
