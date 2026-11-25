//! battle flow rules. The call sites retain their original ordering.
use super::*;

impl Game {
    pub(super) fn starting_roster(&self) -> Result<Vec<PlayerPiece>, String> {
        let mut roster = self.battle.outcome();
        let mut occupied = HashSet::new();
        let formation = self.latest_formation();
        let mut unplaced = vec![];
        for (index, p) in roster.iter_mut().enumerate() {
            if p.position.is_none() {
                continue; // A preset must never revive a dead ID.
            }
            let desired = formation.and_then(|r| r.position(p.id));
            p.moved = false;
            if let Some(pos) = desired.filter(|pos| occupied.insert(*pos)) {
                p.position = Some(pos);
            } else {
                unplaced.push(index);
            }
        }
        // IDs missing from a preset use their permanent home.
        // Preset positions have priority; a blocked home uses the first free square
        // in a1..h1, a2..h2, ... order. No survivor is displaced or removed.
        for index in unplaced {
            let p = &mut roster[index];
            let pos = if !occupied.contains(&p.origin) {
                p.origin
            } else {
                (0..8)
                    .rev()
                    .flat_map(|rank| (0..8).map(move |file| Position::new(file, rank)))
                    .find(|pos| !occupied.contains(pos))
                    .ok_or("아군을 배치할 빈 칸이 없습니다.")?
            };
            occupied.insert(pos);
            p.position = Some(pos);
        }
        Ok(roster)
    }
    pub(super) fn start_battle(&mut self) -> Result<(), String> {
        let index = self
            .current_node
            .and_then(|id| self.map.nodes[id].encounter)
            .ok_or("전투 노드가 아닙니다.")?;
        self.debug_event("battle_start_before", None, None, "전투 시작 전 상태");
        let mut encounter = encounter(index);
        content::encounters::choose_random_royal(index, &mut encounter, &mut self.rng);
        self.log(format!("{} · 전투 시작", ENCOUNTERS[index].name));
        let mut players = self.starting_roster()?;
        self.debug_event("formation", None, None, "생존 로스터를 시작 진형에 배치");
        for piece in &mut players {
            piece.persistent.remove("hunger_worsened");
        }
        if let Some(relic) = self.latest_formation() {
            self.log(format!("{}: 시작 진형을 적용했습니다.", relic.name()));
        }
        let occupied: Vec<_> = players.iter().filter_map(|p| p.position).collect();
        if self
            .current_node
            .is_some_and(|id| self.map.nodes[id].kind == NodeKind::Combat)
        {
            content::endless::defend_normal(&mut encounter, &occupied);
        }
        self.endless_scaling = (self.mode == RunMode::Endless).then(|| {
            content::endless::Scaling::for_node(
                self.seed,
                self.cleared_stages + 1,
                self.map.nodes[self.current_node.unwrap()].kind,
            )
        });
        if let Some(scaling) = &self.endless_scaling {
            scaling.deploy(&mut encounter, &occupied);
        }
        let haste = self.relics.contains(&Relic::Haste);
        let options = BattleOptions {
            enemy_racing_goal: self.relics.contains(&Relic::Omnipotence).then_some(7),
            actions_per_turn: if haste { HASTE_ACTIONS } else { 1 },
            self_defense: self.relics.contains(&Relic::SelfDefense),
            initiative: self.relics.contains(&Relic::Initiative),
            enemy_initiative: ENCOUNTERS[index]
                .has_start_rule(content::encounters::StartRule::EnemyInitiative),
        };
        self.battle =
            Battle::with_options(BattleStart { players, encounter }, options, &mut self.rng)
                .map_err(engine_error)?;
        if options.initiative && options.enemy_initiative {
            self.log("선공권: 양측 효과가 서로 상쇄되었습니다.");
        }
        if haste {
            self.log(format!("민첩성: 턴마다 최대 {HASTE_ACTIONS}회 이동합니다."));
        }
        self.phase = RunPhase::Combat;
        self.round = 1;
        self.cards = CardCombat::start_with_rule(
            &self.deck,
            &mut self.rng,
            content::characters::definition(&self.squad_id)
                .expect("registered squad")
                .turn_rule,
        );
        self.debug_event("combat_cards", None, None, "전투 카드 상태와 마나 초기화");
        if self.event_state.next_battle_penalty {
            if let Some(card) = self.cards.hand.pop() {
                self.cards.draw.push(card);
            }
        }
        self.battle.configure_v030(&self.relics);
        self.debug_event("relic_setup", None, None, "보유 유물의 전투 설정 적용");
        self.configure_expansion()?;
        self.debug_event("expansion_setup", None, None, "부대별 전투 시작 효과 적용");
        if let Some(scaling) = &self.endless_scaling {
            scaling.apply(&mut self.battle);
        }
        if self.event_state.next_battle_penalty {
            self.cards.mana = self.cards.mana.saturating_sub(1);
            self.event_state.next_battle_penalty = false;
        }
        self.resource_effects();
        self.events();
        self.check_finish()?;
        self.plan_boss();
        self.debug_event("battle_start_after", None, None, "전투 시작 처리 완료");
        Ok(())
    }
    pub(super) fn perform_action<F>(&mut self, select: F) -> Result<bool, String>
    where
        F: for<'a> FnOnce(&'a Battle) -> Result<Option<GeneratedAction<'a>>, Error>,
    {
        let side = self.battle.turn;
        let Some((action, piece, captured)) =
            self.battle.select_and_apply(select).map_err(engine_error)?
        else {
            return Ok(false);
        };
        let destination = action.steps().last().unwrap().move_to;
        let still_at = self
            .battle
            .board
            .piece_at(action.first.from)
            .ok()
            .flatten()
            .is_some_and(|p| p.id == piece.id);
        let (dx, dy) = (
            (destination.file - action.first.from.file).abs(),
            (destination.rank - action.first.from.rank).abs(),
        );
        let rule = if side != ENEMY {
            ""
        } else if piece.kind == content::pieces::UNICORN && !matches!((dx, dy), (1, 2) | (2, 1)) {
            " · 규칙: 빈칸 순간이동"
        } else if piece.kind == content::pieces::EMPEROR && (dx > 1 || dy > 1) {
            " · 규칙: 포획한 기물의 행마 흡수"
        } else if !action.continuation.is_empty() {
            " · 규칙: 특수 경로 행마"
        } else if still_at
            && action.first.effect == Effect::Catch
            && content::pieces::definition(piece.kind)
                .is_some_and(|d| d.description.contains("사격"))
        {
            " · 규칙: 제자리 사격 (명중·빗나감은 아래 기록)"
        } else if piece.kind == content::pieces::TEMPEST_PAWN && dy == 2 {
            " · 규칙: 템페스트 폰의 전방 두 칸 포획"
        } else {
            ""
        };
        self.log(format!(
            "{} {} · {} → {}{}{}",
            if side == PLAYER { "백" } else { "흑" },
            piece_name(piece.kind),
            square(action.first.from),
            square(destination),
            if still_at {
                " (제자리 판정)".to_string()
            } else {
                captured
                    .map(|p| format!(" · {} 포획", piece_name(p.kind)))
                    .unwrap_or_default()
            },
            rule
        ));
        if !still_at {
            if let Some(key) = action.steps().last().and_then(|s| s.transition.as_deref()) {
                if let Some(d) = content::pieces::ALL.iter().find(|d| d.key == key) {
                    self.log(format!("승격: {} → {}", piece_name(piece.kind), d.name));
                }
            }
        }
        self.events();
        self.check_finish()?;
        if self.phase == RunPhase::Combat && self.battle.turn != side {
            if side == PLAYER {
                self.cards.end_turn();
            } else {
                self.begin_player_turn();
            }
        }
        Ok(true)
    }
    pub(super) fn begin_player_turn(&mut self) {
        self.round = self.round.saturating_add(1);
        self.cards
            .begin_turn(&mut self.rng, self.relics.contains(&Relic::IceCream));
        self.plan_boss();
    }
    pub(super) fn plan_boss(&mut self) {
        self.boss_plans.clear();
        self.boss_sequence_step = 0;
        if self.phase != RunPhase::Combat || self.battle.boss().is_none() {
            return;
        }
        self.boss_plans = ai::choose_dictator_plan(&self.battle)
            .unwrap_or_default()
            .into_iter()
            .map(|(before, action)| Self::describe_boss_action(&before, action))
            .collect();
    }
    pub(super) fn describe_boss_action(battle: &Battle, action: DefaultAction) -> BossPlan {
        let from = action.first.from;
        let to = action.steps().last().unwrap().move_to;
        let tag = action.first.transition.as_deref();
        let copied = tag
            .and_then(|tag| tag.strip_prefix("boss_copy:"))
            .and_then(|tag| tag.split_once(':'))
            .and_then(|(kind, _)| kind.parse::<PieceType>().ok());
        let (mut area, mut target_positions) = battle.boss_shot_area(&action);
        if let Some(kind) = copied {
            if action.first.effect == Effect::Catch && action.first.take == from {
                area = battle.blast_preview(kind, from);
                target_positions = area
                    .iter()
                    .copied()
                    .filter(|&at| battle.board.piece_at(at).ok().flatten().is_some())
                    .collect();
            } else if action.first.effect == Effect::Catch && to == from {
                let (blast, paths) = battle.shot_effect_preview(kind, from, action.first.take);
                area.push(action.first.take);
                area.extend(blast);
                area.extend(paths.into_iter().flatten());
                target_positions.push(action.first.take);
            } else {
                area.push(to);
                if battle
                    .board
                    .piece_at(action.first.take)
                    .ok()
                    .flatten()
                    .is_some()
                {
                    target_positions.push(action.first.take);
                }
            }
        } else if tag != Some("boss_shot") && tag != Some("boss_reload") {
            area.push(to);
            if battle
                .board
                .piece_at(action.first.take)
                .ok()
                .flatten()
                .is_some()
            {
                target_positions.push(action.first.take);
            }
        }
        let targets: Vec<_> = target_positions
            .iter()
            .filter_map(|&at| {
                battle
                    .board
                    .piece_at(at)
                    .ok()
                    .flatten()
                    .map(|p| BossTargetView {
                        cell: cell(at),
                        chance: copied
                            .and_then(|kind| battle.boss_copied_hit_chance(kind, from, at))
                            .or_else(|| battle.hit_chance(from, at))
                            .unwrap_or(100),
                        name: piece_name(p.kind).into(),
                    })
            })
            .collect();
        let name = match tag {
            Some("boss_reload") => "샷건 재장전 · 행동 1회".into(),
            Some("boss_shot") => format!(
                "3갈래 샷건 · {} 방향 · 탄도별 첫 기물 공격",
                square(action.first.take)
            ),
            _ if copied.is_some() && action.first.effect == Effect::Catch => format!(
                "습득 능력 · {} · {}",
                piece_name(copied.unwrap()),
                if action.first.take == from {
                    "자폭".into()
                } else {
                    square(action.first.take)
                }
            ),
            _ if copied.is_some() => format!(
                "습득 행마 · {} · {}",
                piece_name(copied.unwrap()),
                square(to)
            ),
            _ if battle
                .board
                .piece_at(action.first.take)
                .ok()
                .flatten()
                .is_some_and(|p| p.owner == PLAYER) =>
            {
                format!("포획 · {}", square(action.first.take))
            }
            _ => format!("이동 · {}", square(to)),
        };
        BossPlan {
            action,
            name,
            area: area.into_iter().map(cell).collect(),
            targets,
        }
    }
    pub fn enemy_recommendation(&self) -> Result<Option<(usize, usize)>, String> {
        if self.phase != RunPhase::Combat || self.battle.turn != ENEMY {
            return Err("적의 턴이 아닙니다.".into());
        }
        if self.battle.boss().is_some() {
            let plan = self.boss_plans.get(usize::from(self.boss_sequence_step));
            return Ok(plan.and_then(|plan| {
                let from = plan.action.first.from;
                self.battle
                    .legal_actions(from)
                    .ok()?
                    .iter()
                    .position(|action| action == &plan.action)
                    .map(|index| (cell(from), index))
            }));
        }
        let budget = if self
            .endless_scaling
            .as_ref()
            .is_some_and(|s| s.modifiers.contains(&"전술 교리"))
        {
            Budget::boss()
        } else {
            match self.current_node.map(|id| self.map.nodes[id].kind) {
                Some(NodeKind::Boss) => Budget::boss(),
                Some(NodeKind::Elite) => Budget::elite(),
                _ => Budget::normal(),
            }
        };
        let choice = ai::choose(&self.battle, budget).map_err(engine_error)?;
        let Some(choice) = choice else {
            return Ok(None);
        };
        let actions = self
            .battle
            .legal_actions(choice.first.from)
            .map_err(engine_error)?;
        let index = actions
            .iter()
            .position(|a| a == &choice)
            .ok_or("AI가 선택한 행동이 현재 전투에서 유효하지 않습니다.")?;
        Ok(Some((cell(choice.first.from), index)))
    }
    pub(super) fn enemy_step(&mut self) -> Result<(), String> {
        let selected = self.enemy_recommendation()?;
        self.enemy_step_selected(selected.map(|s| s.0), selected.map(|s| s.1))
    }
    pub(super) fn enemy_step_selected(
        &mut self,
        from: Option<usize>,
        index: Option<usize>,
    ) -> Result<(), String> {
        if self.phase != RunPhase::Combat
            || self.battle.turn != ENEMY
            || self.battle.pending_choice().is_some()
            || from.is_some() != index.is_some()
        {
            return Err("적의 턴이 아닙니다.".into());
        }
        let boss_fight = self.battle.boss().is_some();
        let replaying = self.replaying_command;
        let planned = self
            .boss_plans
            .get(usize::from(self.boss_sequence_step))
            .map(|p| p.action.clone());
        let acted = self.perform_action(|battle| {
            let (Some(from), Some(index)) = (from, index) else {
                if boss_fight {
                    return Ok(None);
                }
                for (i, piece) in battle.board.cells().iter().enumerate() {
                    if piece.is_some_and(|p| p.owner == ENEMY)
                        && !battle
                            .legal_actions(pos(i).map_err(|_| Error::IllegalAction)?)?
                            .is_empty()
                    {
                        return Err(Error::IllegalAction);
                    }
                }
                return Ok(None);
            };
            let from = pos(from).map_err(|_| Error::IllegalAction)?;
            if boss_fight && !replaying {
                let Some(plan) = planned.as_ref() else {
                    return Ok(None);
                };
                if plan.first.from != from {
                    return Err(Error::IllegalAction);
                }
                return Ok(battle
                    .generated_legal_actions(from)
                    .unwrap_or_default()
                    .into_iter()
                    .find(|candidate| candidate.action() == plan));
            }
            battle
                .generated_legal_actions(from)?
                .into_iter()
                .nth(index)
                .map(Some)
                .ok_or(Error::IllegalAction)
        })?;
        if !acted {
            if boss_fight {
                if let Some(action) = &planned {
                    self.battle
                        .forfeit_boss_action(action)
                        .map_err(engine_error)?;
                } else {
                    self.battle.end_turn().map_err(engine_error)?;
                }
                self.boss_sequence_step = self.boss_sequence_step.saturating_add(1);
            } else {
                self.battle.end_turn().map_err(engine_error)?;
            }
            self.events();
            self.check_finish()?;
            if boss_fight && planned.is_some() && self.phase == RunPhase::Combat {
                self.log("예고된 적 행동이 불가능해져 실패했습니다.");
            }
            if self.phase == RunPhase::Combat && self.battle.turn == PLAYER {
                self.begin_player_turn();
                if !boss_fight {
                    self.log("적에게 합법적인 수가 없어 턴을 넘겼습니다. 전투는 계속됩니다.");
                }
            }
        } else if boss_fight && self.phase == RunPhase::Combat && self.battle.turn == ENEMY {
            self.boss_sequence_step = self.boss_sequence_step.saturating_add(1);
        }
        if boss_fight
            && self.phase == RunPhase::Combat
            && self.battle.turn == ENEMY
            && (self.legacy_boss_turn
                || usize::from(self.boss_sequence_step) >= self.boss_plans.len())
            && self.battle.pending_choice().is_none()
        {
            self.battle.end_turn().map_err(engine_error)?;
            self.events();
            self.check_finish()?;
            if self.phase == RunPhase::Combat {
                self.begin_player_turn();
            }
        }
        Ok(())
    }
}
