//! shooting rules. The call sites retain their original ordering.
use super::*;

impl Battle {
    pub fn volley_shot_count(&self) -> usize {
        self.volley_shots().len()
    }
    pub(in crate::battle) fn volley_shots(&self) -> Vec<(Position, Piece, Position, Piece, u8)> {
        let mut shots = Vec::new();
        for (from, shooter) in self.living(PLAYER) {
            if !pieces::is_infantry(shooter.kind) || !self.can_act(shooter) {
                continue;
            }
            for (to, target) in self.living(ENEMY) {
                if let Some(chance) = self.hit_chance(from, to) {
                    shots.push((from, shooter, to, target, chance));
                }
            }
        }
        shots
    }
    pub(in crate::battle) fn roll_shot(&mut self, chance: u8) -> bool {
        let roll = self.mechanics.rng.index(100);
        self.mechanics
            .search_shot_outcome
            .take()
            .unwrap_or(roll < usize::from(chance))
    }
    pub(in crate::battle) fn blast_cells(&self, at: Position, shape: BlastShape) -> Vec<Position> {
        let mut offsets = vec![(0, 0)];
        offsets.extend([(1, 0), (-1, 0), (0, 1), (0, -1)]);
        if matches!(shape, BlastShape::King | BlastShape::Nuclear) {
            offsets.extend([(1, 1), (1, -1), (-1, 1), (-1, -1)]);
        }
        if matches!(shape, BlastShape::Nuclear) {
            offsets.extend([
                (1, 2),
                (2, 1),
                (2, -1),
                (1, -2),
                (-1, -2),
                (-2, -1),
                (-2, 1),
                (-1, 2),
            ]);
        }
        offsets
            .into_iter()
            .map(|(dx, dy)| Position::new(at.file + dx, at.rank + dy))
            .filter(|&pos| self.board.contains(pos))
            .collect()
    }
    pub fn blast_preview(&self, kind: PieceType, at: Position) -> Vec<Position> {
        let shape = match kind {
            pieces::C4_DRONE => BlastShape::King,
            pieces::TACTICAL_NUKE_DRONE => BlastShape::Nuclear,
            _ => BlastShape::Cross,
        };
        self.blast_cells(at, shape)
    }
    pub fn shot_effect_preview(
        &self,
        kind: PieceType,
        from: Position,
        to: Position,
    ) -> (Vec<Position>, Vec<Vec<Position>>) {
        let blast = if matches!(
            kind,
            pieces::HE_TANK | pieces::DOUBLE_BARREL | pieces::GRENADE_LAUNCHER_DRONE
        ) {
            self.blast_cells(to, BlastShape::Cross)
        } else {
            vec![]
        };
        let mut paths = vec![];
        if matches!(kind, pieces::AP_TANK | pieces::FRAGMENT_TANK) {
            let direction = (
                (to.file - from.file).signum(),
                (to.rank - from.rank).signum(),
            );
            let directions = if kind == pieces::FRAGMENT_TANK {
                vec![direction, rotate45(direction, -1), rotate45(direction, 1)]
            } else {
                vec![direction]
            };
            for (dx, dy) in directions {
                let mut path = vec![];
                let mut pos = to;
                loop {
                    pos = Position::new(pos.file + dx, pos.rank + dy);
                    if !self.board.contains(pos) {
                        break;
                    }
                    path.push(pos);
                }
                paths.push(path);
            }
        }
        (blast, paths)
    }
    pub fn fallout_remaining(&self, at: Position) -> u32 {
        self.board.index(at).map_or(0, |i| {
            self.mechanics.fallout[i].saturating_sub(self.mechanics.rounds)
        })
    }
    pub fn fallout_cells(&self) -> Vec<(Position, u32)> {
        (0..64)
            .filter_map(|i| {
                let at = Position::new(i % 8, i / 8);
                let remaining = self.fallout_remaining(at);
                (remaining > 0).then_some((at, remaining))
            })
            .collect()
    }
    pub(in crate::battle) fn explode(
        &mut self,
        at: Position,
        shape: BlastShape,
        fallout: bool,
        attacker: Option<PieceId>,
    ) -> Result<(), Error> {
        let cells = self.blast_cells(at, shape);
        for &pos in &cells {
            if let Some(piece) = self.board.piece_at(pos)? {
                if piece.owner == ENEMY && self.turn == PLAYER {
                    self.mechanics.killed_this_turn = true;
                }
                self.kill_at_by(pos, false, attacker)?;
            }
        }
        if fallout {
            let expires = self.mechanics.rounds + FALLOUT_ROUNDS + u32::from(self.turn == ENEMY);
            for pos in &cells {
                let index = self.board.index(*pos).expect("valid blast cell");
                self.mechanics.fallout[index] = self.mechanics.fallout[index].max(expires);
            }
        }
        self.events.push(format!(
            "{} 폭발: {}칸 처리{}.",
            square(at),
            cells.len(),
            if fallout {
                " · 방사성 낙진 2라운드"
            } else {
                ""
            }
        ));
        Ok(())
    }
    /// A reusable penetration path; empty cells do not consume probability.
    pub(in crate::battle) fn penetrate_paths(
        &mut self,
        start: Position,
        directions: &[(i32, i32)],
        initial: u8,
        decay: u8,
        target_owner: Option<PlayerId>,
        shooter: Piece,
    ) -> Result<(), Error> {
        for &direction in directions {
            self.penetrate(start, direction, initial, decay, target_owner, shooter)?;
        }
        Ok(())
    }
    pub(in crate::battle) fn penetrate(
        &mut self,
        start: Position,
        direction: (i32, i32),
        initial: u8,
        decay: u8,
        target_owner: Option<PlayerId>,
        shooter: Piece,
    ) -> Result<(), Error> {
        let mut at = start;
        let mut successes = 0;
        loop {
            at = Position::new(at.file + direction.0, at.rank + direction.1);
            let chance = penetration_chance(initial, decay, successes);
            if !self.board.contains(at) || chance == 0 {
                break;
            }
            let Some(victim) = self.board.piece_at(at)? else {
                continue;
            };
            if target_owner.is_some_and(|owner| owner != victim.owner) {
                continue;
            }
            let roll = self.mechanics.rng.index(100);
            let hit = self
                .mechanics
                .search_shot_outcome
                .take()
                .unwrap_or(roll < usize::from(chance));
            self.events.push(format!(
                "관통경로 {}: {}% {}.",
                square(at),
                chance,
                if hit { "명중" } else { "빗나감" }
            ));
            if !hit {
                break;
            }
            if self.kill_at_by(at, false, Some(shooter.id))? {
                self.record_kill(shooter, victim);
            }
            successes += 1;
        }
        Ok(())
    }
    pub fn use_drone_ability(&mut self, from: Position) -> Result<(), Error> {
        let actor = self.board.piece_at(from)?.ok_or(Error::IllegalAction)?;
        if !matches!(
            actor.kind,
            pieces::DRONE
                | pieces::C4_DRONE
                | pieces::MACHINE_GUN_DRONE
                | pieces::TACTICAL_NUKE_DRONE
                | pieces::GRENADE_LAUNCHER_DRONE
        ) {
            return Err(Error::IllegalAction);
        }
        let action = self
            .legal_actions(from)?
            .into_iter()
            .find(|action| action.first.effect == Effect::Catch && action.first.take == from)
            .ok_or(Error::IllegalAction)?;
        self.apply_piece_action(&action)
    }
    pub fn hit_chance(&self, from: Position, to: Position) -> Option<u8> {
        let p = self.board.piece_at(from).ok().flatten()?;
        let base_chance = if p.kind == pieces::DOUBLE_BARREL {
            let mut board = self.board.clone();
            board
                .set_piece(
                    from,
                    Some(Piece {
                        kind: pieces::TEMPEST_ROOK,
                        ..p
                    }),
                )
                .ok()?;
            let program = self.programs.get(&pieces::TEMPEST_ROOK)?;
            let in_range = program
                .execute(&board, from)
                .ok()?
                .iter()
                .any(|action| action.steps().last().is_some_and(|step| step.move_to == to));
            in_range.then_some(90)?
        } else {
            shot_preview(p.kind, p.owner, from, to).or_else(|| {
                if p.owner != PLAYER || !self.mechanics.entrenched_ready.contains(&p.id) {
                    return None;
                }
                let dx = (to.file - from.file).signum();
                let dy = (to.rank - from.rank).signum();
                let closer = Position::new(to.file - dx, to.rank - dy);
                let (_, falloff, range) = shot_profile(key(p.kind))?;
                (distance(from, to) == range + 1)
                    .then(|| shot_preview(p.kind, p.owner, from, closer))
                    .flatten()
                    .map(|chance| chance.saturating_sub(falloff))
            })?
        };
        let dx = to.file - from.file;
        let dy = to.rank - from.rank;
        let d = distance(from, to);
        let k = key(p.kind);
        if !matches!(k, "mortar" | "double_barrel") {
            for n in 1..d {
                let at = Position::new(from.file + dx.signum() * n, from.rank + dy.signum() * n);
                if self.board.piece_at(at).ok().flatten().is_some() {
                    return None;
                }
            }
        }
        let bonus = if p.owner == PLAYER {
            [(Relic::FireControl, 10), (Relic::ReconScope, 15)]
                .iter()
                .filter(|(r, _)| self.mechanics.relics.contains(r))
                .map(|(_, n)| n)
                .sum::<i32>()
        } else {
            0
        };
        let bonus = bonus
            + i32::from(*self.mechanics.aimed.get(&p.id).unwrap_or(&0))
            + if self.mechanics.relics.contains(&Relic::BallisticComputer)
                && self.mechanics.shot_counts.get(&p.id).copied().unwrap_or(0) >= 1
            {
                10
            } else {
                0
            }
            + if self.mechanics.relics.contains(&Relic::GunMount)
                && !self.mechanics.turn_moved.contains(&p.id)
            {
                15
            } else {
                0
            }
            + if self.mechanics.entrenched_ready.contains(&p.id) {
                15
            } else {
                0
            }
            + if self.mechanics.fire_data_sharing
                && self.mechanics.shared_target
                    == self.board.piece_at(to).ok().flatten().map(|p| p.id)
            {
                15
            } else {
                0
            };
        Some((i32::from(base_chance) + bonus).clamp(0, 100) as u8)
    }
}
