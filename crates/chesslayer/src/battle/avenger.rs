use super::*;
use crate::content::cards::CardKind;
use crate::{ActionStep, Effect};

impl Battle {
    pub fn avenger_targets(&self, kind: CardKind) -> Vec<PieceId> {
        use CardKind::*;
        if matches!(kind, MakeBaby | BabyRank) {
            if self.family_formations(PLAYER).is_empty() {
                return vec![];
            }
            return (8..64)
                .filter(|&i| {
                    self.board.cells()[i].is_none()
                        && (kind != BabyRank
                            || self.board.cells()[i / 8 * 8..i / 8 * 8 + 8]
                                .iter()
                                .all(Option::is_none))
                })
                .map(|i| 10_000 + i as u32)
                .collect();
        }
        if matches!(kind, AlekhineGun | GunBarrage) {
            return self.gun_formations(PLAYER).iter().map(|f| f[0]).collect();
        }
        self.board
            .cells()
            .iter()
            .flatten()
            .filter(|p| kind == AbsoluteAuthority || p.owner < 2)
            .filter(|&&p| match kind {
                AbsoluteAuthority => true,
                AuthoritarianOrder => {
                    p.owner == PLAYER
                        && self.piece_class(p.id) != PieceClass::Major
                        && !self.is_royal_piece(p.id)
                }
                PawnOrder => {
                    p.owner == PLAYER
                        && (self.piece_class(p.id) == PieceClass::Pawn
                            || self.matches_kind(p, PAWN))
                }
                MinorOrder => p.owner == PLAYER && self.piece_class(p.id) == PieceClass::Minor,
                MajorOrder => p.owner == PLAYER && self.piece_class(p.id) == PieceClass::Major,
                MindControl => p.owner == ENEMY,
                Interference => p.owner == ENEMY && !self.is_enemy_royal(p.id),
                RoyalRevolution => p.owner == PLAYER && !self.is_royal_piece(p.id),
                BloodOffering | WealthOffering => p.owner == PLAYER && !self.is_royal_piece(p.id),
                Overestimate | Bounty | TacticalMove | Rename => p.owner == PLAYER,
                Castling => self.castle_positions(p.id).is_some(),
                EnPassant => self.en_passant_action(p.id).is_some(),
                ComeHere => self.king_adjacent_empty().is_some_and(|v| !v.is_empty()),
                _ => true,
            })
            .map(|p| p.id)
            .collect()
    }
    fn castle_positions(&self, major: PieceId) -> Option<(Position, Position, Position, Position)> {
        let (from, p) = self.piece_position(major)?;
        let (king, k) = self.piece_position(self.king_id)?;
        if p.owner != PLAYER
            || k.owner != PLAYER
            || major == self.king_id
            || self.piece_class(major) != PieceClass::Major
        {
            return None;
        }
        let dx = from.file - king.file;
        let dy = from.rank - king.rank;
        let distance = dx.abs().max(dy.abs());
        if (dx != 0 && dy != 0) || distance < 3 {
            return None;
        }
        if (1..distance).any(|n| {
            self.board
                .piece_at(Position::new(
                    king.file + dx.signum() * n,
                    king.rank + dy.signum() * n,
                ))
                .ok()
                .flatten()
                .is_some()
        }) {
            return None;
        }
        Some((
            king,
            from,
            Position::new(king.file + dx.signum() * 2, king.rank + dy.signum() * 2),
            Position::new(king.file + dx.signum(), king.rank + dy.signum()),
        ))
    }
    fn en_passant_action(&self, id: PieceId) -> Option<DefaultAction> {
        let (victim, land) = self.en_passant?;
        let (from, p) = self.piece_position(id)?;
        let (at, q) = self.piece_position(victim)?;
        if p.owner != PLAYER
            || q.owner != ENEMY
            || !self.matches_kind(p, PAWN)
            || self.stats(id).disarmed
            || from.rank != at.rank
            || (from.file - at.file).abs() != 1
            || land.rank != from.rank - 1
            || self.board.piece_at(land).ok().flatten().is_some()
        {
            return None;
        }
        Some(DefaultAction::one(ActionStep::new(
            from,
            at,
            land,
            Effect::TakeJump,
        )))
    }
    fn king_adjacent_empty(&self) -> Option<Vec<Position>> {
        let (at, _) = self
            .board
            .cells()
            .iter()
            .flatten()
            .filter(|p| p.owner == PLAYER && self.matches_kind(**p, KING))
            .find_map(|p| self.piece_position(p.id))?;
        Some(
            (-1..=1)
                .flat_map(|dy| (-1..=1).map(move |dx| Position::new(at.file + dx, at.rank + dy)))
                .filter(|&p| {
                    self.board.contains(p) && self.board.piece_at(p).ok().flatten().is_none()
                })
                .collect(),
        )
    }
    pub fn spawn_pawn(&mut self, at: Position) -> Result<PieceId, Error> {
        if !self.board.contains(at) || at.rank == 0 || self.board.piece_at(at)?.is_some() {
            return Err(Error::IllegalAction);
        }
        // A created pawn is a permanent ally. Give it a distinct home before
        // changing the board so run-boundary roster rebuilding keeps it.
        let origin = (0..8)
            .rev()
            .flat_map(|rank| (0..8).map(move |file| Position::new(file, rank)))
            .find(|pos| !self.players.iter().any(|p| p.origin == *pos))
            .ok_or(Error::InvalidSetup)?;
        let id = (1_000_000..=1_000_064)
            .find(|id| {
                !self.board.cells().iter().flatten().any(|p| p.id == *id)
                    && !self.players.iter().any(|p| p.id == *id)
            })
            .ok_or(Error::InvalidSetup)?;
        self.board.set_piece(
            at,
            Some(Piece {
                id,
                kind: PAWN,
                owner: PLAYER,
            }),
        )?;
        self.players.push(PlayerPiece {
            id,
            kind: PAWN,
            origin,
            position: Some(at),
            moved: false,
            upgrades: vec![],
            persistent: BTreeMap::new(),
        });
        self.mechanics.statuses.insert(id, PieceStatus::default());
        Ok(id)
    }
    /// Board effects; resources and payment are staged by the run's card transaction.
    pub fn avenger_effect(
        &mut self,
        kind: CardKind,
        target: Option<PieceId>,
        name: Option<&str>,
    ) -> Result<(), Error> {
        use CardKind::*;
        let id = target.unwrap_or(0);
        if kind.definition().targeted && !self.avenger_targets(kind).contains(&id) {
            return Err(Error::IllegalAction);
        }
        if id == self.boss_id && self.boss.is_some() {
            self.combat_stats.insert(id, self.stats(id));
        }
        match kind {
            PawnOrder | MinorOrder | MajorOrder | MindControl | TacticalMove
            | AbsoluteAuthority | AuthoritarianOrder => {
                self.grant_move_rights(id, 1)?;
                if kind == TacticalMove {
                    self.stats_mut(id).disarmed = true;
                }
            }
            Castling => {
                let (a, b, c, d) = self.castle_positions(id).ok_or(Error::IllegalAction)?;
                let k = self.board.piece_at(a)?;
                let p = self.board.piece_at(b)?;
                self.board.set_piece(a, None)?;
                self.board.set_piece(b, None)?;
                self.board.set_piece(c, k)?;
                self.board.set_piece(d, p)?;
                self.moved_ids.insert(id);
                self.moved_ids.insert(self.king_id);
                self.sync_players();
                self.update_result(ResultReason::RoyalCaptured);
            }
            EnPassant => {
                let action = self.en_passant_action(id).ok_or(Error::IllegalAction)?;
                if let Some(victim) = self.warded_victim(&action, PLAYER)? {
                    self.wards.remove(&victim.id);
                } else {
                    self.apply_effect(&action, ActionSource::PieceMove)?;
                }
                self.moved_ids.insert(id);
                self.en_passant = None;
            }
            Exile | Revolution => {
                let ids: Vec<_> = self
                    .board
                    .cells()
                    .iter()
                    .flatten()
                    .filter(|p| {
                        if kind == Exile {
                            matches!(self.piece_class(p.id), PieceClass::Pawn | PieceClass::Minor)
                        } else {
                            self.piece_class(p.id) == PieceClass::Major
                        }
                    })
                    .map(|p| p.id)
                    .collect();
                self.force_remove(&ids)?;
            }
            RoyalRevolution => self.change_player_royal(id)?,
            Interference => self.change_enemy_royal(id)?,
            Aerobics | FullArmor => {
                let s = self.stats_mut(id);
                let n = if kind == FullArmor { 3 } else { 1 };
                s.health = s.health.saturating_add(n);
                s.max_health = s.max_health.saturating_add(n);
                if kind == Aerobics {
                    s.temporary_health += n;
                }
                if id == self.boss_id {
                    if let Some(boss) = &mut self.boss {
                        boss.hp = boss.hp.saturating_add(n);
                        boss.max_health = boss.max_health.saturating_add(n);
                    }
                }
            }
            Anaerobics | Steroids => {
                let s = self.stats_mut(id);
                let requested = if kind == Steroids { s.attack } else { 1 };
                let n = requested.min(i32::MAX - s.attack);
                s.attack += n;
                s.temporary_attack = s.temporary_attack.saturating_add(n);
            }
            SwapStats => {
                let s = self.stats_mut(id);
                std::mem::swap(&mut s.health, &mut s.attack);
                s.max_health = s.max_health.max(s.health);
                let (health, max_health) = (s.health, s.max_health);
                self.set_health(id, health, max_health);
            }
            Overestimate => {
                let s = self.stats_mut(id);
                s.score_multiplier = s.score_multiplier.saturating_mul(2);
            }
            Rename => {
                let name = name
                    .filter(|n| pieces::ALL.iter().any(|d| d.name == *n))
                    .ok_or(Error::IllegalAction)?;
                self.stats_mut(id).name = Some(name.into());
            }
            AvengerLimiter => {
                self.grant_death_threshold(id)?;
                self.grant_move_rights(id, 3)?;
                self.stats_mut(id).dies_next_turn = true;
            }
            Scp500 => {
                self.mechanics.statuses.insert(id, PieceStatus::default());
                self.mechanics.pending_agility.remove(&id);
                self.mechanics.suppressed.remove(&id);
                self.limiter.remove(&id);
                let s = self.stats_mut(id);
                s.disarmed = false;
                s.dies_next_turn = false;
                if let Some(p) = self.players.iter_mut().find(|p| p.id == id) {
                    for key in [
                        "hunger",
                        "madness",
                        "guilt",
                        "incapacitated",
                        "death_threshold",
                        "hunger_worsened",
                    ] {
                        p.persistent.remove(key);
                    }
                }
            }
            AllCharge => {
                let ids: Vec<_> = self
                    .board
                    .cells()
                    .iter()
                    .flatten()
                    .filter(|p| p.owner == PLAYER)
                    .map(|p| p.id)
                    .collect();
                for id in ids {
                    self.grant_move_rights(id, 1)?;
                }
            }
            DisarmAll => {
                let ids: Vec<_> = self
                    .board
                    .cells()
                    .iter()
                    .flatten()
                    .filter(|p| p.owner == ENEMY)
                    .map(|p| p.id)
                    .collect();
                for id in ids {
                    self.stats_mut(id).disarmed = true;
                }
            }
            MakeBaby | BabyRank => {
                let cell = id - 10_000;
                let at = Position::new(cell as i32 % 8, cell as i32 / 8);
                if kind == MakeBaby {
                    self.spawn_pawn(at)?;
                } else {
                    for file in 0..8 {
                        self.spawn_pawn(Position::new(file, at.rank))?;
                    }
                }
            }
            AlekhineGun | GunBarrage => {
                let formation = self
                    .gun_formations(PLAYER)
                    .into_iter()
                    .find(|f| f[0] == id)
                    .ok_or(Error::IllegalAction)?;
                let (front, _) = self.piece_position(id).ok_or(Error::IllegalAction)?;
                let ids: Vec<_> = self
                    .board
                    .cells()
                    .iter()
                    .enumerate()
                    .filter_map(|(i, p)| {
                        p.filter(|p| {
                            let rank = i as i32 / 8;
                            (rank < front.rank
                                && if kind == AlekhineGun {
                                    (i as i32 % 8 - front.file).abs() <= 1
                                } else {
                                    rank >= front.rank - 4
                                })
                                || (kind == GunBarrage && formation.contains(&p.id))
                        })
                        .map(|p| p.id)
                    })
                    .collect();
                self.force_remove(&ids)?;
            }
            ComeHere => {
                let empty = self.king_adjacent_empty().ok_or(Error::IllegalAction)?;
                let to = empty[self.mechanics.rng.index(empty.len())];
                let (from, p) = self.piece_position(id).ok_or(Error::IllegalAction)?;
                self.board.set_piece(from, None)?;
                self.board.set_piece(to, Some(p))?;
                self.sync_players();
                self.update_result(ResultReason::RoyalCaptured);
            }
            // Resource effects and permanent upgrades are resolved in Game.
            BadHand | PotOfGreed | Optimize | ClaudeOptimize | BloodOffering | WealthOffering
            | Bounty | Divinity => {}
        }
        Ok(())
    }
}
