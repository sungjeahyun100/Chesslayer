//! Shared combat attributes and live position predicates. Presets never set predicates.
use super::*;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PieceClass {
    Pawn,
    Minor,
    Major,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct CombatStats {
    pub health: i32,
    pub max_health: i32,
    pub attack: i32,
    pub move_rights: u32,
    pub disarmed: bool,
    pub name: Option<String>,
    pub score_bonus: i32,
    pub score_multiplier: i32,
    pub(crate) temporary_health: i32,
    pub(crate) temporary_attack: i32,
    pub(crate) dies_next_turn: bool,
}
impl Default for CombatStats {
    fn default() -> Self {
        Self {
            health: 1,
            max_health: 1,
            attack: 1,
            move_rights: 0,
            disarmed: false,
            name: None,
            score_bonus: 0,
            score_multiplier: 1,
            temporary_health: 0,
            temporary_attack: 0,
            dies_next_turn: false,
        }
    }
}

pub fn classification(kind: PieceType) -> (PieceClass, i32) {
    if let Some(profile) = pieces::definition(kind).and_then(|d| d.combat_profile) {
        return profile;
    }
    let key = pieces::definition(kind).map(|d| d.key).unwrap_or("");
    if key.contains("pawn") {
        (PieceClass::Pawn, 1)
    } else if key.contains("knight") || key.contains("bishop") {
        (PieceClass::Minor, 3)
    } else if key.contains("rook") {
        (PieceClass::Major, 5)
    } else if key.contains("queen") {
        (PieceClass::Major, 9)
    } else if pieces::is_royal(kind) {
        (PieceClass::Other, 0)
    } else {
        let score = pieces::definition(kind)
            .map(|d| (d.value / 10).max(1))
            .unwrap_or(1);
        (
            if score <= 1 {
                PieceClass::Pawn
            } else if score <= 3 {
                PieceClass::Minor
            } else {
                PieceClass::Major
            },
            score,
        )
    }
}

impl Battle {
    pub fn stats(&self, id: PieceId) -> CombatStats {
        let mut stats = self.combat_stats.get(&id).cloned().unwrap_or_default();
        if id == self.boss_id {
            if let Some(boss) = &self.boss {
                stats.health = boss.hp;
                stats.max_health = stats.max_health.max(boss.max_health).max(boss.hp);
            }
        }
        stats
    }
    pub fn set_health(&mut self, id: PieceId, health: i32, max_health: i32) {
        let s = self.stats_mut(id);
        s.health = health;
        s.max_health = max_health.max(health);
        if id == self.boss_id {
            if let Some(boss) = &mut self.boss {
                boss.hp = health;
                boss.max_health = max_health.max(health);
            }
        }
    }
    pub fn stats_mut(&mut self, id: PieceId) -> &mut CombatStats {
        self.combat_stats.entry(id).or_default()
    }
    pub fn card_movement(&self) -> bool {
        self.card_movement
    }
    pub fn enable_card_movement(&mut self) {
        self.card_movement = true;
    }
    pub fn player_controls(&self, piece: Piece) -> bool {
        piece.owner == PLAYER || (self.card_movement && self.stats(piece.id).move_rights > 0)
    }
    pub fn piece_class(&self, id: PieceId) -> PieceClass {
        self.piece_position(id)
            .map(|(_, p)| classification(p.kind).0)
            .unwrap_or(PieceClass::Other)
    }
    pub fn piece_score(&self, id: PieceId) -> i32 {
        let s = self.stats(id);
        self.piece_position(id)
            .map(|(_, p)| {
                classification(p.kind)
                    .1
                    .saturating_add(s.score_bonus)
                    .saturating_mul(s.score_multiplier)
            })
            .unwrap_or(0)
    }
    pub fn matches_kind(&self, piece: Piece, kind: PieceType) -> bool {
        piece.kind == kind
            || (piece.owner == PLAYER
                && self
                    .players
                    .iter()
                    .any(|owned| owned.id == piece.id && pieces::owned_matches_kind(owned, kind)))
            || self.stats(piece.id).name.as_deref() == Some(piece_name(kind))
    }
    pub fn grant_move_rights(&mut self, id: PieceId, count: u32) -> Result<(), Error> {
        if self.phase != Phase::Active || self.piece_position(id).is_none() {
            return Err(Error::IllegalAction);
        }
        let stats = self.stats_mut(id);
        stats.move_rights = stats.move_rights.saturating_add(count);
        Ok(())
    }
    /// Exact R-K-Q / P-P-P pattern, relative to each owner's forward direction.
    pub fn family_formations(&self, owner: PlayerId) -> Vec<[PieceId; 6]> {
        let forward = if owner == PLAYER { -1 } else { 1 };
        let mut found = vec![];
        for rank in 0..8 {
            for file in 0..6 {
                let cells = [
                    (file, rank, ROOK),
                    (file + 1, rank, KING),
                    (file + 2, rank, QUEEN),
                    (file, rank + forward, PAWN),
                    (file + 1, rank + forward, PAWN),
                    (file + 2, rank + forward, PAWN),
                ];
                let ids: Option<Vec<_>> = cells
                    .iter()
                    .map(|&(x, y, kind)| {
                        self.board
                            .piece_at(Position::new(x, y))
                            .ok()
                            .flatten()
                            .filter(|p| p.owner == owner && self.matches_kind(*p, kind))
                            .map(|p| p.id)
                    })
                    .collect();
                if let Some(ids) = ids {
                    found.push(ids.try_into().unwrap());
                }
            }
        }
        found
    }
    /// R-R-Q, front to back; intervening squares need not be adjacent or empty.
    pub fn gun_formations(&self, owner: PlayerId) -> Vec<[PieceId; 3]> {
        let pieces: Vec<_> = self
            .board
            .cells()
            .iter()
            .enumerate()
            .filter_map(|(i, p)| {
                p.filter(|p| p.owner == owner)
                    .map(|p| (Position::new(i as i32 % 8, i as i32 / 8), p))
            })
            .collect();
        let ahead = |a: i32, b: i32| if owner == PLAYER { a < b } else { a > b };
        let mut found = vec![];
        for &(a, r1) in &pieces {
            if !self.matches_kind(r1, ROOK) {
                continue;
            }
            for &(b, r2) in &pieces {
                if a.file != b.file || !ahead(a.rank, b.rank) || !self.matches_kind(r2, ROOK) {
                    continue;
                }
                for &(c, q) in &pieces {
                    if b.file == c.file && ahead(b.rank, c.rank) && self.matches_kind(q, QUEEN) {
                        found.push([r1.id, r2.id, q.id]);
                    }
                }
            }
        }
        found
    }
    /// Forced removals bypass health, wards and direct-capture reactions; outcome once at the end.
    pub fn force_remove(&mut self, ids: &[PieceId]) -> Result<usize, Error> {
        let mut count = 0;
        for &id in ids {
            if let Some((at, piece)) = self.piece_position(id) {
                self.board.set_piece(at, None)?;
                self.boss_on_death(piece, at);
                if id == self.boss_id {
                    self.boss = None;
                }
                self.wards.remove(&id);
                count += 1;
            }
        }
        self.sync_players();
        self.update_result(ResultReason::RoyalCaptured);
        Ok(count)
    }
    pub fn change_player_royal(&mut self, id: PieceId) -> Result<(), Error> {
        let (_, p) = self
            .piece_position(id)
            .filter(|(_, p)| p.owner == PLAYER)
            .ok_or(Error::IllegalAction)?;
        if p.id == self.king_id {
            return Err(Error::IllegalAction);
        }
        let old = self.king_id;
        if !self.players.iter().any(|p| p.id == id) {
            let origin = self
                .players
                .iter()
                .find(|p| p.id == old)
                .ok_or(Error::IllegalAction)?
                .origin;
            self.players.push(PlayerPiece {
                id,
                kind: p.kind,
                origin,
                position: self.piece_position(id).map(|(at, _)| at),
                moved: true,
                upgrades: vec![],
                persistent: BTreeMap::new(),
            });
        }
        self.king_id = id;
        for p in &mut self.players {
            p.persistent.insert("royal".into(), i32::from(p.id == id));
        }
        self.force_remove(&[old])?;
        self.players.retain(|p| p.id != old);
        Ok(())
    }
    pub fn change_enemy_royal(&mut self, id: PieceId) -> Result<(), Error> {
        self.piece_position(id)
            .filter(|(_, p)| p.owner == ENEMY && !self.is_enemy_royal(id))
            .ok_or(Error::IllegalAction)?;
        self.royal_id = id;
        self.additional_royal_ids.clear();
        self.update_result(ResultReason::RoyalCaptured);
        Ok(())
    }
    pub(super) fn finish_stats_turn(&mut self) -> Result<(), Error> {
        let mut dead = vec![];
        for (&id, s) in &mut self.combat_stats {
            if self.turn == PLAYER {
                s.move_rights = 0;
                s.name = None;
                s.max_health -= s.temporary_health;
                if id == self.boss_id {
                    if let Some(boss) = &mut self.boss {
                        s.max_health = s.max_health.max(boss.max_health - s.temporary_health);
                        if s.temporary_health > 0 {
                            boss.hp = boss.hp.min(s.max_health);
                        }
                        boss.max_health = s.max_health;
                        s.health = boss.hp;
                        s.max_health = s.max_health.max(boss.hp);
                    }
                }
                s.health = s.health.min(s.max_health);
                s.attack -= s.temporary_attack;
                s.temporary_health = 0;
                s.temporary_attack = 0;
                if s.health <= 0
                    && self
                        .board
                        .cells()
                        .iter()
                        .flatten()
                        .any(|p| p.id == id && p.kind != pieces::CORPSE)
                {
                    dead.push(id);
                }
            }
            let owner = self
                .board
                .cells()
                .iter()
                .flatten()
                .find(|p| p.id == id)
                .map(|p| p.owner);
            if owner == Some(self.turn) {
                s.disarmed = false;
            }
        }
        self.force_remove(&dead)?;
        Ok(())
    }
    pub(super) fn start_stats_turn(&mut self) -> Result<(), Error> {
        if self.turn == PLAYER {
            let dead: Vec<_> = self
                .combat_stats
                .iter()
                .filter(|(_, s)| s.dies_next_turn)
                .map(|(&id, _)| id)
                .collect();
            self.force_remove(&dead)?;
        }
        Ok(())
    }
}
