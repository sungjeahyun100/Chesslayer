use super::*;
use crate::{ActionStep, Effect};

#[derive(Clone, Debug)]
pub struct BossState {
    pub hp: i32,
    pub ammo: u8,
    pub hook: Option<PieceId>,
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

    pub fn forfeit_boss_action(&mut self) -> Result<(), Error> {
        if self.boss.is_none() || self.turn != ENEMY || self.phase != Phase::Active {
            return Err(Error::IllegalAction);
        }
        self.normal_actions_used = self.normal_actions_used.saturating_add(1);
        self.actions_left = self.actions_left.saturating_sub(1);
        if self.actions_left == 0 {
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

    pub fn boss_shot_area(&self, action: &DefaultAction) -> (Vec<Position>, Vec<Position>) {
        if action.first.transition.as_deref() != Some("boss_shot") {
            return (vec![], vec![]);
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
        let Some(center) = directions.iter().position(|&v| v == direction) else {
            return (vec![], vec![]);
        };
        let mut area = vec![];
        let mut targets = vec![];
        for offset in [-1isize, 0, 1] {
            let (dx, dy) = directions[(center as isize + offset).rem_euclid(8) as usize];
            let mut first_found = false;
            for distance in 1..=3 {
                let at = Position::new(at.file + dx * distance, at.rank + dy * distance);
                if !self.board.contains(at) {
                    break;
                }
                area.push(at);
                if !first_found && self.board.piece_at(at).ok().flatten().is_some() {
                    targets.push(at);
                    first_found = true;
                }
            }
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
        let dx = action.first.take.file - at.file;
        let dy = action.first.take.rank - at.rank;
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
        let Some(center) = directions.iter().position(|&v| v == (dx, dy)) else {
            return Err(Error::IllegalAction);
        };
        self.events.push(format!(
            "부족의 독재자: 3갈래 샷건 발사 · 탄약 {}/2.",
            boss.ammo
        ));
        for offset in [-1isize, 0, 1] {
            let (ray_x, ray_y) = directions[(center as isize + offset).rem_euclid(8) as usize];
            for distance in 1..=3 {
                let target = Position::new(at.file + ray_x * distance, at.rank + ray_y * distance);
                if !self.board.contains(target) {
                    break;
                }
                let Some(victim) = self.board.piece_at(target)? else {
                    continue;
                };
                let chance = self.hit_chance(at, target).unwrap_or(0);
                let hit = self
                    .mechanics
                    .search_shot_outcome
                    .take()
                    .unwrap_or_else(|| self.mechanics.rng.index(100) < chance as usize);
                self.events.push(format!(
                    "샷건 {} · {}% · {}.",
                    square(target),
                    chance,
                    if hit { "명중" } else { "빗나감" }
                ));
                if hit {
                    if self.wards.remove(&victim.id) {
                        self.events.push(format!(
                            "가호: {}에게 명중한 사격을 막았습니다.",
                            piece_name(victim.kind)
                        ));
                    } else {
                        self.kill_at(target, false)?;
                        if self.piece_position(victim.id).is_none() {
                            self.record_kill(actor, victim);
                        }
                    }
                }
                break;
            }
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
        battle.apply_piece_action(&action).unwrap();
        assert!(battle.piece_position(2).is_none());
        assert!(battle.piece_position(3).is_some());
        assert_eq!(battle.boss().unwrap().ammo, 1);
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
