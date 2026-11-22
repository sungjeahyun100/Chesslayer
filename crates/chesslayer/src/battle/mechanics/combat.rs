//! combat rules. The call sites retain their original ordering.
use super::*;

impl Battle {
    pub(in crate::battle) fn counterattack_candidates(
        &self,
        enemy_id: PieceId,
        shooting: bool,
    ) -> Vec<PieceId> {
        let Some((enemy_at, _)) = self.piece_position(enemy_id) else {
            return vec![];
        };
        self.living(PLAYER)
            .into_iter()
            .filter(|(from, p)| {
                self.can_act(*p)
                    && if shooting {
                        self.special_actions(*from).iter().any(|a| {
                            a.first.take == enemy_at
                                && a.first.move_to == *from
                                && self.hit_chance(*from, enemy_at).is_some()
                        })
                    } else {
                        self.actions_from(*from, false).ok().is_some_and(|actions| {
                            actions.iter().any(|a| {
                                a.steps().any(|s| {
                                    s.take == enemy_at
                                        && s.move_to != *from
                                        && !matches!(s.effect, Effect::Move | Effect::Shift)
                                })
                            })
                        })
                    }
            })
            .map(|(_, p)| p.id)
            .collect()
    }
    pub(in crate::battle) fn free_counterattack(
        &mut self,
        ally_id: PieceId,
        enemy_id: PieceId,
        shooting: bool,
    ) -> Result<(), String> {
        let (from, p) = self
            .piece_position(ally_id)
            .ok_or("반격할 아군이 없습니다.")?;
        let (at, _) = self
            .piece_position(enemy_id)
            .ok_or("포획한 적이 없습니다.")?;
        let actions = if shooting {
            self.special_actions(from)
        } else {
            self.actions_from(from, false)
                .map_err(|e| format!("{e:?}"))?
        };
        let action = actions
            .into_iter()
            .find(|a| {
                a.steps()
                    .any(|s| s.take == at && !matches!(s.effect, Effect::Move | Effect::Shift))
            })
            .ok_or("반격 가능한 행마가 없습니다.")?;
        self.apply_effect(&action, ActionSource::PieceMove)
            .map_err(|e| format!("{e:?}"))?;
        self.forced_turn_end |= shooting;
        self.events.push(format!("{} 반격.", piece_name(p.kind)));
        Ok(())
    }
    pub(in crate::battle) fn apply_mechanics_action(
        &mut self,
        action: &DefaultAction,
    ) -> Result<bool, Error> {
        if self.boss_copied_action(action)? {
            return Ok(true);
        }
        if self.boss_action(action)? {
            return Ok(true);
        }
        let first = &action.first;
        if first.effect == Effect::Catch && first.move_to == first.from {
            if let Some(actor) = self.board.piece_at(first.from)? {
                let k = key(actor.kind);
                if matches!(k, "slaughter_cart" | "feast_cart")
                    && self.status(actor.id).corpse_bags > 0
                {
                    self.mechanics
                        .statuses
                        .entry(actor.id)
                        .or_default()
                        .corpse_bags -= 1;
                    if k == "slaughter_cart" {
                        let i = self.board.index(first.take).ok_or(Error::IllegalAction)?;
                        self.mechanics.plague[i] =
                            self.mechanics.rounds + 2 + u32::from(self.turn == ENEMY);
                        self.events
                            .push(format!("시체 투척: {}에 역병 생성.", square(first.take)));
                    } else {
                        let ids: Vec<_> = self
                            .living(actor.owner)
                            .into_iter()
                            .filter(|(pos, q)| {
                                q.id != actor.id
                                    && distance(*pos, first.from) == 1
                                    && self.status(q.id).blessing
                            })
                            .map(|(_, q)| q.id)
                            .collect();
                        for id in ids {
                            let _ = self.grant_move_rights(id, 1);
                        }
                        self.events
                            .push("고기다!: 주변 황충의 축복 아군에 행마권 1.".into());
                    }
                    return Ok(true);
                }
            }
        }
        let special = action.steps().any(|s| {
            let q = self.board.piece_at(s.take).ok().flatten();
            s.effect == Effect::Catch
                || q.is_some_and(|p| {
                    p.kind == pieces::CORPSE
                        || key(p.kind) == "warehouse"
                        || self.status(p.id).ingredient
                })
        });
        if !special {
            return Ok(false);
        }
        for step in action.steps() {
            let Some(captor) = self.board.piece_at(step.from)? else {
                break;
            };
            if step.effect == Effect::Catch
                && step.take == step.from
                && matches!(
                    captor.kind,
                    pieces::DRONE
                        | pieces::C4_DRONE
                        | pieces::MACHINE_GUN_DRONE
                        | pieces::TACTICAL_NUKE_DRONE
                        | pieces::GRENADE_LAUNCHER_DRONE
                )
            {
                let shape = match captor.kind {
                    pieces::C4_DRONE => BlastShape::King,
                    pieces::TACTICAL_NUKE_DRONE => BlastShape::Nuclear,
                    _ => BlastShape::Cross,
                };
                self.explode(
                    step.from,
                    shape,
                    captor.kind == pieces::TACTICAL_NUKE_DRONE,
                    Some(captor.id),
                )?;
                break;
            }
            let victim = self.board.piece_at(step.take)?;
            let capture = !matches!(step.effect, Effect::Move | Effect::Shift) && victim.is_some();
            if capture {
                let victim = victim.unwrap();
                let mut penetration_bonus = 0;
                if step.effect == Effect::Catch && victim.owner < 2 && is_lady(key(captor.kind)) {
                    self.make_corpse(step.take)?;
                    self.mechanics
                        .turn_used
                        .insert(format!("lady:{}", captor.id));
                    if key(captor.kind) == "banquet_lady" && captor.owner == PLAYER {
                        self.mechanics.pending_draw += 1;
                    }
                    if key(captor.kind) == "inviting_lady" {
                        self.hunger_near(step.from, captor.owner, 8);
                    }
                    self.events.push(format!(
                        "{}: 아군을 제물로 삼아 시체를 남겼습니다.",
                        piece_name(captor.kind)
                    ));
                    break;
                }
                if step.effect == Effect::Catch && shot_profile(key(captor.kind)).is_some() {
                    let chance = self.hit_chance(step.from, step.take).unwrap_or(0);
                    if captor.owner == PLAYER {
                        *self.mechanics.shot_counts.entry(captor.id).or_default() += 1;
                        self.mechanics.aimed.remove(&captor.id);
                        penetration_bonus =
                            self.mechanics.propellant.remove(&captor.id).unwrap_or(0);
                        if self.mechanics.fire_data_sharing
                            && self.mechanics.shared_target == Some(victim.id)
                        {
                            self.mechanics.shared_target = None;
                        }
                    }
                    if captor.owner == PLAYER
                        && matches!(
                            key(captor.kind),
                            "rifleman"
                                | "mortar"
                                | "machine_gun_drone"
                                | "grenade_launcher_drone"
                                | "tank"
                                | "he_tank"
                                | "ap_tank"
                                | "fragment_tank"
                                | "double_barrel"
                        )
                    {
                        let cost = if matches!(
                            key(captor.kind),
                            "mortar"
                                | "tank"
                                | "he_tank"
                                | "ap_tank"
                                | "fragment_tank"
                                | "double_barrel"
                        ) {
                            2
                        } else {
                            1
                        };
                        if self.mechanics.ammo < cost {
                            return Err(Error::IllegalAction);
                        }
                        self.mechanics.ammo -= cost;
                        self.mechanics.pending_ammo -= cost as i32;
                    }
                    let hit = self.roll_shot(chance);
                    if !hit {
                        self.events.push(format!(
                            "{} 사격 빗나감 (명중률 {}%).",
                            piece_name(captor.kind),
                            chance
                        ));
                        break;
                    }
                    self.events.push(format!(
                        "{} 사격 명중 ({}%).",
                        piece_name(captor.kind),
                        chance
                    ));
                    if captor.owner == PLAYER {
                        if self.mechanics.fire_data_sharing {
                            self.mechanics.shared_target = Some(victim.id);
                        }
                        self.mechanics.shot_hits_this_turn =
                            self.mechanics.shot_hits_this_turn.saturating_add(1);
                        if self.mechanics.relics.contains(&Relic::AmmunitionLine)
                            && self.mechanics.shot_hits_this_turn >= 2
                            && self.mechanics.turn_used.insert("ammunition_line".into())
                        {
                            if self.mechanics.player_artillery {
                                self.mechanics.pending_ammo += 1;
                            } else {
                                self.mechanics.pending_mana += 1;
                            }
                        }
                    }
                    if self.wards.remove(&victim.id) {
                        self.combat_events.push(CombatEvent::StatusTriggered {
                            piece_id: victim.id,
                            at: step.take,
                            status: "ward",
                        });
                        self.events.push(format!(
                            "가호: {}에게 명중한 사격을 한 번 막았습니다.",
                            piece_name(victim.kind)
                        ));
                        break;
                    }
                    if captor.owner == PLAYER
                        && self.mechanics.relics.contains(&Relic::Silencer)
                        && self.mechanics.used.insert("silencer".into())
                    {
                        self.grant_agility(captor.id, 1);
                    }
                }
                if victim.kind == pieces::CORPSE {
                    apply_default_action(&mut self.board, &DefaultAction::one(step.clone()))?;
                    self.boss_on_death(victim, step.take);
                    self.eat_corpse(captor.id)?;
                    self.forced_turn_end = true;
                } else if victim.kind != pieces::TRIBAL_DICTATOR && {
                    let damage = self.stats(captor.id).attack.max(0);
                    let stats = self.stats_mut(victim.id);
                    stats.health = stats.health.saturating_sub(damage);
                    stats.health > 0
                } || !self.boss_hit(victim, Some(captor.id))
                {
                    break;
                } else if self.status(victim.id).ingredient
                    && !self.is_royal_piece(victim.id)
                    && !(step.effect == Effect::Catch
                        && matches!(
                            captor.kind,
                            pieces::HE_TANK
                                | pieces::DOUBLE_BARREL
                                | pieces::GRENADE_LAUNCHER_DRONE
                                | pieces::AP_TANK
                                | pieces::FRAGMENT_TANK
                        ))
                {
                    self.make_corpse(step.take)?;
                    self.record_kill(captor, victim);
                    // A live ingredient capture ends here. The new corpse needs its own action.
                    if victim.owner == PLAYER && self.self_defense_target == Some(victim.id) {
                        self.kill_at(step.from, false)?;
                    }
                    break;
                } else {
                    apply_default_action(&mut self.board, &DefaultAction::one(step.clone()))?;
                    self.boss_on_death(victim, step.take);
                    self.record_kill(captor, victim);
                    if step.effect == Effect::Catch {
                        match captor.kind {
                            pieces::HE_TANK
                            | pieces::DOUBLE_BARREL
                            | pieces::GRENADE_LAUNCHER_DRONE => {
                                self.explode(step.take, BlastShape::Cross, false, Some(captor.id))?;
                            }
                            pieces::AP_TANK | pieces::FRAGMENT_TANK => {
                                let direction = (
                                    (step.take.file - step.from.file).signum(),
                                    (step.take.rank - step.from.rank).signum(),
                                );
                                let paths = if captor.kind == pieces::FRAGMENT_TANK {
                                    vec![direction, rotate45(direction, -1), rotate45(direction, 1)]
                                } else {
                                    vec![direction]
                                };
                                self.penetrate_paths(
                                    step.take,
                                    &paths,
                                    (if captor.kind == pieces::FRAGMENT_TANK {
                                        75u8
                                    } else {
                                        80u8
                                    })
                                    .saturating_add(penetration_bonus)
                                    .min(100),
                                    if captor.kind == pieces::FRAGMENT_TANK {
                                        25
                                    } else {
                                        20
                                    },
                                    None,
                                    captor,
                                )?;
                            }
                            _ => {}
                        }
                    }
                    if key(victim.kind) == "warehouse" {
                        self.warehouse_explosion(step.take, victim.owner)?;
                    }
                    if victim.owner == PLAYER && self.self_defense_target == Some(victim.id) {
                        if let Some((pos, _)) = self.piece_position(captor.id) {
                            self.kill_at(pos, false)?;
                        }
                    }
                }
            } else {
                apply_default_action(&mut self.board, &DefaultAction::one(step.clone()))?;
            }
        }
        self.sync_players();
        self.update_result(ResultReason::RoyalCaptured);
        Ok(true)
    }
    pub(in crate::battle) fn record_kill(&mut self, captor: Piece, victim: Piece) {
        if captor.owner == PLAYER && victim.owner == ENEMY {
            self.mechanics.killed_this_turn = true;
            self.initiative_turns_left = 0;
        }
        if captor.owner == ENEMY && victim.owner == PLAYER {
            self.enemy_initiative_turns_left = 0;
        }
        if captor.owner == ENEMY
            && victim.owner == PLAYER
            && !self.mechanics.temporary_pieces.contains(&victim.id)
        {
            if self.mechanics.mobilization_left > 0 {
                self.mechanics.mobilization_left -= 1;
                self.mechanics.pending_mana += 1;
            }
            self.mechanics.regular_losses = self.mechanics.regular_losses.saturating_add(1);
            if self.mechanics.relics.contains(&Relic::WarBonds)
                && self.mechanics.regular_losses % 3 == 0
            {
                self.mechanics.pending_draw += 1;
            }
            if self.turn == ENEMY && self.mechanics.reckless.remove(&victim.id) {
                self.mechanics.pending_mana -= 1;
            }
            if self.mechanics.relief_ready {
                self.mechanics.relief_ready = false;
                let cells = self.friendly_empty_cells(false);
                self.enqueue(
                    "교대 병력: 예비 폰을 배치할 칸 선택",
                    cells,
                    ChoiceEffect::ReservePawn,
                    false,
                );
            }
            if self.mechanics.revenge_ready {
                self.mechanics.revenge_ready = false;
                let targets = self.counterattack_candidates(captor.id, false);
                self.enqueue(
                    "전우의 복수: 반격할 아군 선택",
                    targets,
                    ChoiceEffect::Revenge(captor.id),
                    false,
                );
            }
            if self.mechanics.last_defense_ready {
                self.mechanics.last_defense_ready = false;
                let targets = self.counterattack_candidates(captor.id, true);
                self.enqueue(
                    "최후의 방어선: 반격할 사수 선택",
                    targets,
                    ChoiceEffect::LastDefense(captor.id),
                    false,
                );
            }
        }
        if captor.owner == PLAYER && victim.owner == ENEMY && self.mechanics.human_wave_left > 0 {
            self.mechanics.human_wave_left -= 1;
            let targets =
                self.living(PLAYER)
                    .into_iter()
                    .filter(|(from, p)| {
                        p.id != captor.id
                            && !self.mechanics.wave_movers.contains(&p.id)
                            && self.actions_from(*from, false).ok().is_some_and(|actions| {
                                actions.iter().any(|a| a.first.move_to != *from)
                            })
                    })
                    .map(|(_, p)| p.id)
                    .collect();
            self.enqueue(
                "인해전술: 추가 이동할 아군 선택",
                targets,
                ChoiceEffect::ExtraMove,
                false,
            );
        }
    }
    pub(in crate::battle) fn kill_at(
        &mut self,
        at: Position,
        ingredient: bool,
    ) -> Result<(), Error> {
        self.kill_at_by(at, ingredient, None).map(|_| ())
    }
    pub(in crate::battle) fn kill_at_by(
        &mut self,
        at: Position,
        ingredient: bool,
        attacker: Option<PieceId>,
    ) -> Result<bool, Error> {
        let Some(p) = self.board.piece_at(at)? else {
            return Ok(false);
        };
        if !self.boss_hit(p, attacker) {
            return Ok(false);
        }
        if !(ingredient
            && self.status(p.id).ingredient
            && p.kind != pieces::CORPSE
            && !self.is_royal_piece(p.id))
        {
            self.boss_on_death(p, at);
        }
        if p.kind == pieces::CORPSE {
            self.forced_turn_end = true;
        }
        if ingredient
            && self.status(p.id).ingredient
            && p.kind != pieces::CORPSE
            && !self.is_royal_piece(p.id)
        {
            self.make_corpse(at)?;
        } else {
            self.board.set_piece(at, None)?;
        }
        self.wards.remove(&p.id);
        self.limiter.remove(&p.id);
        if key(p.kind) == "warehouse" {
            self.warehouse_explosion(at, p.owner)?;
        }
        Ok(true)
    }
    pub(in crate::battle) fn warehouse_explosion(
        &mut self,
        at: Position,
        owner: PlayerId,
    ) -> Result<(), Error> {
        if owner == PLAYER {
            self.mechanics.pending_ammo -= 200;
        }
        self.events.push(format!(
            "{} 창고 폭발: 상하좌우 기물 소멸, 탄약 -200.",
            square(at)
        ));
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let p = Position::new(at.file + dx, at.rank + dy);
            if self.board.contains(p) {
                self.kill_at(p, true)?;
            }
        }
        Ok(())
    }
}
