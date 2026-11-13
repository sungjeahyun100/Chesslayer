use super::*;
use crate::{ActionStep, Effect};

#[derive(Clone, Debug)]
pub struct BossState {
    pub hp: i32,
    pub ammo: u8,
    pub hook: Option<PieceId>,
    pub movement_used: bool,
    pub special_used: bool,
}

pub(super) fn hit_boss(
    boss: &mut Option<BossState>,
    events: &mut Vec<String>,
    board: &Board,
    victim: Piece,
    attacker: Option<PieceId>,
) -> bool {
    let Some(state) = boss.as_mut() else {
        return true;
    };
    state.hp -= 1;
    events.push(format!("부족의 독재자 HP -1 · 현재 {}.", state.hp.max(0)));
    if state.hp <= 0 {
        state.hook = None;
        return true;
    }
    if let Some(id) = attacker.filter(|id| *id != victim.id) {
        if board.cells().iter().flatten().any(|p| p.id == id) {
            state.hook = Some(id);
            events.push(format!(
                "갈고리 반격: 기물 #{id}에 갈고리가 박혔습니다. 다음 적 턴에 끌어옵니다."
            ));
        }
    }
    false
}

pub(super) fn death_near_boss(
    boss: &mut Option<BossState>,
    absorbed: &mut HashMap<PieceId, HashSet<PieceType>>,
    events: &mut Vec<String>,
    boss_id: PieceId,
    board: &Board,
    victim: Piece,
    at: Position,
) {
    if victim.id == boss_id {
        return;
    }
    let Some(state) = boss.as_mut() else { return };
    if state.hp <= 0 {
        return;
    }
    let Some((index, _)) = board
        .cells()
        .iter()
        .enumerate()
        .find(|(_, p)| p.is_some_and(|p| p.id == boss_id))
    else {
        return;
    };
    let boss_at = Position::new(
        (index % board.width()) as i32,
        (index / board.width()) as i32,
    );
    if (boss_at.file - at.file)
        .abs()
        .max((boss_at.rank - at.rank).abs())
        != 1
    {
        return;
    }
    state.hp += 1;
    events.push(format!(
        "찢고 죽여라: {}의 죽음으로 HP +1 · 현재 {}.",
        piece_name(victim.kind),
        state.hp
    ));
    if victim.kind != pieces::CORPSE && absorbed.entry(boss_id).or_default().insert(victim.kind) {
        events.push(format!(
            "제왕학: {}의 행마를 습득했습니다.",
            piece_name(victim.kind)
        ));
    }
}

impl Battle {
    pub fn boss(&self) -> Option<&BossState> {
        self.boss.as_ref()
    }

    pub fn boss_action_candidates(&self) -> Vec<DefaultAction> {
        self.piece_position(self.royal_id)
            .filter(|(_, p)| p.kind == pieces::TRIBAL_DICTATOR)
            .and_then(|(at, _)| self.actions_from(at, false).ok())
            .unwrap_or_default()
    }

    /// Moving captures and learned movement share one slot; stationary shotgun,
    /// reload, and learned abilities share the other. Classify the intended move,
    /// so a ward or failed attack still consumes the correct slot.
    pub(crate) fn boss_action_is_movement(action: &DefaultAction) -> bool {
        action.steps().any(|step| step.move_to != step.from)
    }

    pub(super) fn is_boss_actor(&self, piece: Piece) -> bool {
        self.boss.is_some()
            && piece.owner == ENEMY
            && piece.id == self.royal_id
            && piece.kind == pieces::TRIBAL_DICTATOR
    }

    pub(super) fn boss_action_available(&self, action: &DefaultAction) -> bool {
        self.boss.as_ref().is_some_and(|boss| {
            if Self::boss_action_is_movement(action) {
                !boss.movement_used
            } else {
                !boss.special_used
            }
        })
    }

    pub(super) fn spend_boss_action(&mut self, action: &DefaultAction) {
        if let Some(boss) = self.boss.as_mut() {
            if Self::boss_action_is_movement(action) {
                boss.movement_used = true;
            } else {
                boss.special_used = true;
            }
        }
    }

    pub(super) fn boss_actions_spent(&self) -> bool {
        self.boss
            .as_ref()
            .is_some_and(|boss| boss.movement_used && boss.special_used)
    }

    pub fn forfeit_boss_action(&mut self, action: &DefaultAction) -> Result<(), Error> {
        if self.boss.is_none()
            || self.turn != ENEMY
            || self.phase != Phase::Active
            || self.pending_choice().is_some()
        {
            return Err(Error::IllegalAction);
        }
        if !self.boss_action_available(action) {
            return Err(Error::IllegalAction);
        }
        self.spend_boss_action(action);
        self.normal_actions_used = self.normal_actions_used.saturating_add(1);
        self.actions_left = self.actions_left.saturating_sub(1);
        if self.actions_left == 0 || self.boss_actions_spent() {
            self.end_turn()?;
        }
        Ok(())
    }

    pub fn boss_copied_hit_chance(
        &self,
        kind: PieceType,
        from: Position,
        to: Position,
    ) -> Option<u8> {
        let mut proxy = self.clone();
        let piece = proxy.board.piece_at(from).ok().flatten()?;
        if piece.id != self.royal_id {
            return None;
        }
        proxy
            .board
            .set_piece(from, Some(Piece { kind, ..piece }))
            .ok()?;
        proxy.hit_chance(from, to)
    }

    pub(super) fn boss_copied_movement_actions(
        &self,
        kind: PieceType,
        from: Position,
    ) -> Result<Vec<DefaultAction>, Error> {
        let mut proxy = self.clone();
        let piece = proxy.board.piece_at(from)?.ok_or(Error::IllegalAction)?;
        proxy.board.set_piece(from, Some(Piece { kind, ..piece }))?;
        proxy.piece_actions(kind, from)
    }

    fn boss_shot_directions(action: &DefaultAction) -> Option<[(i32, i32); 3]> {
        if action.first.transition.as_deref() != Some("boss_shot") {
            return None;
        }
        let at = action.first.from;
        let directions = [
            (0, -1),
            (1, -1),
            (1, 0),
            (1, 1),
            (0, 1),
            (-1, 1),
            (-1, 0),
            (-1, -1),
        ];
        let direction = (
            action.first.take.file - at.file,
            action.first.take.rank - at.rank,
        );
        let center = directions.iter().position(|&v| v == direction)?;
        Some(
            [-1isize, 0, 1]
                .map(|offset| directions[(center as isize + offset).rem_euclid(8) as usize]),
        )
    }

    fn boss_shot_ray(&self, from: Position, direction: (i32, i32)) -> ShotgunRay {
        let mut cells = vec![];
        for distance in 1..=3 {
            let at = Position::new(
                from.file + direction.0 * distance,
                from.rank + direction.1 * distance,
            );
            if !self.board.contains(at) {
                break;
            }
            cells.push(at);
            if self.board.piece_at(at).ok().flatten().is_some() {
                break;
            }
        }
        ShotgunRay { direction, cells }
    }

    pub fn boss_shot_area(&self, action: &DefaultAction) -> (Vec<Position>, Vec<Position>) {
        let Some(directions) = Self::boss_shot_directions(action) else {
            return (vec![], vec![]);
        };
        let mut area = vec![];
        let mut targets = vec![];
        for direction in directions {
            let ray = self.boss_shot_ray(action.first.from, direction);
            if let Some(&at) = ray.cells.last() {
                if self.board.piece_at(at).ok().flatten().is_some() {
                    targets.push(at);
                }
            }
            area.extend(ray.cells);
        }
        (area, targets)
    }

    pub(super) fn boss_hit(&mut self, victim: Piece, attacker: Option<PieceId>) -> bool {
        if victim.id != self.royal_id {
            return true;
        }
        hit_boss(
            &mut self.boss,
            &mut self.events,
            &self.board,
            victim,
            attacker,
        )
    }

    pub(super) fn boss_on_death(&mut self, victim: Piece, at: Position) {
        death_near_boss(
            &mut self.boss,
            &mut self.absorbed,
            &mut self.events,
            self.royal_id,
            &self.board,
            victim,
            at,
        );
    }

    pub(super) fn boss_special_actions(&self, from: Position) -> Vec<DefaultAction> {
        let Some(boss) = &self.boss else {
            return vec![];
        };
        let Some(piece) = self.board.piece_at(from).ok().flatten() else {
            return vec![];
        };
        if piece.id != self.royal_id {
            return vec![];
        }
        if boss.ammo == 0 {
            let mut step = ActionStep::new(from, from, from, Effect::Catch);
            step.transition = Some("boss_reload".into());
            return vec![DefaultAction::one(step)];
        }
        let mut actions = vec![];
        for (dx, dy) in [
            (0, -1),
            (1, -1),
            (1, 0),
            (1, 1),
            (0, 1),
            (-1, 1),
            (-1, 0),
            (-1, -1),
        ] {
            let to = Position::new(from.file + dx, from.rank + dy);
            if self.board.contains(to) {
                let mut step = ActionStep::new(from, to, from, Effect::Catch);
                step.transition = Some("boss_shot".into());
                actions.push(DefaultAction::one(step));
            }
        }
        actions
    }

    pub(super) fn boss_copied_special_actions(&self, from: Position) -> Vec<DefaultAction> {
        let Some(kinds) = self.absorbed.get(&self.royal_id) else {
            return vec![];
        };
        let mut kinds: Vec<_> = kinds.iter().copied().collect();
        kinds.sort_unstable();
        let mut result = vec![];
        for kind in kinds {
            let mut proxy = self.clone();
            let Some(piece) = proxy.board.piece_at(from).ok().flatten() else {
                continue;
            };
            if proxy
                .board
                .set_piece(from, Some(Piece { kind, ..piece }))
                .is_err()
            {
                continue;
            }
            for mut action in proxy.special_actions(from) {
                let original = action.first.transition.take().unwrap_or_default();
                action.first.transition = Some(format!("boss_copy:{kind}:{original}"));
                result.push(action);
            }
        }
        result
    }

    pub(super) fn boss_copied_action(&mut self, action: &DefaultAction) -> Result<bool, Error> {
        let Some((kind, original)) = action
            .first
            .transition
            .as_deref()
            .and_then(|tag| tag.strip_prefix("boss_copy:"))
            .and_then(|tag| tag.split_once(':'))
        else {
            return Ok(false);
        };
        let kind = kind
            .parse::<PieceType>()
            .map_err(|_| Error::IllegalAction)?;
        if !self
            .absorbed
            .get(&self.royal_id)
            .is_some_and(|kinds| kinds.contains(&kind))
        {
            return Err(Error::IllegalAction);
        }
        let from = action.first.from;
        let Some(piece) = self.board.piece_at(from)? else {
            return Err(Error::IllegalAction);
        };
        if piece.id != self.royal_id || piece.kind != pieces::TRIBAL_DICTATOR {
            return Err(Error::IllegalAction);
        }
        self.board.set_piece(from, Some(Piece { kind, ..piece }))?;
        let mut copied = action.clone();
        copied.first.transition = (!original.is_empty()).then(|| original.to_string());
        let result = self.apply_effect(&copied, ActionSource::PieceMove);
        if let Some((at, current)) = self.piece_position(self.royal_id) {
            self.board.set_piece(
                at,
                Some(Piece {
                    kind: pieces::TRIBAL_DICTATOR,
                    ..current
                }),
            )?;
        }
        result?;
        Ok(true)
    }

    pub(super) fn boss_pull_hook(&mut self) -> Result<(), Error> {
        let Some(id) = self.boss.as_mut().and_then(|b| b.hook.take()) else {
            return Ok(());
        };
        let Some((boss_at, _)) = self.piece_position(self.royal_id) else {
            return Ok(());
        };
        let Some((target_at, target)) = self.piece_position(id) else {
            return Ok(());
        };
        let mut empty = [
            (0, -1),
            (1, -1),
            (1, 0),
            (1, 1),
            (0, 1),
            (-1, 1),
            (-1, 0),
            (-1, -1),
        ]
        .into_iter()
        .map(|(dx, dy)| Position::new(boss_at.file + dx, boss_at.rank + dy))
        .filter(|&p| self.board.contains(p) && self.board.piece_at(p).ok().flatten().is_none())
        .collect::<Vec<_>>();
        empty.sort_by_key(|p| {
            (
                (p.file - target_at.file)
                    .abs()
                    .max((p.rank - target_at.rank).abs()),
                p.rank,
                p.file,
            )
        });
        if let Some(to) = empty.first().copied() {
            self.board.set_piece(target_at, None)?;
            self.board.set_piece(to, Some(target))?;
            self.sync_players();
            self.combat_events.push(CombatEvent::ChainPull {
                boss_id: self.royal_id,
                boss_at,
                piece_id: id,
                from: target_at,
                to,
            });
            self.events.push(format!(
                "갈고리 반격: {}을 {}에서 {}로 끌어왔습니다.",
                piece_name(target.kind),
                square(target_at),
                square(to)
            ));
        } else {
            self.events
                .push("갈고리 반격: 독재자 주변에 빈칸이 없어 끌어오지 못했습니다.".into());
        }
        Ok(())
    }

    pub(super) fn boss_action(&mut self, action: &DefaultAction) -> Result<bool, Error> {
        let Some(tag) = action.first.transition.as_deref() else {
            return Ok(false);
        };
        let Some((at, actor)) = self.piece_position(self.royal_id) else {
            return Ok(false);
        };
        if actor.kind != pieces::TRIBAL_DICTATOR || action.first.from != at {
            return Ok(false);
        }
        if tag == "boss_reload" {
            if let Some(boss) = self.boss.as_mut() {
                boss.ammo = 2;
            }
            self.events.push("부족의 독재자: 샷건 재장전 · 2/2.".into());
            return Ok(true);
        }
        if tag != "boss_shot" {
            return Ok(false);
        }
        let Some(boss) = self.boss.as_mut() else {
            return Ok(false);
        };
        if boss.ammo == 0 {
            return Err(Error::IllegalAction);
        }
        boss.ammo -= 1;
        let Some(directions) = Self::boss_shot_directions(action) else {
            return Err(Error::IllegalAction);
        };
        self.events.push(format!(
            "부족의 독재자: 3갈래 샷건 발사 · 탄약 {}/2.",
            boss.ammo
        ));
        let mut rays = vec![];
        let mut hits = vec![];
        let mut triggered_wards = vec![];
        for direction in directions {
            // Resolve against the board at this ray's firing time. Earlier rays
            // can trigger explosions which remove a later ray's first blocker.
            let ray = self.boss_shot_ray(at, direction);
            if let Some(&target) = ray.cells.last() {
                if let Some(victim) = self.board.piece_at(target)? {
                    let chance = self.hit_chance(at, target).unwrap_or(0);
                    let hit = self.mechanics.boss_shot_hit(chance);
                    self.events.push(format!(
                        "샷건 {} · {}% · {}.",
                        square(target),
                        chance,
                        if hit { "명중" } else { "빗나감" }
                    ));
                    let mut result = ShotgunResult::Miss;
                    if hit {
                        if self.wards.remove(&victim.id) {
                            result = ShotgunResult::Ward;
                            triggered_wards.push((victim.id, target));
                            self.events.push(format!(
                                "가호: {}에게 명중한 사격을 막았습니다.",
                                piece_name(victim.kind)
                            ));
                        } else {
                            self.kill_at(target, false)?;
                            result = if victim.kind != pieces::CORPSE
                                && self
                                    .piece_position(victim.id)
                                    .is_some_and(|(_, piece)| piece.kind == pieces::CORPSE)
                            {
                                ShotgunResult::Corpse
                            } else {
                                ShotgunResult::Hit
                            };
                            if self.piece_position(victim.id).is_none() {
                                self.record_kill(actor, victim);
                            }
                        }
                    }
                    hits.push(ShotgunHit {
                        at: target,
                        piece_id: victim.id,
                        result,
                    });
                }
            }
            rays.push(ray);
        }
        self.combat_events.push(CombatEvent::Shotgun {
            actor_id: actor.id,
            from: at,
            toward: action.first.take,
            rays,
            hits,
        });
        for (piece_id, at) in triggered_wards {
            self.combat_events.push(CombatEvent::StatusTriggered {
                piece_id,
                at,
                status: "ward",
            });
        }
        self.sync_players();
        self.update_result(ResultReason::RoyalCaptured);
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(file: i32, rank: i32) -> Position {
        Position::new(file, rank)
    }
    fn player(id: PieceId, kind: PieceType, at: Position) -> PlayerPiece {
        PlayerPiece {
            id,
            kind,
            origin: at,
            position: Some(at),
            moved: false,
            upgrades: vec![],
            persistent: BTreeMap::new(),
        }
    }
    fn battle(extra_players: Vec<PlayerPiece>, extra_enemies: Vec<EnemyPiece>) -> Battle {
        let mut players = vec![player(1, pieces::KING, p(7, 7))];
        players.extend(extra_players);
        let mut enemies = vec![EnemyPiece {
            id: 100,
            kind: pieces::TRIBAL_DICTATOR,
            position: p(4, 4),
        }];
        enemies.extend(extra_enemies);
        Battle::new(BattleStart {
            players,
            encounter: Encounter {
                enemies,
                royal_id: 100,
                additional_royal_ids: vec![],
            },
        })
        .unwrap()
    }

    fn action_to(battle: &Battle, from: Position, to: Position) -> DefaultAction {
        battle
            .legal_actions(from)
            .unwrap()
            .into_iter()
            .find(|action| action.steps().last().unwrap().move_to == to)
            .unwrap()
    }

    fn shotgun(battle: &Battle, from: Position, toward: Position) -> DefaultAction {
        battle
            .legal_actions(from)
            .unwrap()
            .into_iter()
            .find(|action| {
                action.first.transition.as_deref() == Some("boss_shot")
                    && action.first.take == toward
            })
            .unwrap()
    }

    #[test]
    fn boss_moves_then_fires_from_new_square_and_resets_next_turn() {
        let mut battle = battle(vec![player(2, pieces::PAWN, p(3, 2))], vec![]);
        battle.end_turn().unwrap();
        assert_eq!(battle.actions_left, 2);
        let movement = action_to(&battle, p(4, 4), p(3, 3));
        battle.apply_piece_action(&movement).unwrap();
        assert_eq!(battle.turn, ENEMY);
        assert_eq!(battle.actions_left, 1);
        assert!(battle.boss().unwrap().movement_used);
        assert!(!battle.boss().unwrap().special_used);
        assert!(battle
            .legal_actions(p(3, 3))
            .unwrap()
            .iter()
            .all(|action| !Battle::boss_action_is_movement(action)));
        let shot = shotgun(&battle, p(3, 3), p(3, 2));
        battle.apply_piece_action(&shot).unwrap();
        assert_eq!(battle.turn, PLAYER);
        assert!(battle.piece_position(2).is_none());
        assert!(matches!(
            battle.combat_events.last(),
            Some(CombatEvent::Shotgun { from, .. }) if *from == p(3, 3)
        ));
        battle.end_turn().unwrap();
        assert_eq!(battle.actions_left, 2);
        assert!(!battle.boss().unwrap().movement_used);
        assert!(!battle.boss().unwrap().special_used);
        let available = battle.legal_actions(p(3, 3)).unwrap();
        assert!(available.iter().any(Battle::boss_action_is_movement));
        assert!(available
            .iter()
            .any(|a| !Battle::boss_action_is_movement(a)));
    }

    #[test]
    fn boss_fires_then_moves_but_cannot_repeat_a_slot_or_gain_a_third_action() {
        let mut battle = battle(vec![], vec![]);
        battle.grant_agility(100, 3);
        battle.end_turn().unwrap();
        assert_eq!(battle.actions_left, 2);
        let shot = shotgun(&battle, p(4, 4), p(4, 3));
        battle.apply_piece_action(&shot).unwrap();
        assert_eq!(battle.turn, ENEMY);
        assert_eq!(battle.actions_left, 1);
        assert!(battle.boss().unwrap().special_used);
        assert!(battle
            .legal_actions(p(4, 4))
            .unwrap()
            .iter()
            .all(Battle::boss_action_is_movement));
        assert!(battle.apply_piece_action_unchecked(&shot).is_err());
        assert_eq!(battle.boss().unwrap().ammo, 1);
        let movement = action_to(&battle, p(4, 4), p(3, 4));
        battle.grant_agility(100, 2);
        battle.apply_piece_action(&movement).unwrap();
        assert_eq!(battle.turn, PLAYER);
        assert_eq!(battle.piece_position(100).unwrap().0, p(3, 4));
    }

    #[test]
    fn boss_reload_and_copied_shot_share_special_slot_and_allow_movement() {
        let mut reloading = battle(vec![], vec![]);
        reloading.boss.as_mut().unwrap().ammo = 0;
        reloading.end_turn().unwrap();
        let reload = reloading
            .legal_actions(p(4, 4))
            .unwrap()
            .into_iter()
            .find(|a| a.first.transition.as_deref() == Some("boss_reload"))
            .unwrap();
        reloading.apply_piece_action(&reload).unwrap();
        assert_eq!(reloading.boss().unwrap().ammo, 2);
        assert_eq!(reloading.turn, ENEMY);
        assert!(reloading
            .legal_actions(p(4, 4))
            .unwrap()
            .iter()
            .all(Battle::boss_action_is_movement));
        let movement = action_to(&reloading, p(4, 4), p(3, 4));
        reloading.apply_piece_action(&movement).unwrap();
        assert_eq!(reloading.turn, PLAYER);

        let mut copied = battle(vec![player(2, pieces::PAWN, p(4, 6))], vec![]);
        copied
            .absorbed
            .entry(100)
            .or_default()
            .insert(pieces::PISTOL_INFANTRY);
        copied.end_turn().unwrap();
        let shot = copied
            .legal_actions(p(4, 4))
            .unwrap()
            .into_iter()
            .find(|a| {
                !Battle::boss_action_is_movement(a)
                    && a.first.take == p(4, 6)
                    && a.first
                        .transition
                        .as_deref()
                        .is_some_and(|tag| tag.starts_with("boss_copy:"))
            })
            .unwrap();
        copied.force_search_shot(false);
        copied.apply_piece_action(&shot).unwrap();
        assert_eq!(copied.turn, ENEMY);
        assert_eq!(
            copied.piece_position(100).unwrap().1.kind,
            pieces::TRIBAL_DICTATOR
        );
        assert!(copied.boss().unwrap().special_used);
        assert!(copied
            .legal_actions(p(4, 4))
            .unwrap()
            .iter()
            .all(Battle::boss_action_is_movement));
        let movement = action_to(&copied, p(4, 4), p(3, 4));
        copied.apply_piece_action(&movement).unwrap();
        assert_eq!(copied.turn, PLAYER);
    }

    #[test]
    fn warded_capture_and_forfeited_intent_spend_their_original_slots() {
        let mut battle = battle(vec![player(2, pieces::PAWN, p(3, 4))], vec![]);
        battle.grant_ward(2).unwrap();
        battle.end_turn().unwrap();
        let capture = action_to(&battle, p(4, 4), p(3, 4));
        let hash = battle.search_hash();
        battle.apply_piece_action(&capture).unwrap();
        assert_eq!(battle.piece_position(100).unwrap().0, p(4, 4));
        assert!(battle.boss().unwrap().movement_used);
        assert_ne!(battle.search_hash(), hash);
        assert!(battle.apply_piece_action_unchecked(&capture).is_err());
        assert!(battle.forfeit_boss_action(&capture).is_err());
        let shot = shotgun(&battle, p(4, 4), p(4, 3));
        battle.forfeit_boss_action(&shot).unwrap();
        assert_eq!(battle.turn, PLAYER);
        assert_eq!(battle.boss().unwrap().ammo, 2);
    }

    #[test]
    fn pending_reaction_rejects_forfeit_without_spending_remaining_boss_slot() {
        let mut battle = battle(vec![player(2, pieces::PAWN, p(3, 4))], vec![]);
        battle
            .v030_card_effect(
                crate::content::cards::CardKind::ReliefForce,
                &crate::content::cards::CardSelection::single(None),
            )
            .unwrap();
        battle.end_turn().unwrap();
        let capture = action_to(&battle, p(4, 4), p(3, 4));
        battle.apply_piece_action(&capture).unwrap();
        let reaction = battle.pending_choice().unwrap();
        let shot = battle.boss_special_actions(p(3, 4)).remove(0);
        let hash = battle.search_hash();
        assert!(battle.forfeit_boss_action(&shot).is_err());
        assert_eq!(battle.search_hash(), hash);
        assert_eq!(battle.actions_left, 1);
        assert_eq!(battle.normal_actions_used, 1);
        assert!(battle.boss().unwrap().movement_used);
        assert!(!battle.boss().unwrap().special_used);
        battle.resolve_choice(reaction.targets[0]).unwrap();
        assert_eq!(battle.turn, ENEMY);
        battle.apply_piece_action(&shot).unwrap();
        assert_eq!(battle.turn, PLAYER);
    }

    #[test]
    fn ordinary_enemy_shooting_still_ends_a_multi_action_turn() {
        let mut battle = battle(vec![player(2, pieces::PAWN, p(4, 6))], vec![]);
        battle.boss = None;
        battle
            .board
            .set_piece(
                p(4, 4),
                Some(Piece {
                    id: 100,
                    owner: ENEMY,
                    kind: pieces::PISTOL_INFANTRY,
                }),
            )
            .unwrap();
        battle.mechanics.enemy_actions = 2;
        battle.end_turn().unwrap();
        assert_eq!(battle.actions_left, 2);
        let shot = battle
            .legal_actions(p(4, 4))
            .unwrap()
            .into_iter()
            .find(|a| a.first.move_to == p(4, 4) && a.first.take == p(4, 6))
            .unwrap();
        battle.force_search_shot(false);
        battle.apply_piece_action(&shot).unwrap();
        assert_eq!(battle.turn, PLAYER);
    }

    #[test]
    fn base_movement_is_king_but_learned_rook_movement_remains_available() {
        let mut battle = battle(vec![player(2, pieces::ROOK, p(3, 4))], vec![]);
        battle.end_turn().unwrap();
        let base: Vec<_> = battle
            .legal_actions(p(4, 4))
            .unwrap()
            .into_iter()
            .filter(|a| a.first.move_to != p(4, 4))
            .collect();
        assert_eq!(base.len(), 8);
        assert!(base.iter().all(|a| {
            (a.first.move_to.file - 4)
                .abs()
                .max((a.first.move_to.rank - 4).abs())
                == 1
        }));
        assert!(base.iter().any(|a| a.first.take == p(3, 4)));

        battle.kill_at(p(3, 4), false).unwrap();
        assert!(battle.absorbed_kinds(100).contains(&pieces::ROOK));
        let learned = battle
            .legal_actions(p(4, 4))
            .unwrap()
            .into_iter()
            .find(|a| a.first.move_to == p(4, 0))
            .unwrap();
        assert!(learned
            .first
            .transition
            .as_deref()
            .unwrap()
            .starts_with("boss_copy:"));
        battle.apply_piece_action(&learned).unwrap();
        assert_eq!(battle.piece_position(100).unwrap().0, p(4, 0));
        assert_eq!(battle.boss().unwrap().hp, 6);
    }

    #[test]
    fn independent_removal_events_lower_hp_and_only_fifth_kills() {
        let mut battle = battle(
            vec![],
            vec![EnemyPiece {
                id: 101,
                kind: pieces::ROOK,
                position: p(0, 0),
            }],
        );
        for remaining in (1..=4).rev() {
            battle.kill_at(p(4, 4), false).unwrap();
            assert_eq!(battle.boss().unwrap().hp, remaining);
            assert!(battle.board.piece_at(p(4, 4)).unwrap().is_some());
        }
        battle.kill_at(p(4, 4), false).unwrap();
        battle.update_result(ResultReason::RoyalCaptured);
        assert_eq!(battle.boss().unwrap().hp, 0);
        assert!(battle.board.piece_at(p(4, 4)).unwrap().is_none());
        assert!(battle.board.piece_at(p(0, 0)).unwrap().is_some());
        assert!(matches!(
            battle.result,
            GameResult::Win { winner: PLAYER, .. }
        ));
    }

    #[test]
    fn area_card_removal_damages_boss_without_declaring_a_death() {
        let mut battle = battle(vec![], vec![]);
        assert_eq!(battle.remove_nonroyals(&[100]).unwrap(), 0);
        assert_eq!(battle.boss().unwrap().hp, 4);
        assert!(battle.piece_position(100).is_some());
        assert_eq!(battle.result, GameResult::Ongoing);
    }

    #[test]
    fn corpse_transition_heals_once_and_later_corpse_death_heals_again() {
        let mut battle = battle(vec![player(2, pieces::KNIGHT, p(3, 4))], vec![]);
        battle.mechanics.statuses.get_mut(&2).unwrap().ingredient = true;
        battle.kill_at(p(3, 4), true).unwrap();
        assert_eq!(battle.boss().unwrap().hp, 6);
        assert_eq!(
            battle.board.piece_at(p(3, 4)).unwrap().unwrap().kind,
            pieces::CORPSE
        );
        assert!(battle.absorbed_kinds(100).contains(&pieces::KNIGHT));
        battle.kill_at(p(3, 4), false).unwrap();
        assert_eq!(battle.boss().unwrap().hp, 7);
    }

    #[test]
    fn attacker_is_hooked_and_pulled_before_boss_action() {
        let mut battle = battle(vec![player(2, pieces::ROOK, p(4, 6))], vec![]);
        assert!(!battle.kill_at_by(p(4, 4), false, Some(2)).unwrap());
        assert_eq!(battle.boss().unwrap().hook, Some(2));
        battle.end_turn().unwrap();
        let (at, _) = battle.piece_position(2).unwrap();
        assert_eq!(
            (at.file - p(4, 4).file)
                .abs()
                .max((at.rank - p(4, 4).rank).abs()),
            1
        );
        assert_eq!(battle.boss().unwrap().hook, None);
        assert_eq!(battle.turn, ENEMY);
        assert_eq!(
            battle.combat_events,
            vec![CombatEvent::ChainPull {
                boss_id: 100,
                boss_at: p(4, 4),
                piece_id: 2,
                from: p(4, 6),
                to: at,
            }]
        );
    }

    #[test]
    fn missing_or_blocked_hook_has_no_pull_event() {
        let mut battle = battle(vec![player(2, pieces::ROOK, p(4, 6))], vec![]);
        battle.boss_pull_hook().unwrap();
        assert!(battle.combat_events.is_empty());
        battle.boss.as_mut().unwrap().hook = Some(999);
        battle.boss_pull_hook().unwrap();
        assert!(battle.combat_events.is_empty());
        for (index, at) in (3..=5)
            .flat_map(|rank| (3..=5).map(move |file| p(file, rank)))
            .filter(|&at| at != p(4, 4))
            .enumerate()
        {
            battle
                .board
                .set_piece(
                    at,
                    Some(Piece {
                        id: 200 + index as u32,
                        owner: ENEMY,
                        kind: pieces::PAWN,
                    }),
                )
                .unwrap();
        }
        battle.boss.as_mut().unwrap().hook = Some(2);
        battle.boss_pull_hook().unwrap();
        assert!(battle.combat_events.is_empty());
        assert_eq!(battle.piece_position(2).unwrap().0, p(4, 6));
    }

    #[test]
    fn shotgun_stops_at_first_piece_and_spends_one_shell() {
        let mut battle = battle(
            vec![
                player(2, pieces::PAWN, p(4, 3)),
                player(3, pieces::QUEEN, p(4, 2)),
            ],
            vec![],
        );
        battle.end_turn().unwrap();
        let action = battle
            .legal_actions(p(4, 4))
            .unwrap()
            .into_iter()
            .find(|a| a.first.transition.as_deref() == Some("boss_shot") && a.first.take == p(4, 3))
            .unwrap();
        let (area, targets) = battle.boss_shot_area(&action);
        assert_eq!(targets, vec![p(4, 3)]);
        assert!(!area.contains(&p(4, 2)));
        assert!(!area.contains(&p(4, 1)));
        battle.apply_piece_action(&action).unwrap();
        assert!(battle.piece_position(2).is_none());
        assert!(battle.piece_position(3).is_some());
        assert_eq!(battle.boss().unwrap().ammo, 1);
        let CombatEvent::Shotgun { rays, hits, .. } = &battle.combat_events[0] else {
            panic!("expected resolved shotgun event");
        };
        assert_eq!(
            rays.iter()
                .flat_map(|ray| ray.cells.iter().copied())
                .collect::<Vec<_>>(),
            area
        );
        assert_eq!(rays[0].cells, vec![p(3, 3), p(2, 2), p(1, 1)]);
        assert_eq!(rays[1].cells, vec![p(4, 3)]);
        assert_eq!(rays[2].cells, vec![p(5, 3), p(6, 2), p(7, 1)]);
        assert_eq!(
            hits,
            &[ShotgunHit {
                at: p(4, 3),
                piece_id: 2,
                result: ShotgunResult::Hit
            }]
        );
    }

    #[test]
    fn shotgun_event_records_resolved_miss_ward_and_corpse_removal() {
        let mut battle = battle(
            vec![
                player(2, pieces::PAWN, p(3, 3)),
                player(3, pieces::PAWN, p(4, 3)),
            ],
            vec![EnemyPiece {
                id: 101,
                kind: pieces::CORPSE,
                position: p(5, 3),
            }],
        );
        battle.end_turn().unwrap();
        battle.grant_ward(3).unwrap();
        let action = battle
            .legal_actions(p(4, 4))
            .unwrap()
            .into_iter()
            .find(|a| a.first.transition.as_deref() == Some("boss_shot") && a.first.take == p(4, 3))
            .unwrap();
        battle.force_search_boss_shots(vec![false, true, true]);
        battle.apply_piece_action(&action).unwrap();
        let CombatEvent::Shotgun { hits, .. } = &battle.combat_events[0] else {
            panic!("expected resolved shotgun event");
        };
        assert_eq!(
            hits.iter().map(|hit| hit.result).collect::<Vec<_>>(),
            vec![ShotgunResult::Miss, ShotgunResult::Ward, ShotgunResult::Hit]
        );
        assert!(battle.piece_position(2).is_some());
        assert!(battle.piece_position(3).is_some());
        assert!(!battle.has_ward(3));
        assert!(battle.piece_position(101).is_none());
    }

    #[test]
    fn shotgun_event_traces_later_rays_after_explosion_removes_blocker() {
        let mut battle = battle(
            vec![
                player(2, pieces::PAWN, p(4, 3)),
                player(3, pieces::QUEEN, p(4, 2)),
            ],
            vec![EnemyPiece {
                id: 101,
                kind: pieces::WAREHOUSE,
                position: p(3, 3),
            }],
        );
        battle.end_turn().unwrap();
        let action = battle
            .legal_actions(p(4, 4))
            .unwrap()
            .into_iter()
            .find(|a| a.first.transition.as_deref() == Some("boss_shot") && a.first.take == p(4, 3))
            .unwrap();
        assert!(!battle.boss_shot_area(&action).0.contains(&p(4, 2)));
        battle.force_search_boss_shots(vec![true, true]);
        battle.apply_piece_action(&action).unwrap();
        let CombatEvent::Shotgun { rays, hits, .. } = &battle.combat_events[0] else {
            panic!("expected resolved shotgun event");
        };
        assert_eq!(rays[1].cells, vec![p(4, 3), p(4, 2)]);
        assert_eq!(
            hits.iter().map(|hit| hit.piece_id).collect::<Vec<_>>(),
            vec![101, 3]
        );
        assert!(battle.piece_position(2).is_none());
        assert!(battle.piece_position(3).is_none());
    }

    #[test]
    fn shotgun_three_rays_hit_allies_and_enemies_and_feed_the_boss() {
        let mut battle = battle(
            vec![
                player(2, pieces::PAWN, p(3, 3)),
                player(3, pieces::PAWN, p(4, 3)),
            ],
            vec![EnemyPiece {
                id: 101,
                kind: pieces::KNIGHT,
                position: p(5, 3),
            }],
        );
        battle.end_turn().unwrap();
        let action = battle
            .legal_actions(p(4, 4))
            .unwrap()
            .into_iter()
            .find(|a| a.first.transition.as_deref() == Some("boss_shot") && a.first.take == p(4, 3))
            .unwrap();
        battle.apply_piece_action(&action).unwrap();
        for id in [2, 3, 101] {
            assert!(battle.piece_position(id).is_none());
        }
        assert_eq!(battle.boss().unwrap().hp, 8);
        assert!(battle.absorbed_kinds(100).contains(&pieces::KNIGHT));
    }

    #[test]
    fn learned_drone_ability_uses_original_explosion_and_boss_hp() {
        let mut battle = battle(
            vec![],
            vec![EnemyPiece {
                id: 101,
                kind: pieces::C4_DRONE,
                position: p(3, 4),
            }],
        );
        battle.kill_at(p(3, 4), false).unwrap();
        assert!(battle.absorbed_kinds(100).contains(&pieces::C4_DRONE));
        let before = battle.boss().unwrap().hp;
        battle.end_turn().unwrap();
        let action = battle
            .legal_actions(p(4, 4))
            .unwrap()
            .into_iter()
            .find(|a| {
                a.first
                    .transition
                    .as_deref()
                    .is_some_and(|tag| tag.starts_with("boss_copy:"))
                    && a.first.take == p(4, 4)
            })
            .unwrap();
        battle.apply_piece_action(&action).unwrap();
        assert_eq!(battle.boss().unwrap().hp, before - 1);
        assert!(battle.piece_position(100).is_some());
    }
}
