//! choices rules. The call sites retain their original ordering.
use super::*;

impl Battle {
    pub fn pending_choice(&self) -> Option<EffectChoice> {
        self.mechanics.choices.front().map(|c| c.view.clone())
    }
    pub(in crate::battle) fn enqueue(
        &mut self,
        name: &str,
        targets: Vec<PieceId>,
        effect: ChoiceEffect,
        can_finish: bool,
    ) {
        if !targets.is_empty() {
            self.mechanics.choices.push_back(PendingChoice {
                view: EffectChoice {
                    name: name.into(),
                    targets,
                    can_finish,
                },
                effect,
            });
        }
    }
    pub(in crate::battle) fn choose_ally(
        &mut self,
        at: Position,
        owner: PlayerId,
        except: PieceId,
        range: i32,
        n: u8,
        label: &str,
    ) {
        if owner == PLAYER {
            let targets = self
                .living(owner)
                .into_iter()
                .filter(|(pos, p)| p.id != except && distance(*pos, at) <= range)
                .map(|(_, p)| p.id)
                .collect();
            self.enqueue(label, targets, ChoiceEffect::Agility(n), false);
        } else if let Some(id) = self.ally_near(at, owner, except, range) {
            self.grant_agility(id, n);
        }
    }
    pub(in crate::battle) fn queue_formation(&mut self) {
        let targets = self.living(PLAYER).into_iter().map(|(_, p)| p.id).collect();
        self.enqueue(
            "전선잡기: 재배치할 기물 선택 (완료 가능)",
            targets,
            ChoiceEffect::Formation,
            true,
        );
    }
    pub fn resolve_choice(&mut self, target: PieceId) -> Result<(), String> {
        let choice = self
            .mechanics
            .choices
            .front()
            .cloned()
            .ok_or("해결할 능력이 없습니다.")?;
        if target == 0 && choice.view.can_finish {
            self.mechanics.choices.pop_front();
        } else {
            if !choice.view.targets.contains(&target) {
                return Err("강조된 대상을 선택하세요.".into());
            }
            self.mechanics.choices.pop_front();
            match choice.effect {
                ChoiceEffect::Agility(n) => self.grant_agility(target, n),
                ChoiceEffect::MoveRights(n) => {
                    self.grant_move_rights(target, n)
                        .map_err(|e| format!("{e:?}"))?;
                }
                ChoiceEffect::Hunger => self.worsen_hunger(target),
                ChoiceEffect::RelieveHunger => {
                    let s = self.mechanics.statuses.entry(target).or_default();
                    s.hunger = s.hunger.saturating_sub(1);
                }
                ChoiceEffect::Madness(n) => {
                    let s = self.mechanics.statuses.entry(target).or_default();
                    s.madness = s.madness.saturating_add(n);
                    self.resolve_conscience(target)
                        .map_err(|e| format!("{e:?}"))?;
                }
                ChoiceEffect::Remove => {
                    if let Some((at, _)) = self.piece_position(target) {
                        self.kill_at(at, false).map_err(|e| format!("{e:?}"))?;
                    }
                }
                ChoiceEffect::Relocate(id) => {
                    let (from, p) = self.piece_position(id).ok_or("기물이 없습니다.")?;
                    let cell = target.checked_sub(10000).ok_or("빈 칸을 선택하세요.")?;
                    let to = Position::new((cell % 8) as i32, (cell / 8) as i32);
                    if self.board.piece_at(to).ok().flatten().is_some() {
                        return Err("빈 칸을 선택하세요.".into());
                    }
                    self.board
                        .set_piece(from, None)
                        .map_err(|e| format!("{e:?}"))?;
                    self.board
                        .set_piece(to, Some(p))
                        .map_err(|e| format!("{e:?}"))?;
                    self.apply_plague_on_entry(p.id);
                    if matches!(choice.effect, ChoiceEffect::Relocate(_)) {
                        self.queue_formation();
                    }
                }
                ChoiceEffect::Formation => {
                    let targets = self
                        .board
                        .cells()
                        .iter()
                        .enumerate()
                        .filter(|(_, p)| p.is_none())
                        .map(|(i, _)| 10000 + i as u32)
                        .collect();
                    self.enqueue(
                        "전선잡기: 이동할 빈 칸 선택",
                        targets,
                        ChoiceEffect::Relocate(target),
                        false,
                    );
                }
                ChoiceEffect::ReservePawn => {
                    let cell = target.checked_sub(10000).ok_or("빈 칸을 선택하세요.")?;
                    let at = Position::new((cell % 8) as i32, (cell / 8) as i32);
                    self.spawn_reserve_pawn(at)?;
                }
                ChoiceEffect::Revenge(enemy_id) => {
                    self.free_counterattack(target, enemy_id, false)?;
                }
                ChoiceEffect::LastDefense(enemy_id) => {
                    self.free_counterattack(target, enemy_id, true)?;
                }
                ChoiceEffect::ExtraMove => {
                    let (from, _) = self.piece_position(target).ok_or("기물이 없습니다.")?;
                    let destinations: Vec<_> = self
                        .actions_from(from, false)
                        .map_err(|e| format!("{e:?}"))?
                        .into_iter()
                        .filter(|a| a.first.move_to != from)
                        .filter_map(|a| {
                            a.steps()
                                .last()
                                .and_then(|s| self.board.index(s.move_to))
                                .map(|i| 10000 + i as u32)
                        })
                        .collect();
                    self.enqueue(
                        "인해전술: 이동할 칸 선택",
                        destinations,
                        ChoiceEffect::ExtraMoveDestination(target),
                        false,
                    );
                }
                ChoiceEffect::ExtraMoveDestination(id) => {
                    let (from, _) = self.piece_position(id).ok_or("기물이 없습니다.")?;
                    let cell = target.checked_sub(10000).ok_or("이동할 칸을 선택하세요.")?;
                    let at = Position::new((cell % 8) as i32, (cell / 8) as i32);
                    let action = self
                        .actions_from(from, false)
                        .map_err(|e| format!("{e:?}"))?
                        .into_iter()
                        .find(|a| {
                            a.steps()
                                .last()
                                .is_some_and(|s| s.move_to == at && s.move_to != from)
                        })
                        .ok_or("합법적인 이동을 선택하세요.")?;
                    self.mechanics.wave_movers.insert(id);
                    self.apply_effect(&action, ActionSource::PieceMove)
                        .map_err(|e| format!("{e:?}"))?;
                    self.moved_ids.insert(id);
                }
            }
        }
        self.sync_players();
        self.update_result(ResultReason::RoyalCaptured);
        if self.mechanics.choices.is_empty() && self.phase == Phase::Active {
            let side = self.turn;
            self.finish_forced_turn().map_err(|e| format!("{e:?}"))?;
            if self.turn == side && self.actions_left == 0 {
                self.end_turn().map_err(|e| format!("{e:?}"))?;
            }
        }
        Ok(())
    }
}
