//! effects rules. The call sites retain their original ordering.
use super::*;

impl Battle {
    pub(in crate::battle) fn warded_victim(
        &self,
        action: &DefaultAction,
        owner: PlayerId,
    ) -> Result<Option<Piece>, Error> {
        if action.continuation.is_empty() {
            let step = &action.first;
            return Ok(
                if matches!(step.effect, crate::Effect::Shift | crate::Effect::Move) {
                    None
                } else {
                    self.board
                        .piece_at(step.take)?
                        .filter(|p| p.owner != owner && self.wards.contains(&p.id))
                },
            );
        }
        let mut board = self.board.clone();
        for step in action.steps() {
            if !matches!(step.effect, crate::Effect::Shift | crate::Effect::Move) {
                if let Some(victim) = board
                    .piece_at(step.take)?
                    .filter(|p| p.owner != owner && self.wards.contains(&p.id))
                {
                    return Ok(Some(victim));
                }
            }
            apply_default_action(&mut board, &DefaultAction::one(step.clone()))?;
        }
        Ok(None)
    }
    pub fn grant_limiter(&mut self, id: PieceId) -> Result<(), Error> {
        if self.phase != Phase::Active {
            return Err(Error::Finished);
        }
        if self.limiter.contains_key(&id) {
            return Err(Error::IllegalAction);
        }
        let piece = self
            .board
            .cells()
            .iter()
            .flatten()
            .find(|p| p.id == id)
            .ok_or(Error::IllegalAction)?;
        let owner = piece.owner;
        self.limiter.insert(
            id,
            LimiterState {
                owner,
                moves_left: 3,
            },
        );
        self.grant_death_threshold(id)?;
        if owner == self.turn {
            self.limiter_turn = true;
            self.actions_left = self.actions_left.saturating_add(3);
        }
        Ok(())
    }
    pub fn grant_death_threshold(&mut self, id: PieceId) -> Result<(), Error> {
        if self.phase != Phase::Active {
            return Err(Error::Finished);
        }
        if !self
            .board
            .cells()
            .iter()
            .flatten()
            .any(|piece| piece.id == id)
        {
            return Err(Error::IllegalAction);
        }
        self.mechanics
            .statuses
            .entry(id)
            .or_default()
            .death_threshold = true;
        Ok(())
    }
    pub fn apply_resolved_card_effect(&mut self, action: &DefaultAction) -> Result<(), Error> {
        if self.phase != Phase::Active {
            return Err(Error::Finished);
        }
        let mut next = self.clone();
        next.apply_effect(action, ActionSource::ResolvedCardEffect)?;
        *self = next;
        Ok(())
    }
    pub fn has_ward(&self, id: PieceId) -> bool {
        self.wards.contains(&id)
    }
    pub fn has_self_defense(&self, id: PieceId) -> bool {
        self.self_defense_target == Some(id)
    }
    pub fn initiative_turns_left(&self) -> u8 {
        self.initiative_turns_left
    }
    pub fn enemy_initiative_turns_left(&self) -> u8 {
        self.enemy_initiative_turns_left
    }
    pub fn is_enemy_royal(&self, id: PieceId) -> bool {
        id == self.royal_id || self.additional_royal_ids.contains(&id)
    }
    pub fn grant_ward(&mut self, id: PieceId) -> Result<(), Error> {
        if self.phase != Phase::Active {
            return Err(Error::Finished);
        }
        if !self.board.cells().iter().flatten().any(|p| p.id == id) {
            return Err(Error::IllegalAction);
        }
        self.wards.insert(id);
        Ok(())
    }
    pub fn remove_nonroyals(&mut self, ids: &[PieceId]) -> Result<usize, Error> {
        self.remove_pieces(ids, false)
    }
    pub(in crate::battle) fn remove_for_limiter(&mut self, ids: &[PieceId]) -> Result<(), Error> {
        self.remove_pieces(ids, true)?;
        self.update_result(ResultReason::RoyalCaptured);
        Ok(())
    }
    pub(in crate::battle) fn remove_pieces(
        &mut self,
        ids: &[PieceId],
        include_royals: bool,
    ) -> Result<usize, Error> {
        if self.phase != Phase::Active {
            return Err(Error::Finished);
        }
        let mut removed = 0;
        for rank in 0..8 {
            for file in 0..8 {
                let pos = Position::new(file, rank);
                if let Some(piece) = self.board.piece_at(pos)? {
                    if ids.contains(&piece.id)
                        && (include_royals
                            || (piece.id != self.king_id && !self.is_enemy_royal(piece.id))
                            || (self.boss.is_some() && piece.id == self.royal_id))
                    {
                        if !self.boss_hit(piece, None) {
                            continue;
                        }
                        self.board.set_piece(pos, None)?;
                        self.boss_on_death(piece, pos);
                        self.wards.remove(&piece.id);
                        self.limiter.remove(&piece.id);
                        removed += 1;
                    }
                }
            }
        }
        self.sync_players();
        if self.boss.is_some() {
            self.update_result(ResultReason::RoyalCaptured);
        }
        Ok(removed)
    }
    pub(in crate::battle) fn apply_effect(
        &mut self,
        action: &DefaultAction,
        source: ActionSource,
    ) -> Result<(), Error> {
        if source == ActionSource::PieceMove
            && action
                .steps()
                .last()
                .is_some_and(|s| s.move_to != action.first.from)
        {
            if let Some(p) = self.board.piece_at(action.first.from)? {
                self.mechanics.turn_moved.insert(p.id);
            }
        }
        if source == ActionSource::PieceMove && self.apply_mechanics_action(action)? {
            return Ok(());
        }
        // Card effects need royal-protection rollback; compound moves need a staging board
        // while rule effects inspect each intermediate position. A single piece step can
        // use chessembly's changed-square rollback directly on the live board.
        let mut staged = (source == ActionSource::ResolvedCardEffect
            || !action.continuation.is_empty()
            || self.boss.is_some())
        .then(|| self.board.clone());
        let next = staged.as_mut().unwrap_or(&mut self.board);
        let mut retaliation = None;
        let mut player_captured = false;
        let mut enemy_captured = false;
        let mut absorbed = vec![];
        let mut captures = vec![];
        let mut corpse_captured = false;
        let self_defense_target = self.self_defense_target;
        if source == ActionSource::ResolvedCardEffect {
            for step in action.steps() {
                let victim = next.piece_at(step.take)?;
                let attacker = next
                    .piece_at(step.from)?
                    .filter(|p| p.owner == PLAYER)
                    .map(|p| p.id);
                if !matches!(step.effect, crate::Effect::Move | crate::Effect::Shift)
                    && victim.is_some_and(|p| p.kind == pieces::TRIBAL_DICTATOR)
                    && !boss::hit_boss(
                        &mut self.boss,
                        &mut self.events,
                        next,
                        victim.unwrap(),
                        attacker,
                        1,
                    )
                {
                    continue;
                }
                apply_default_action(next, &DefaultAction::one(step.clone()))?;
                corpse_captured |= victim.is_some_and(|p| p.kind == pieces::CORPSE)
                    && !matches!(step.effect, crate::Effect::Move | crate::Effect::Shift)
                    && victim.is_some_and(|p| next.cells().iter().flatten().all(|q| q.id != p.id));
                if let Some(victim) = victim {
                    if !matches!(step.effect, crate::Effect::Move | crate::Effect::Shift)
                        && next.cells().iter().flatten().all(|p| p.id != victim.id)
                    {
                        boss::death_near_boss(
                            &mut self.boss,
                            &mut self.absorbed,
                            &mut self.events,
                            self.boss_id,
                            next,
                            victim,
                            step.take,
                        );
                    }
                }
            }
            if !self.sandbox {
                for id in std::iter::once(self.king_id)
                    .chain(std::iter::once(self.royal_id))
                    .chain(self.additional_royal_ids.iter().copied())
                {
                    if !next.cells().iter().flatten().any(|p| p.id == id)
                        && !(id == self.royal_id && self.boss.as_ref().is_some_and(|b| b.hp <= 0))
                    {
                        return Err(Error::ProtectedRoyal);
                    }
                }
            }
        } else {
            for step in action.steps() {
                // Resolve the actual captor and victim at this step, including multi-step moves.
                let captor = next.piece_at(step.from)?;
                let victim = next.piece_at(step.take)?;
                if !matches!(step.effect, crate::Effect::Move | crate::Effect::Shift) {
                    if let (Some(captor), Some(victim)) = (captor, victim) {
                        if victim.kind != pieces::CORPSE && victim.kind != pieces::TRIBAL_DICTATOR {
                            let damage = self
                                .combat_stats
                                .get(&captor.id)
                                .cloned()
                                .unwrap_or_default()
                                .attack
                                .max(0);
                            let stats = self.combat_stats.entry(victim.id).or_default();
                            stats.health = stats.health.saturating_sub(damage);
                            if stats.health > 0 {
                                break;
                            }
                        }
                    }
                }
                if captor.is_some_and(|p| p.owner == ENEMY && p.kind == pieces::EMPEROR)
                    && victim.is_some_and(|p| p.owner == PLAYER)
                    && !matches!(step.effect, crate::Effect::Move | crate::Effect::Shift)
                {
                    absorbed.push((captor.unwrap().id, victim.unwrap().kind));
                }
                let retaliates = !matches!(step.effect, crate::Effect::Move | crate::Effect::Shift)
                    && captor.is_some_and(|p| p.owner == ENEMY)
                    && victim
                        .is_some_and(|p| p.owner == PLAYER && self_defense_target == Some(p.id));
                if !matches!(step.effect, crate::Effect::Move | crate::Effect::Shift)
                    && victim.is_some_and(|p| p.kind == pieces::TRIBAL_DICTATOR)
                    && !boss::hit_boss(
                        &mut self.boss,
                        &mut self.events,
                        next,
                        victim.unwrap(),
                        captor.map(|p| p.id),
                        captor
                            .and_then(|p| self.combat_stats.get(&p.id))
                            .map_or(1, |s| s.attack.max(0)),
                    )
                {
                    break;
                }
                apply_default_action(next, &DefaultAction::one(step.clone()))?;
                corpse_captured |= victim.is_some_and(|p| p.kind == pieces::CORPSE)
                    && !matches!(step.effect, crate::Effect::Move | crate::Effect::Shift)
                    && victim.is_some_and(|p| next.cells().iter().flatten().all(|q| q.id != p.id));
                if let Some(victim) = victim {
                    if !matches!(step.effect, crate::Effect::Move | crate::Effect::Shift)
                        && next.cells().iter().flatten().all(|p| p.id != victim.id)
                    {
                        boss::death_near_boss(
                            &mut self.boss,
                            &mut self.absorbed,
                            &mut self.events,
                            self.boss_id,
                            next,
                            victim,
                            step.take,
                        );
                    }
                }
                player_captured |=
                    !matches!(step.effect, crate::Effect::Move | crate::Effect::Shift)
                        && captor.is_some_and(|p| p.owner == PLAYER)
                        && victim.is_some_and(|p| p.owner == ENEMY);
                enemy_captured |=
                    !matches!(step.effect, crate::Effect::Move | crate::Effect::Shift)
                        && captor.is_some_and(|p| p.owner == ENEMY)
                        && victim.is_some_and(|p| p.owner == PLAYER);
                if !matches!(step.effect, crate::Effect::Move | crate::Effect::Shift) {
                    if let (Some(captor), Some(victim)) = (captor, victim) {
                        if captor.owner < 2 && victim.owner < 2 && captor.owner != victim.owner {
                            captures.push((captor, victim));
                        }
                    }
                }
                if retaliates {
                    let captor = captor.unwrap();
                    if let Some(index) = next
                        .cells()
                        .iter()
                        .position(|p| p.is_some_and(|p| p.id == captor.id))
                    {
                        let width = next.width();
                        next.set_piece(
                            Position::new((index % width) as i32, (index / width) as i32),
                            None,
                        )
                        .expect("existing board square is always valid");
                    }
                    retaliation = Some(captor);
                    // A dead captor cannot perform any remaining steps of its move.
                    break;
                }
            }
        }
        if let Some(next) = staged {
            self.board = next;
        }
        self.forced_turn_end |= corpse_captured;
        for (id, kind) in absorbed {
            if self.board.cells().iter().flatten().any(|p| p.id == id) {
                self.absorbed.entry(id).or_default().insert(kind);
            }
        }
        if player_captured {
            self.mechanics.killed_this_turn = true;
        }
        if player_captured && self.initiative_turns_left > 0 {
            self.initiative_turns_left = 0;
            self.events
                .push("선공권: 아군의 포획이 성공해 보호가 끝났습니다.".into());
        }
        if enemy_captured && self.enemy_initiative_turns_left > 0 {
            self.enemy_initiative_turns_left = 0;
            self.events
                .push("선공권: 적의 포획이 성공해 보호가 끝났습니다.".into());
        }
        for (captor, victim) in captures {
            self.record_kill(captor, victim);
        }
        self.wards
            .retain(|id| self.board.cells().iter().flatten().any(|p| p.id == *id));
        self.limiter
            .retain(|id, _| self.board.cells().iter().flatten().any(|p| p.id == *id));
        self.sync_players();
        if let Some(captor) = retaliation {
            self.events.push(format!(
                "호신용품: 보호 대상을 포획한 적 {}도 사망했습니다.",
                piece_name(captor.kind)
            ));
        }
        if source == ActionSource::PieceMove || self.boss.is_some() {
            self.update_result(if retaliation.is_some() {
                ResultReason::SelfDefense
            } else {
                ResultReason::RoyalCaptured
            });
        }
        Ok(())
    }
    pub(in crate::battle) fn sync_players(&mut self) {
        if self.board.cells().iter().flatten().any(|p| {
            matches!(
                crate::content::pieces::definition(p.kind).map(|d| d.key),
                Some("smiling_king" | "toast_lord" | "ecstatic_lord" | "eternal_guest")
            )
        }) {
            for piece in self
                .board
                .cells()
                .iter()
                .flatten()
                .filter(|p| p.kind != pieces::CORPSE)
            {
                self.mechanics
                    .statuses
                    .entry(piece.id)
                    .or_default()
                    .ingredient = true;
            }
        }
        if self.boss.is_some()
            && !self
                .board
                .cells()
                .iter()
                .flatten()
                .any(|p| p.id == self.boss_id)
        {
            self.boss = None;
        }
        self.mechanics.pending_agility.retain(|id, _| {
            self.board
                .cells()
                .iter()
                .flatten()
                .any(|p| p.id == *id && p.kind != pieces::CORPSE)
        });
        let previous: Vec<_> = self.players.iter().map(|p| p.position).collect();
        for p in &mut self.players {
            p.position = None;
        }
        for (index, piece) in self.board.cells().iter().enumerate() {
            if let Some(piece) = piece {
                if piece.owner == PLAYER && piece.kind != pieces::CORPSE {
                    if let Some(p) = self.players.iter_mut().find(|p| p.id == piece.id) {
                        p.position = Some(Position::new(index as i32 % 8, index as i32 / 8));
                    }
                }
            }
        }
        for (piece, before) in self.players.iter_mut().zip(previous) {
            if piece.position != before && piece.position.is_some() {
                piece.moved = true;
                self.moved_ids.insert(piece.id);
                if before.is_some() {
                    self.mechanics.turn_moved.insert(piece.id);
                }
            }
        }
    }
    pub(in crate::battle) fn update_result(&mut self, reason: ResultReason) {
        if self.sandbox {
            return;
        }
        let king_alive = if self.mechanics.player_artillery {
            self.board
                .cells()
                .iter()
                .flatten()
                .any(|p| p.owner == PLAYER && p.kind == pieces::WAREHOUSE)
        } else {
            self.players
                .iter()
                .any(|p| p.id == self.king_id && p.position.is_some())
        };
        let royal_alive = self
            .board
            .cells()
            .iter()
            .flatten()
            .any(|p| p.owner == ENEMY && self.is_enemy_royal(p.id));
        let racing_loss = self.enemy_racing_goal.is_some_and(|rank| {
            self.board.cells().iter().enumerate().any(|(i, p)| {
                i as i32 / 8 == rank
                    && p.is_some_and(|p| p.owner == ENEMY && self.is_enemy_royal(p.id))
            })
        });
        self.result = if racing_loss {
            GameResult::Win {
                winner: ENEMY,
                reason: ResultReason::RacingGoal,
            }
        } else if !king_alive && self.enemy_racing_goal.is_none() {
            GameResult::Win {
                winner: ENEMY,
                reason,
            }
        } else if !royal_alive {
            GameResult::Win {
                winner: PLAYER,
                reason,
            }
        } else {
            GameResult::Ongoing
        };
        self.finalize_result();
    }
    pub(in crate::battle) fn check_king_goal(&mut self, actor: Piece, from: Position) {
        if self.sandbox
            || self.result != GameResult::Ongoing
            || (actor.owner == ENEMY && self.enemy_racing_goal.is_some())
        {
            return;
        }
        let Some((at, current)) = self.piece_position(actor.id) else {
            return;
        };
        if at == from || current.kind != actor.kind {
            return;
        }
        let racing = matches!(actor.kind, pieces::RACING_KING | pieces::JOCKEY);
        let reached_racing_goal = racing && at.rank == if actor.owner == PLAYER { 0 } else { 7 };
        let reached_hill = actor.kind == pieces::HILL_KING
            && (3..=4).contains(&at.file)
            && (3..=4).contains(&at.rank);
        let reason = if reached_hill {
            Some(ResultReason::HillGoal)
        } else if reached_racing_goal {
            Some(ResultReason::RacingGoal)
        } else {
            None
        };
        if let Some(reason) = reason {
            self.result = GameResult::Win {
                winner: actor.owner,
                reason,
            };
            self.events.push(format!(
                "{}: 목표 칸에 도달했습니다.",
                piece_name(actor.kind)
            ));
            self.finalize_result();
        }
    }
    pub(in crate::battle) fn finalize_result(&mut self) {
        self.phase = if self.result == GameResult::Ongoing {
            Phase::Active
        } else {
            Phase::Finished
        };
        if self.phase == Phase::Finished {
            self.finish_persistent_statuses();
            self.self_defense_target = None;
            self.initiative_turns_left = 0;
            self.enemy_initiative_turns_left = 0;
            self.limiter.clear();
        }
    }
}
