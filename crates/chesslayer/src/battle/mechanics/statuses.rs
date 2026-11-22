//! statuses rules. The call sites retain their original ordering.
use super::*;

impl Battle {
    pub fn plague_cells(&self) -> Vec<(Position, u32)> {
        (0..64)
            .filter_map(|i| {
                let remaining = self.mechanics.plague[i].saturating_sub(self.mechanics.rounds);
                (remaining > 0)
                    .then_some((Position::new((i % 8) as i32, (i / 8) as i32), remaining))
            })
            .collect()
    }
    pub(in crate::battle) fn initialize_statuses(&mut self) {
        self.mechanics.statuses.clear();
        self.mechanics.pending_agility.clear();
        self.mechanics.player_artillery = self.players.iter().any(|p| key(p.kind) == "warehouse");
        self.mechanics.initial_allies =
            self.players.iter().filter(|p| p.position.is_some()).count();
        for p in &self.players {
            let get = |name: &str| p.persistent.get(name).copied().unwrap_or(0).max(0) as u8;
            self.mechanics.statuses.insert(
                p.id,
                PieceStatus {
                    hunger: get("hunger").min(3),
                    madness: get("madness"),
                    guilt: get("guilt"),
                    incapacitated: get("incapacitated"),
                    blessing: get("blessing") > 0,
                    death_threshold: get("death_threshold") > 0,
                    ..PieceStatus::default()
                },
            );
        }
        // Corpses are neutral, so ordinary Chesssembly capture rules support both sides.
        let corpses: Vec<_> = self
            .board
            .cells()
            .iter()
            .enumerate()
            .filter_map(|(i, p)| p.filter(|p| p.kind == pieces::CORPSE).map(|p| (i, p)))
            .collect();
        for (i, p) in corpses {
            self.board
                .set_piece(
                    Position::new(i as i32 % 8, i as i32 / 8),
                    Some(Piece { owner: 2, ..p }),
                )
                .expect("existing corpse");
        }
    }
    pub(in crate::battle) fn trigger_death_thresholds(&mut self) -> Result<(), Error> {
        let targets: Vec<_> = self
            .players
            .iter()
            .filter(|p| p.position.is_some() && self.status(p.id).death_threshold)
            .filter_map(|p| p.position.map(|pos| (p.id, pos)))
            .collect();
        for (id, pos) in targets {
            self.combat_events.push(CombatEvent::StatusTriggered {
                piece_id: id,
                at: pos,
                status: "death_threshold",
            });
            self.mechanics
                .statuses
                .entry(id)
                .or_default()
                .death_threshold = false;
            if let Some(p) = self.players.iter_mut().find(|p| p.id == id) {
                p.persistent.remove("death_threshold");
            }
            self.kill_at(pos, false)?;
            self.events.push(format!(
                "죽음의 문턱: {}의 기물이 전투 시작 시 사망했습니다.",
                square(pos)
            ));
        }
        self.sync_players();
        Ok(())
    }
    pub fn status(&self, id: PieceId) -> PieceStatus {
        self.mechanics
            .statuses
            .get(&id)
            .cloned()
            .unwrap_or_default()
    }
    pub fn status_labels(&self, id: PieceId) -> Vec<String> {
        let s = self.status(id);
        let mut result = vec![];
        if s.ingredient {
            result.push("식재료".into());
        }
        if s.blessing {
            result.push("황충의 축복".into());
        }
        if s.hunger > 0 {
            result.push(["정상", "공복", "굶주림", "기아"][usize::from(s.hunger.min(3))].into());
        }
        for (label, n) in [
            ("광기", s.madness),
            ("죄책감", s.guilt),
            ("민첩", s.agility),
            ("약화", s.weakness),
        ] {
            if n > 0 {
                result.push(format!("{label} {n}"));
            }
        }
        if s.incapacitated > 0 {
            result.push(format!("전투불능 {}", s.incapacitated));
        }
        if s.pest > 0 {
            result.push(format!("페스트 {}", s.pest));
        }
        if s.corpse_bags > 0 {
            result.push(format!("시체자루 {}", s.corpse_bags));
        }
        if s.spirit_bomb {
            result.push("원기옥".into());
        }
        if s.dining {
            result.push("식사의 예법".into());
        }
        if s.death_threshold {
            result.push("죽음의 문턱".into());
        }
        if self.mechanics.bell_targets.contains(&Some(id)) {
            result.push("다음 수 만찬의 종소리".into());
        }
        if self.mechanics.temporary_pieces.contains(&id) {
            result.push("임시 기물".into());
        }
        if let Some(bonus) = self.mechanics.aimed.get(&id) {
            result.push(format!("다음 사격 명중률 +{bonus}%p"));
        }
        if let Some(bonus) = self.mechanics.propellant.get(&id) {
            result.push(format!("다음 관통 +{bonus}%p"));
        }
        if self.mechanics.entrenched_pending.contains(&id) {
            result.push("진지 유지 중".into());
        }
        if self.mechanics.entrenched_ready.contains(&id) {
            result.push("방어 진지 강화".into());
        }
        if self.mechanics.suppressed.contains(&id) {
            result.push("이동 제압".into());
        }
        if self.mechanics.reckless.contains(&id) {
            result.push("무모한 돌격".into());
        }
        result
    }
    pub(in crate::battle) fn agility(&self, id: PieceId) -> u8 {
        self.mechanics.statuses.get(&id).map_or(0, |s| s.agility)
    }
    pub fn grant_agility(&mut self, id: PieceId, n: u8) {
        let owner = self.piece_position(id).map(|(_, p)| p.owner);
        let st = self.mechanics.statuses.entry(id).or_default();
        let before = st.agility;
        st.agility = st.agility.saturating_add(n);
        if owner == Some(self.turn) {
            self.actions_left = self.actions_left.saturating_add(st.agility - before);
        }
    }
    pub(in crate::battle) fn worsen_hunger(&mut self, id: PieceId) {
        let s = self.mechanics.statuses.entry(id).or_default();
        s.hunger = s.hunger.saturating_add(1).min(3);
    }
    pub(in crate::battle) fn filter_hunger(&self, p: Piece, actions: &mut Vec<DefaultAction>) {
        let hunger = self.status(p.id).hunger;
        if hunger < 2 {
            return;
        }
        let corpse = |a: &DefaultAction| {
            a.steps().any(|s| {
                !matches!(s.effect, Effect::Move | Effect::Shift)
                    && self
                        .board
                        .piece_at(s.take)
                        .ok()
                        .flatten()
                        .is_some_and(|p| p.kind == pieces::CORPSE)
            })
        };
        if !actions.iter().any(corpse) {
            return;
        }
        actions.retain(|a| {
            corpse(a)
                || (hunger == 2
                    && a.steps().all(|s| {
                        matches!(s.effect, Effect::Move | Effect::Shift)
                            || self.board.piece_at(s.take).ok().flatten().is_none()
                    }))
        });
    }
    pub(in crate::battle) fn persistent_outcome(&self) -> Vec<PlayerPiece> {
        let mut out = self.players.clone();
        for p in &mut out {
            if let Some(s) = self.mechanics.statuses.get(&p.id) {
                for (k, n) in [
                    ("hunger", i32::from(s.hunger)),
                    ("madness", i32::from(s.madness)),
                    ("guilt", i32::from(s.guilt)),
                    ("incapacitated", i32::from(s.incapacitated)),
                    ("blessing", i32::from(s.blessing)),
                    ("death_threshold", i32::from(s.death_threshold)),
                ] {
                    if n > 0 {
                        p.persistent.insert(k.into(), n);
                    } else {
                        p.persistent.remove(k);
                    }
                }
            }
        }
        out
    }
    pub(in crate::battle) fn finish_persistent_statuses(&mut self) {
        if self.mechanics.finished {
            return;
        }
        self.mechanics.finished = true;
        self.mechanics.choices.clear();
        for p in &mut self.players {
            let s = self.mechanics.statuses.entry(p.id).or_default();
            if p.position.is_some() && s.hunger > 0 && s.hunger < 3 {
                s.hunger += 1;
                p.persistent.insert("hunger_worsened".into(), 1);
            }
        }
    }
    pub(in crate::battle) fn finish_mechanics_turn(&mut self) -> Result<(), Error> {
        // Fallout damages the side whose turn is ending. A newly created cloud
        // remains through two complete enemy-to-enemy round boundaries.
        let hazardous: Vec<_> = self
            .living(self.turn)
            .into_iter()
            .filter(|(pos, _)| self.fallout_remaining(*pos) > 0)
            .collect();
        for (pos, piece) in hazardous {
            self.kill_at(pos, false)?;
            self.events
                .push(format!("방사성 낙진: {} 제거.", piece_name(piece.kind)));
        }
        if !self.sandbox {
            self.sync_players();
            self.update_result(ResultReason::RoyalCaptured);
            if self.phase != Phase::Active {
                return Ok(());
            }
        }
        if !self.sandbox
            && self.turn == PLAYER
            && self.mechanics.last_stand
            && !self.mechanics.killed_this_turn
        {
            self.result = GameResult::Win {
                winner: ENEMY,
                reason: ResultReason::RoyalCaptured,
            };
            self.phase = Phase::Finished;
            self.finish_persistent_statuses();
            self.events
                .push("마지막 저항: 이번 턴 적을 처치하지 못해 패배했습니다.".into());
            return Ok(());
        }
        let infected: Vec<_> = self
            .living(PLAYER)
            .into_iter()
            .chain(self.living(ENEMY))
            .map(|(at, p)| (at, p, self.status(p.id).pest))
            .collect();
        for (at, p, stacks) in infected {
            if stacks == 0 {
                continue;
            }
            self.mechanics.statuses.entry(p.id).or_default().pest = stacks - 1;
            let neighbors: Vec<_> = self
                .board
                .cells()
                .iter()
                .flatten()
                .copied()
                .filter(|q| q.kind != pieces::CORPSE && q.id != p.id)
                .filter(|q| {
                    self.piece_position(q.id)
                        .is_some_and(|(pos, _)| distance(at, pos) == 1)
                })
                .map(|q| q.id)
                .collect();
            for target in neighbors {
                if self.mechanics.rng.index(2) == 0 {
                    self.mechanics.statuses.entry(target).or_default().pest = 3;
                }
            }
            if stacks == 1 {
                if let Some((pos, _)) = self.piece_position(p.id) {
                    self.kill_at(pos, false)?;
                }
            }
        }
        if !self.sandbox {
            self.sync_players();
            self.update_result(ResultReason::RoyalCaptured);
            if self.phase != Phase::Active {
                return Ok(());
            }
        }
        let ids: Vec<_> = self.living(self.turn).iter().map(|(_, p)| p.id).collect();
        for id in ids {
            let status = self.mechanics.statuses.entry(id).or_default();
            status.agility = 0;
            if status.incapacitated_fresh {
                status.incapacitated_fresh = false;
            } else {
                status.incapacitated = status.incapacitated.saturating_sub(1);
            }
        }
        if self.turn == PLAYER {
            self.mechanics
                .entrenched_pending
                .retain(|id| !self.mechanics.turn_moved.contains(id));
            self.mechanics.entrenched_ready.clear();
            self.mechanics.human_wave_left = 0;
            self.mechanics.wave_movers.clear();
            self.mechanics.fire_data_sharing = false;
            self.mechanics.shared_target = None;
            self.mechanics.endless = false;
            self.mechanics.last_stand = false;
            self.mechanics.killed_this_turn = false;
        }
        self.mechanics.turn_used.clear();
        self.mechanics.shot_hits_this_turn = 0;
        self.mechanics.turn_moved.clear();
        if self.turn == ENEMY {
            self.mechanics.suppressed.clear();
            self.mechanics.rounds += 1;
            for expires in &mut self.mechanics.fallout {
                if *expires <= self.mechanics.rounds {
                    *expires = 0;
                }
            }
            for expires in &mut self.mechanics.plague {
                if *expires <= self.mechanics.rounds {
                    *expires = 0;
                }
            }
            let n = self.mechanics.rounds;
            // Resolve the enemy's bell first to preserve its established random choice.
            for guest_owner in [ENEMY, PLAYER] {
                if !self
                    .living(guest_owner)
                    .iter()
                    .any(|(_, p)| key(p.kind) == "eternal_guest")
                {
                    self.mechanics.bell_targets[usize::from(guest_owner)] = None;
                    continue;
                }
                if matches!(n, 2 | 5) {
                    let mut targets =
                        self.living(if guest_owner == PLAYER { ENEMY } else { PLAYER });
                    targets.retain(|(_, p)| !self.is_royal_piece(p.id));
                    let highest = targets
                        .iter()
                        .map(|(_, p)| self.status(p.id).hunger)
                        .max()
                        .unwrap_or(0);
                    targets.retain(|(_, p)| self.status(p.id).hunger == highest);
                    if !targets.is_empty() {
                        let draw = self.mechanics.rng.index(targets.len());
                        let selected = if guest_owner == ENEMY {
                            self.mechanics.search_random_arity = Some(targets.len());
                            self.mechanics.search_random_index.take().unwrap_or(draw)
                                % targets.len()
                        } else {
                            draw
                        };
                        let (_, p) = targets[selected];
                        self.mechanics.bell_targets[usize::from(guest_owner)] = Some(p.id);
                        self.events.push(format!(
                            "만찬의 종소리: 다음 수 {}의 허기가 악화됩니다.",
                            piece_name(p.kind)
                        ));
                    }
                }
                if matches!(n, 3 | 6) {
                    if let Some(id) = self.mechanics.bell_targets[usize::from(guest_owner)].take() {
                        if self.piece_position(id).is_some() {
                            self.worsen_hunger(id);
                            self.events
                                .push("만찬의 종소리: 표시된 기물의 허기가 악화되었습니다.".into());
                        }
                    }
                }
            }
        }
        Ok(())
    }
    pub(in crate::battle) fn start_mechanics_turn(&mut self) -> Result<(), Error> {
        if self.turn == ENEMY {
            self.boss_pull_hook()?;
        }
        if self.turn == PLAYER {
            self.mechanics.mobilization_left = 0;
            self.mechanics.revenge_ready = false;
            self.mechanics.relief_ready = false;
            self.mechanics.last_defense_ready = false;
            self.mechanics.reckless.clear();
            self.mechanics.entrenched_ready =
                std::mem::take(&mut self.mechanics.entrenched_pending);
        }
        let targets = self.living(self.turn);
        for (pos, p) in targets {
            let weakness = self.status(p.id).weakness;
            if weakness == 0 {
                continue;
            }
            self.mechanics.statuses.entry(p.id).or_default().weakness = 0;
            let mut kind = p.kind;
            let mut dead = false;
            for step in 0..weakness {
                let recorded_parent =
                    self.players
                        .iter()
                        .find(|owned| owned.id == p.id)
                        .and_then(|owned| {
                            pieces::upgrade_lineage(owned.kind, &owned.upgrades)
                                .get(step as usize + 1)
                                .copied()
                        });
                if let Some(parent) = recorded_parent.or_else(|| pieces::previous_upgrade(kind)) {
                    kind = parent;
                } else {
                    dead = true;
                    break;
                }
            }
            if dead {
                self.kill_at(pos, false)?;
            } else {
                self.board.set_piece(pos, Some(Piece { kind, ..p }))?;
                if let Some(owned) = self.players.iter_mut().find(|q| q.id == p.id) {
                    owned.kind = kind;
                    owned
                        .upgrades
                        .truncate(owned.upgrades.len().saturating_sub(usize::from(weakness)));
                }
            }
        }
        self.sync_players();
        self.update_result(ResultReason::RoyalCaptured);
        Ok(())
    }
    pub(in crate::battle) fn make_corpse(&mut self, at: Position) -> Result<(), Error> {
        let p = self.board.piece_at(at)?.ok_or(Error::IllegalAction)?;
        self.combat_stats.remove(&p.id);
        self.boss_on_death(p, at);
        self.board.set_piece(
            at,
            Some(Piece {
                kind: pieces::CORPSE,
                owner: 2,
                ..p
            }),
        )?;
        self.wards.remove(&p.id);
        self.limiter.remove(&p.id);
        self.sync_players();
        Ok(())
    }
    pub(in crate::battle) fn apply_plague_on_entry(&mut self, id: PieceId) {
        if let Some((at, p)) = self.piece_position(id) {
            if p.kind == pieces::CORPSE {
                return;
            }
            if self
                .board
                .index(at)
                .is_some_and(|i| self.mechanics.plague[i] > self.mechanics.rounds)
            {
                let s = self.mechanics.statuses.entry(id).or_default();
                s.ingredient = true;
                s.pest = 3;
                self.events
                    .push(format!("역병: {}에 식재료와 페스트 3.", piece_name(p.kind)));
            }
        }
    }
    pub(in crate::battle) fn hunger_near(&mut self, at: Position, owner: PlayerId, range: i32) {
        if owner == PLAYER {
            let targets = self
                .living(ENEMY)
                .into_iter()
                .filter(|(pos, p)| distance(*pos, at) <= range && !self.is_royal_piece(p.id))
                .map(|(_, p)| p.id)
                .collect();
            self.enqueue(
                "허기를 악화시킬 적 선택",
                targets,
                ChoiceEffect::Hunger,
                false,
            );
            return;
        }
        if let Some((_, p)) = self
            .living(if owner == PLAYER { ENEMY } else { PLAYER })
            .into_iter()
            .filter(|(pos, p)| distance(*pos, at) <= range && !self.is_royal_piece(p.id))
            .min_by_key(|(pos, p)| (distance(*pos, at), p.id))
        {
            self.worsen_hunger(p.id);
        }
    }
    pub(in crate::battle) fn ally_near(
        &self,
        at: Position,
        owner: PlayerId,
        except: PieceId,
        range: i32,
    ) -> Option<PieceId> {
        self.living(owner)
            .into_iter()
            .filter(|(pos, p)| p.id != except && distance(*pos, at) <= range)
            .min_by_key(|(pos, p)| (distance(*pos, at), p.id))
            .map(|(_, p)| p.id)
    }
    pub(in crate::battle) fn eat_corpse(&mut self, id: PieceId) -> Result<(), Error> {
        let Some((at, p)) = self.piece_position(id) else {
            return Ok(());
        };
        let owner = p.owner;
        if owner > 1 {
            return Ok(());
        }
        self.mechanics.statuses.entry(id).or_default().hunger = 0;
        if self.status(id).blessing {
            let pending = self.mechanics.pending_agility.entry(id).or_default();
            *pending = pending.saturating_add(3);
            self.events.push(format!(
                "{} 과흥분: 다음 자기 진영 턴에 민첩 3.",
                piece_name(p.kind)
            ));
        } else if !matches!(
            key(p.kind),
            "frenzied_dancer"
                | "masked_dancer"
                | "ecstatic_dancer"
                | "tempting_dancer"
                | "intoxicated_priest"
                | "feast_priest"
                | "locust_high_priest"
                | "hunger_priest"
        ) {
            let draw = self.mechanics.rng.index(2);
            let selected = self.mechanics.search_random_index.take().unwrap_or(draw) % 2;
            self.mechanics.search_random_arity = Some(2);
            if selected == 0 {
                self.mechanics.statuses.entry(id).or_default().madness += 1;
            } else {
                self.mechanics.statuses.entry(id).or_default().guilt += 1;
            }
            self.resolve_conscience(id)?;
        }
        self.mechanics.corpse_eaten[usize::from(owner)] += 1;
        let eaten = self.mechanics.corpse_eaten[usize::from(owner)];
        let k = key(p.kind);
        {
            if is_cart(k) && owner == PLAYER {
                self.mechanics.pending_draw += 1;
            }
            if is_cart(k) && owner == PLAYER {
                self.mechanics.pending_mana += 1;
            }
            if matches!(k, "glutton_follower" | "masked_dancer") {
                let _ = self.grant_move_rights(id, 1);
            }
            if k == "ecstatic_dancer" {
                let _ = self.grant_move_rights(id, 2);
            }
            if matches!(k, "slaughter_cart" | "feast_cart") {
                self.mechanics.statuses.entry(id).or_default().corpse_bags =
                    self.status(id).corpse_bags.saturating_add(1);
            }
            if is_priest(k) {
                if k == "intoxicated_priest" {
                    let targets: Vec<_> =
                        self.living(owner).into_iter().map(|(_, q)| q.id).collect();
                    if !targets.is_empty() {
                        let target = targets[self.mechanics.rng.index(targets.len())];
                        let _ = self.grant_move_rights(target, 1);
                    }
                } else {
                    let targets: Vec<_> =
                        self.living(owner).into_iter().map(|(_, q)| q.id).collect();
                    if owner == PLAYER {
                        self.enqueue(
                            "사제: 행마권을 받을 아군 선택",
                            targets,
                            ChoiceEffect::MoveRights(if k == "locust_high_priest" { 2 } else { 1 }),
                            false,
                        );
                    } else if let Some(target) = targets.first() {
                        let _ = self.grant_move_rights(
                            *target,
                            if k == "locust_high_priest" { 2 } else { 1 },
                        );
                    }
                }
            }
            if k == "inviting_follower" && self.mechanics.used.insert(format!("invite:{id}")) {
                self.hunger_near(at, owner, 2);
            }
            if k == "hunger_priest" {
                let targets: Vec<_> = self
                    .living(if owner == PLAYER { ENEMY } else { PLAYER })
                    .into_iter()
                    .filter(|(_, q)| !self.is_royal_piece(q.id))
                    .map(|(_, q)| q.id)
                    .collect();
                if owner == PLAYER {
                    self.enqueue(
                        "허기를 약화시킬 적 선택",
                        targets,
                        ChoiceEffect::RelieveHunger,
                        false,
                    );
                } else if let Some(target) = targets.first() {
                    self.mechanics.statuses.entry(*target).or_default().hunger =
                        self.status(*target).hunger.saturating_sub(1);
                }
            }
            if k == "tempting_dancer" {
                let targets: Vec<_> = self
                    .living(if owner == PLAYER { ENEMY } else { PLAYER })
                    .into_iter()
                    .filter(|(_, q)| !self.is_royal_piece(q.id))
                    .map(|(_, q)| q.id)
                    .collect();
                if owner == PLAYER {
                    self.enqueue(
                        "광기 2를 받을 적 선택",
                        targets,
                        ChoiceEffect::Madness(2),
                        false,
                    );
                } else if let Some(target) = targets.first() {
                    self.mechanics.statuses.entry(*target).or_default().madness =
                        self.status(*target).madness.saturating_add(2);
                    self.resolve_conscience(*target)?;
                }
            }
        }
        let kings: Vec<_> = self
            .living(owner)
            .into_iter()
            .filter(|(_, q)| matches!(key(q.kind), "smiling_king" | "toast_lord" | "ecstatic_lord"))
            .collect();
        for (pos, q) in kings {
            let k = key(q.kind);
            let range = match k {
                "smiling_king" => 2,
                "toast_lord" => 3,
                _ => 8,
            };
            if distance(at, pos) <= range && self.mechanics.used.insert(format!("king:{}", q.id)) {
                if owner == PLAYER {
                    self.mechanics.pending_draw += 1;
                }
                if k != "smiling_king" {
                    self.grant_agility(id, 1);
                }
                if k == "ecstatic_lord" && owner == PLAYER {
                    self.mechanics.pending_mana += 1;
                }
            }
        }
        if eaten <= 3
            && self
                .living(owner)
                .iter()
                .any(|(_, p)| key(p.kind) == "eternal_guest")
        {
            self.grant_agility(id, 1);
        }
        if owner == PLAYER {
            if self.mechanics.relics.contains(&Relic::SilverCutlery) && eaten == 1 {
                self.mechanics.pending_draw += 1;
            }
            if self.mechanics.relics.contains(&Relic::EmptyGoblet)
                && self.mechanics.turn_used.insert("goblet".into())
            {
                self.mechanics.pending_mana += 1;
            }
            if self.mechanics.relics.contains(&Relic::GoldenTray)
                && self.mechanics.turn_used.insert("tray".into())
            {
                self.choose_ally(at, owner, id, 1, 1, "금박 쟁반: 민첩을 받을 아군 선택");
            }
            if self.mechanics.relics.contains(&Relic::SmilingMask)
                && self.mechanics.used.insert("smiling_mask".into())
            {
                self.hunger_near(at, owner, 8);
            }
            if self.mechanics.relics.contains(&Relic::EternalFeastBell) && eaten == 3 {
                self.mechanics.pending_draw += 2;
                self.mechanics.pending_mana += 1;
            }
            if self.mechanics.endless && !self.is_royal_piece(id) {
                self.grant_agility(id, 1);
            }
        }
        Ok(())
    }
    pub(in crate::battle) fn resolve_conscience(&mut self, id: PieceId) -> Result<(), Error> {
        let mut switched = false;
        {
            let s = self.mechanics.statuses.entry(id).or_default();
            if s.madness >= 3 {
                s.madness = 0;
                s.blessing = true;
                switched = true;
            }
            if s.guilt >= 2 {
                s.guilt = 0;
                s.incapacitated = s.incapacitated.saturating_add(1);
                s.incapacitated_fresh = true;
            }
        }
        if switched {
            if let Some((pos, p)) = self.piece_position(id) {
                self.board.set_piece(
                    pos,
                    Some(Piece {
                        owner: if p.owner == PLAYER { ENEMY } else { PLAYER },
                        ..p
                    }),
                )?;
                if p.owner == PLAYER {
                    if let Some(owned) = self.players.iter_mut().find(|q| q.id == id) {
                        owned.persistent.insert("defected".into(), 1);
                    }
                }
                self.events.push(format!(
                    "광기: {}이 황충의 축복을 받고 상대 진영으로 전향했습니다.",
                    piece_name(p.kind)
                ));
            }
        }
        if self.status(id).incapacitated > 0 {
            let lost = self.agility(id);
            self.mechanics.statuses.entry(id).or_default().agility = 0;
            if self
                .piece_position(id)
                .is_some_and(|(_, p)| p.owner == self.turn)
            {
                self.actions_left = self.actions_left.saturating_sub(lost);
            }
            self.events
                .push("죄책감: 전투불능. 자기 턴 종료 시 중첩이 1 감소합니다.".into());
        }
        Ok(())
    }
}
