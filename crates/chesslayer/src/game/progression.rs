//! progression rules. The call sites retain their original ordering.
use super::*;

impl Game {
    pub(super) fn assign_map_relics(&mut self) {
        let pool = content::relics::reward_pool(&self.squad_id);
        let mut rng = Rng::new(self.seed ^ 0x0300_00ab);
        for node in &mut self.map.nodes {
            if node.relic.is_some() {
                node.relic = Some(pool[rng.index(pool.len())]);
            }
        }
    }
    pub fn available_nodes(&self) -> Vec<usize> {
        if self.phase != RunPhase::Map {
            return vec![];
        }
        match self.current_node {
            None => self.map.starts(),
            Some(id) if self.completed.contains(&id) => self.map.nodes[id].next.clone(),
            Some(_) => vec![],
        }
    }
    pub(super) fn complete_node(&mut self) {
        let id = self.current_node.expect("content requires a selected node");
        self.record_node_clear();
        if !self.completed.contains(&id) {
            self.completed.push(id);
        }
        self.last_cleared_node = Some(id);
        self.phase = RunPhase::Map;
    }
    pub(super) fn record_node_clear(&mut self) {
        let Some(id) = self.current_node else {
            return;
        };
        // Traversal only moves forward; the sole revisit is this node's rewards.
        // A won battle counts immediately, before its reward screen is closed.
        if self.last_counted_node != Some(id) {
            self.cleared_stages += 1;
            self.last_counted_node = Some(id);
        }
    }
    pub(super) fn continue_reward(&mut self) -> Result<(), String> {
        if self.phase != RunPhase::Reward {
            return Err("지금은 다음으로 진행할 수 없습니다.".into());
        }
        let id = self.current_node.expect("reward requires a selected node");
        if self.completed.contains(&id) {
            self.phase = RunPhase::Map;
            if self.map.nodes[id].kind == NodeKind::Boss {
                self.finish_boss()?;
            }
            return Ok(());
        }
        self.complete_node();
        if self.map.nodes[id].kind == NodeKind::Boss {
            self.finish_boss()?;
        }
        Ok(())
    }
    pub(super) fn finish_boss(&mut self) -> Result<(), String> {
        let mut roster = self.battle.outcome();
        let mut occupied: HashSet<_> = roster.iter().filter_map(|p| p.position).collect();
        let mut revived = 0;
        for piece in roster.iter_mut().filter(|p| p.position.is_none()) {
            let position = if !occupied.contains(&piece.origin) {
                piece.origin
            } else {
                (0..8)
                    .rev()
                    .flat_map(|rank| (0..8).map(move |file| Position::new(file, rank)))
                    .find(|pos| !occupied.contains(pos))
                    .ok_or("아군을 부활시킬 빈 칸이 없습니다.")?
            };
            occupied.insert(position);
            piece.position = Some(position);
            piece.moved = false;
            revived += 1;
        }
        if revived > 0 {
            self.battle
                .replace_roster_at_rest(roster)
                .map_err(engine_error)?;
            self.log(format!(
                "보스 격파: 아군 기물 {revived}기가 회복되었습니다."
            ));
        }
        if let Some(capacity) = self.ammo_capacity() {
            self.ammo = Some(capacity);
        }
        if self.final_battle {
            self.phase = RunPhase::Victory;
            self.log("부족의 독재자를 쓰러뜨렸습니다. 원정을 완수했습니다.");
            return Ok(());
        }
        self.floor += 1;
        self.final_battle = self.mode == RunMode::Campaign && self.floor == 5;
        self.current_node = None;
        self.path.clear();
        self.completed.clear();
        self.reward_node = None;
        self.reward_choice = None;
        self.last_cleared_node = None;
        self.last_counted_node = None;
        self.upgrade_claimed = false;
        self.phase = RunPhase::Map;
        self.map = if self.mode == RunMode::Endless {
            ActMap::generate_endless(self.seed, self.floor)
        } else if self.final_battle {
            self.floor = 4;
            ActMap {
                rows: 1,
                nodes: vec![MapNode {
                    id: 0,
                    row: 0,
                    lane: 2,
                    kind: NodeKind::Boss,
                    encounter: Some(content::encounters::DICTATOR_BOSS),
                    relic: None,
                    next: vec![],
                }],
            }
        } else {
            ActMap::generate_for_floor(
                self.seed ^ self.floor.wrapping_mul(0x9e37_79b9),
                self.floor as u8,
            )
        };
        self.assign_map_relics();
        self.log(if self.final_battle {
            "최종 결전으로 향합니다."
        } else {
            "다음 층으로 향합니다."
        });
        Ok(())
    }
    pub(super) fn unlock_boss(&mut self) {
        if self.mode == RunMode::Endless {
            return;
        }
        if self.final_battle {
            self.progress.final_defeated = true;
        }
    }
    pub(super) fn check_finish(&mut self) -> Result<(), String> {
        if self.phase != RunPhase::Combat {
            return Ok(());
        }
        if let GameResult::Win { winner, reason } = self.battle.result {
            self.resource_effects();
            if self.simulator {
                self.phase = if winner == PLAYER {
                    RunPhase::Victory
                } else {
                    RunPhase::Defeat
                };
                self.log(if winner == PLAYER {
                    format!("시뮬레이터 승리 · {reason:?}")
                } else {
                    format!("시뮬레이터 패배 · {reason:?}")
                });
                return Ok(());
            }
            self.battle.reset_after_battle().map_err(engine_error)?;
            self.cards = CardCombat::default();
            if winner == PLAYER {
                self.record_node_clear();
                self.phase = RunPhase::Reward;
                self.gold += if self.event_state.ambush_combat {
                    45
                } else if self
                    .current_node
                    .is_some_and(|id| self.map.nodes[id].kind == NodeKind::Combat)
                {
                    15
                } else {
                    30
                };
                self.reinforcement_used = false;
                self.hunger_protection_used = false;
                if self
                    .current_node
                    .is_some_and(|id| self.map.nodes[id].kind == NodeKind::Boss)
                {
                    self.bosses_defeated += 1;
                    self.unlock_boss();
                }
                self.generate_rewards();
                self.event_state.ambush_combat = false;
                self.reward_node = self.current_node;
                self.reward_choice = None;
                self.upgrade_claimed = false;
                self.log(match reason {
                    ResultReason::SelfDefense => "승리! 호신용품으로 적 로얄이 사망했습니다.",
                    ResultReason::RacingGoal => "승리! 킹이 상대 진영 끝 랭크에 도달했습니다.",
                    ResultReason::HillGoal => "승리! 언덕의 왕이 중앙에 도달했습니다.",
                    _ => "승리! 적 로얄을 직접 포획했습니다.",
                });
                self.log("생존 기물이 시작 칸으로 복귀합니다.");
            } else {
                self.phase = RunPhase::Defeat;
                self.log(match reason {
                    ResultReason::Counterattack => "역습으로 킹을 잃었습니다. 원정이 끝났습니다.",
                    ResultReason::RacingGoal => {
                        "적 킹이 아군 진영 끝 랭크에 도달했습니다. 원정이 끝났습니다."
                    }
                    ResultReason::HillGoal => {
                        "적 언덕의 왕이 중앙에 도달했습니다. 원정이 끝났습니다."
                    }
                    _ => "킹이 포획되었습니다. 원정이 끝났습니다.",
                });
            }
        }
        Ok(())
    }
}
