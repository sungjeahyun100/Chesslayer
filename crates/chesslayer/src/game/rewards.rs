//! rewards rules. The call sites retain their original ordering.
use super::*;

impl Game {
    pub(super) fn claim_reward(&mut self, index: usize) -> Result<(), String> {
        if self.phase != RunPhase::Reward || self.reward_choice.is_some() {
            return Err("지금은 보상을 선택할 수 없습니다.".into());
        }
        let reward = *self
            .rewards
            .get(index)
            .ok_or("보상 선택이 잘못되었습니다.")?;
        match reward {
            Reward::Card(kind) => {
                self.add_card(kind);
                self.log(format!(
                    "보상: {}을 덱에 추가했습니다.",
                    kind.definition().name
                ));
            }
            Reward::Relic(relic) => self.grant_relic(relic),
        }
        self.reward_choice = Some(index);
        Ok(())
    }
    pub(super) fn claim_upgrade(&mut self, id: u32, target: PieceType) -> Result<(), String> {
        if self.phase != RunPhase::Reward
            || !self.current_node.is_some_and(|node| {
                matches!(self.map.nodes[node].kind, NodeKind::Elite | NodeKind::Boss)
            })
            || self.upgrade_claimed
        {
            return Err("지금은 기물 강화 보상을 받을 수 없습니다.".into());
        }
        self.upgrade_piece(id, target, "보상")?;
        self.upgrade_claimed = true;
        Ok(())
    }
    pub(super) fn upgrade_piece(
        &mut self,
        id: u32,
        target: PieceType,
        source: &str,
    ) -> Result<(), String> {
        let mut roster = self.battle.outcome();
        let index = roster
            .iter()
            .position(|p| p.id == id && p.position.is_some())
            .ok_or("살아 있는 아군 기물을 선택하세요.")?;
        let previous_kind = roster[index].kind;
        if content::pieces::is_monster_only(previous_kind)
            || content::pieces::is_monster_only(target)
        {
            return Err("몬스터 전용 기물은 영구 강화할 수 없습니다.".into());
        }
        let upgrade = content::pieces::definition(previous_kind)
            .and_then(|d| d.upgrades.iter().find(|u| u.target == target))
            .ok_or("이 기물에 정의된 강화 경로가 없습니다.")?;
        let missing: Vec<_> = content::pieces::required_owned(target)
            .iter()
            .filter(|&&kind| {
                !roster
                    .iter()
                    .any(|p| content::pieces::owned_matches_kind(p, kind))
            })
            .map(|&kind| piece_name(kind))
            .collect();
        if !missing.is_empty() {
            return Err(format!(
                "영구 로스터에 필요한 기물이 없습니다: {}",
                missing.join(", ")
            ));
        }
        let p = &mut roster[index];
        p.kind = upgrade.target;
        p.upgrades.push(upgrade.key.into());
        self.battle
            .replace_roster_at_rest(roster)
            .map_err(engine_error)?;
        self.log(format!(
            "{source}: {} → {} 강화",
            piece_name(previous_kind),
            piece_name(target)
        ));
        Ok(())
    }
    pub(super) fn developer_set_upgrade(
        &mut self,
        id: u32,
        target: PieceType,
    ) -> Result<(), String> {
        let roster = self.battle.outcome();
        let piece = roster
            .iter()
            .find(|p| p.id == id)
            .ok_or("아군 기물을 선택하세요.")?;
        let previous_kind = piece.kind;
        if content::pieces::is_monster_only(previous_kind)
            || content::pieces::is_monster_only(target)
        {
            return Err("몬스터 전용 기물은 영구 강화할 수 없습니다.".into());
        }
        // Upgrade keys are shared by families. Resolve each recorded edge from
        // its destination, then search only this piece's actual family tree.
        let mut root = previous_kind;
        for key in piece.upgrades.iter().rev() {
            let edge = content::pieces::ALL
                .iter()
                .flat_map(|d| d.upgrades.iter().map(move |u| (d.kind, u)))
                .find(|(_, upgrade)| {
                    upgrade.target == root
                        && (upgrade.key == key
                            || content::pieces::definition(root).is_some_and(|d| d.key == key))
                })
                .ok_or("기물의 기존 강화 경로를 찾을 수 없습니다.")?;
            root = edge.0;
        }
        let mut queue = VecDeque::from([(root, Vec::new())]);
        let mut visited = HashSet::new();
        let path = loop {
            let Some((kind, path)) = queue.pop_front() else {
                return Err("이 기물에 정의된 강화 상태가 아닙니다.".into());
            };
            if kind == target {
                break path;
            }
            if !visited.insert(kind) {
                continue;
            }
            if let Some(definition) = content::pieces::definition(kind) {
                for upgrade in definition.upgrades {
                    let mut next = path.clone();
                    next.push(upgrade.key.to_string());
                    queue.push_back((upgrade.target, next));
                }
            }
        };
        let board_matches = piece.position.is_none_or(|at| {
            self.battle
                .board
                .piece_at(at)
                .ok()
                .flatten()
                .is_some_and(|on_board| on_board.kind == target)
        });
        if previous_kind == target && piece.upgrades == path && board_matches {
            return Ok(());
        }
        self.battle
            .set_permanent_upgrade(id, target, path, self.phase == RunPhase::Combat)
            .map_err(engine_error)?;
        self.log(format!(
            "개발자 강화: {} → {}",
            piece_name(previous_kind),
            piece_name(target)
        ));
        Ok(())
    }
    pub(super) fn grant_node_relic(&mut self) {
        let relic = self.map.nodes[self.current_node.unwrap()]
            .relic
            .expect("reward node has relic");
        self.grant_relic(relic);
    }
    pub(super) fn add_card(&mut self, kind: CardKind) {
        self.deck.push(Card {
            id: self.next_card_id,
            kind,
        });
        self.next_card_id += 1;
    }
    pub(super) fn card_pool(&self) -> &'static [CardKind] {
        if self.legacy_content && self.squad_id == "avenger" {
            let pool = content::cards::avenger::REWARD_POOL;
            &pool[..pool.len() - 2]
        } else {
            content::cards::reward_pool(&self.squad_id)
        }
    }
    pub(super) fn grant_relic(&mut self, relic: Relic) {
        if self.relics.contains(&relic) {
            self.add_card(self.card_pool()[0]);
            self.log(format!(
                "{}은 이미 보유 중입니다. 대신 부대 전용 카드 1장을 얻었습니다.",
                relic.name()
            ));
        } else {
            self.relics.push(relic);
            self.log(format!(
                "유물 획득: {} · {}",
                relic.name(),
                relic.description()
            ));
        }
    }
    pub(super) fn generate_rewards(&mut self) {
        if self.squad_id == "avenger"
            || self.event_state.ambush_combat
            || (self.mode == RunMode::Endless && self.floor >= 3)
        {
            use content::cards::avenger::Rarity;
            let minimum = if self.event_state.ambush_combat {
                Rarity::Epic
            } else if self
                .current_node
                .is_some_and(|id| self.map.nodes[id].kind == NodeKind::Boss)
            {
                if self.squad_id == "avenger" {
                    Rarity::Legendary
                } else {
                    Rarity::Epic
                }
            } else if self.mode == RunMode::Endless && self.floor >= 5 {
                Rarity::Rare
            } else {
                Rarity::Common
            };
            if self.legacy_content {
                self.rewards = (0..3)
                    .map(|_| Reward::Card(self.roll_reward_card(minimum)))
                    .collect();
                return;
            }
            let pool = self.card_pool();
            self.rewards = self
                .roll_unique_cards(pool, 3, minimum)
                .into_iter()
                .map(Reward::Card)
                .collect();
            return;
        }
        let mut cards = self.card_pool().to_vec();
        if !self.legacy_content {
            cards.sort_unstable();
            cards.dedup();
        }
        if self
            .current_node
            .is_some_and(|id| matches!(self.map.nodes[id].kind, NodeKind::Elite | NodeKind::Boss))
        {
            let mut rewards = Vec::with_capacity(3);
            for _ in 0..if self.legacy_content {
                3
            } else {
                3.min(cards.len())
            } {
                if cards.is_empty() {
                    cards = self.card_pool().to_vec();
                }
                rewards.push(Reward::Card({
                    let chosen =
                        self.roll_card_candidates(&cards, content::cards::avenger::Rarity::Common);
                    let index = cards.iter().position(|k| *k == chosen).unwrap();
                    cards.swap_remove(index)
                }));
            }
            self.rewards = rewards;
            return;
        }
        let mut relics: Vec<_> = content::relics::reward_pool(&self.squad_id)
            .iter()
            .copied()
            .filter(|r| !self.relics.contains(r))
            .collect();
        let mut rewards = vec![Reward::Card({
            let chosen = self.roll_card_candidates(&cards, content::cards::avenger::Rarity::Common);
            let index = cards.iter().position(|k| *k == chosen).unwrap();
            cards.swap_remove(index)
        })];
        if !relics.is_empty() {
            rewards.push(Reward::Relic(
                relics.swap_remove(self.rng.index(relics.len())),
            ));
        }
        let mut remaining: Vec<_> = cards
            .into_iter()
            .map(Reward::Card)
            .chain(relics.into_iter().map(Reward::Relic))
            .collect();
        // Registered card kinds fill every slot when relics run out.
        while rewards.len() < 3 {
            if remaining.is_empty() {
                if self.legacy_content {
                    remaining.extend(self.card_pool().iter().copied().map(Reward::Card));
                    continue;
                }
                break;
            }
            let mut index = self.rng.index(remaining.len());
            if matches!(remaining[index], Reward::Card(_)) {
                let candidates: Vec<_> = remaining
                    .iter()
                    .filter_map(|r| {
                        if let Reward::Card(k) = r {
                            Some(*k)
                        } else {
                            None
                        }
                    })
                    .collect();
                let chosen =
                    self.roll_card_candidates(&candidates, content::cards::avenger::Rarity::Common);
                index = remaining
                    .iter()
                    .position(|r| *r == Reward::Card(chosen))
                    .unwrap();
            }
            rewards.push(remaining.swap_remove(index));
        }
        self.rng.shuffle(&mut rewards);
        self.rewards = rewards;
    }
}
