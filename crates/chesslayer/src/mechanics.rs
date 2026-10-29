//! Native combat effects layered over each piece's independent Chesssembly program.
use super::*;
use crate::content::{pieces::*, Relic};
use serde::Serialize;

#[derive(Clone, Debug, Default, Serialize)]
pub struct Status {
    pub agile: u8,
    pub weak: u8,
    pub hunger: u8,
    pub madness: u8,
    pub guilt: u8,
    pub blessed: bool,
    pub ingredient: bool,
    pub disabled: bool,
    pub etiquette: bool,
}
impl Status {
    pub fn load(values: &BTreeMap<String, i32>) -> Self {
        let get = |key: &str| *values.get(key).unwrap_or(&0);
        Self {
            hunger: get("hunger").clamp(0, 3) as u8,
            madness: get("madness").clamp(0, 2) as u8,
            guilt: get("guilt").clamp(0, 1) as u8,
            disabled: get("disabled") != 0,
            blessed: get("blessed") != 0,
            ..Self::default()
        }
    }
    pub fn save(&self, values: &mut BTreeMap<String, i32>) {
        for (key, value) in [
            ("hunger", self.hunger as i32),
            ("madness", self.madness as i32),
            ("guilt", self.guilt as i32),
            ("disabled", self.disabled as i32),
            ("blessed", self.blessed as i32),
        ] {
            if value != 0 || values.contains_key(key) {
                values.insert(key.into(), value);
            }
        }
    }
}
#[derive(Clone, Serialize)]
pub struct AbilityView {
    pub id: PieceId,
    pub name: &'static str,
    pub targets: Vec<PieceId>,
    pub chances: Vec<u8>,
}
#[derive(Clone, Serialize)]
pub struct ChoiceView {
    pub prompt: String,
    pub kind: String,
    pub targets: Vec<PieceId>,
    pub cells: Vec<usize>,
    pub amount: u8,
}
#[derive(Clone)]
pub struct Mechanics {
    pub active: bool,
    pub statuses: HashMap<PieceId, Status>,
    pub relics: Vec<Relic>,
    pub rng: Rng,
    pub ammo: u32,
    pub ammo_mode: bool,
    pub warehouse_side: [bool; 2],
    pub extra_base: [u8; 2],
    pub draws: usize,
    pub mana: u32,
    pub used: HashSet<(PieceId, String)>,
    pub turn_used: HashSet<String>,
    pub corpse_count: [u8; 2],
    pub choices: Vec<ChoiceView>,
    pub last_stand: bool,
    pub killed: bool,
    pub frenzy: bool,
    pub cycles: u8,
    pub bell_target: Option<PieceId>,
}
impl Mechanics {
    pub fn from_roster(roster: &[PlayerPiece]) -> Self {
        Self {
            active: false,
            statuses: roster
                .iter()
                .map(|p| (p.id, Status::load(&p.persistent)))
                .collect(),
            relics: vec![],
            rng: Rng::new(0),
            ammo: 0,
            ammo_mode: false,
            warehouse_side: [roster.iter().any(|p| p.kind == WAREHOUSE), false],
            extra_base: [0, 0],
            draws: 0,
            mana: 0,
            used: HashSet::new(),
            turn_used: HashSet::new(),
            corpse_count: [0, 0],
            choices: vec![],
            last_stand: false,
            killed: false,
            frenzy: false,
            cycles: 0,
            bell_target: None,
        }
    }
}
pub fn accuracy(base: u8, reference: u8, decay: u8, distance: u8) -> u8 {
    base.saturating_sub(distance.saturating_sub(reference).saturating_mul(decay))
        .min(100)
}
fn distance(a: Position, b: Position) -> u8 {
    (a.file - b.file).abs().max((a.rank - b.rank).abs()) as u8
}
fn cell(p: Position) -> usize {
    (p.rank * 8 + p.file) as usize
}
impl Battle {
    pub fn status(&self, id: PieceId) -> Status {
        self.mechanics
            .statuses
            .get(&id)
            .cloned()
            .unwrap_or_default()
    }
    pub fn status_mut(&mut self, id: PieceId) -> &mut Status {
        self.mechanics.statuses.entry(id).or_default()
    }
    pub fn located(&self, id: PieceId) -> Option<(Position, Piece)> {
        self.board.cells().iter().enumerate().find_map(|(i, p)| {
            p.filter(|p| p.id == id)
                .map(|p| (Position::new(i as i32 % 8, i as i32 / 8), p))
        })
    }
    pub fn living(&self, owner: PlayerId) -> Vec<PieceId> {
        self.board
            .cells()
            .iter()
            .flatten()
            .filter(|p| p.owner == owner && p.kind != CORPSE)
            .map(|p| p.id)
            .collect()
    }
    pub fn is_royal(&self, id: PieceId) -> bool {
        self.is_enemy_royal(id) || (!self.mechanics.warehouse_side[0] && id == self.king_id)
    }
    pub fn configure_mechanics(&mut self, relics: &[Relic], ammo: Option<u32>, seed: u32) {
        self.mechanics.active = self.board.cells().iter().flatten().any(|p| p.kind >= 40)
            || self
                .mechanics
                .statuses
                .values()
                .any(|s| s.hunger > 0 || s.disabled || s.blessed)
            || relics.contains(&Relic::CarnivalFlare);
        self.mechanics.relics = relics.to_vec();
        self.mechanics.rng = Rng::new(seed);
        self.mechanics.ammo_mode = ammo.is_some();
        self.mechanics.ammo = ammo.unwrap_or(0);
        self.mechanics.warehouse_side[1] = self
            .board
            .cells()
            .iter()
            .flatten()
            .any(|p| p.owner == ENEMY && p.kind == WAREHOUSE);
        let boss = self
            .board
            .cells()
            .iter()
            .flatten()
            .any(|p| p.kind == ETERNAL_GUEST);
        self.mechanics.extra_base = [
            u8::from(relics.contains(&Relic::Agility)),
            u8::from(self.board.cells().iter().flatten().any(|p| {
                p.owner == ENEMY && matches!(p.kind, VANGUARD_COMMANDER | MOUNTED_COMMANDER)
            })),
        ];
        self.actions_left = self.base_actions(PLAYER);
        let carnival = relics.contains(&Relic::CarnivalFlare);
        for p in self
            .board
            .cells()
            .iter()
            .flatten()
            .copied()
            .collect::<Vec<_>>()
        {
            if p.kind == CORPSE {
                continue;
            }
            let s = self.status_mut(p.id);
            s.ingredient = carnival || boss;
            if (carnival && p.owner == PLAYER) || (boss && p.owner == ENEMY) {
                s.blessed = true;
            }
        }
        if relics.contains(&Relic::TornMenu) {
            let ids = self
                .living(ENEMY)
                .into_iter()
                .filter(|id| !self.is_royal(*id))
                .collect::<Vec<_>>();
            if !ids.is_empty() {
                let id = ids[self.mechanics.rng.index(ids.len())];
                self.worsen_hunger(id);
            }
        }
    }
    pub(super) fn base_actions(&self, owner: PlayerId) -> u8 {
        self.actions_per_turn
            .saturating_add(self.mechanics.extra_base[owner as usize])
    }
    pub fn grant_agile(&mut self, id: PieceId, n: u8) {
        let s = self.status_mut(id);
        s.agile = s.agile.saturating_add(n);
        self.refresh_actions();
    }
    pub(super) fn refresh_actions(&mut self) {
        let extra = self
            .board
            .cells()
            .iter()
            .flatten()
            .filter(|p| p.owner == self.turn && p.kind != CORPSE && !self.status(p.id).disabled)
            .map(|p| self.status(p.id).agile)
            .fold(0u8, u8::saturating_add);
        self.actions_left = self
            .base_actions(self.turn)
            .saturating_sub(self.normal_actions_used)
            .saturating_add(extra);
    }
    pub(super) fn spend_action(&mut self, piece: Piece) -> Result<(), Error> {
        self.moved_ids.insert(piece.id);
        self.status_mut(piece.id).etiquette = false;
        if let Some(s) = self.limiter.get_mut(&piece.id).filter(|s| s.active) {
            s.moves_left = s.moves_left.saturating_sub(1);
            self.actions_left = self.actions_left.saturating_sub(1);
        } else if self.normal_actions_used < self.base_actions(piece.owner) {
            self.normal_actions_used = self.normal_actions_used.saturating_add(1);
            self.refresh_actions();
        } else {
            let s = self.status_mut(piece.id);
            s.agile = s.agile.saturating_sub(1);
            self.refresh_actions();
        }
        if let Some(p) = self.players.iter_mut().find(|p| p.id == piece.id) {
            p.moved = true;
        }
        if self.phase == Phase::Active
            && self.actions_left == 0
            && self.mechanics.choices.is_empty()
        {
            self.end_turn()?;
        }
        Ok(())
    }
    pub(super) fn movement_board(&self, kind: PieceType, from: Position) -> Result<Board, Error> {
        let mut board = self.board.clone();
        let owner = board.piece_at(from)?.ok_or(Error::IllegalAction)?.owner;
        for (i, p) in self.board.cells().iter().enumerate() {
            if let Some(mut piece) = *p {
                if piece.kind == CORPSE {
                    let next = if matches!(kind, FUNERAL_CART | SLAUGHTER_CART | FEAST_CART) {
                        None
                    } else {
                        piece.owner = 1 - owner;
                        Some(piece)
                    };
                    board.set_piece(Position::new(i as i32 % 8, i as i32 / 8), next)?;
                }
            }
        }
        Ok(board)
    }
    pub(super) fn extra_corpse_actions(
        &self,
        piece: Piece,
        from: Position,
        actions: &mut Vec<DefaultAction>,
    ) -> Result<(), Error> {
        for (i, p) in self.board.cells().iter().enumerate() {
            if !p.is_some_and(|p| p.kind == CORPSE) {
                continue;
            }
            let to = Position::new(i as i32 % 8, i as i32 / 8);
            let dx = (to.file - from.file).abs();
            let dy = (to.rank - from.rank).abs();
            let adjacent = distance(from, to) == 1;
            let allowed = (self.status(piece.id).etiquette && adjacent)
                || (piece.kind == FRENZIED_DANCER && dx + dy == 1)
                || (matches!(
                    piece.kind,
                    MASKED_DANCER | ECSTATIC_DANCER | TEMPTING_DANCER
                ) && adjacent)
                || (matches!(
                    piece.kind,
                    HUNGRY_FOLLOWER | GLUTTON_FOLLOWER | INVITING_FOLLOWER
                ) && dx == 0
                    && to.rank - from.rank == if piece.owner == PLAYER { -1 } else { 1 });
            if allowed {
                actions.push(DefaultAction::one(crate::ActionStep::new(
                    from,
                    to,
                    to,
                    crate::Effect::Take,
                )));
            }
        }
        Ok(())
    }
    pub(super) fn filter_hunger(&self, piece: Piece, actions: &mut Vec<DefaultAction>) {
        let hunger = self.status(piece.id).hunger;
        let corpse = |a: &DefaultAction| {
            a.steps().any(|s| {
                !matches!(s.effect, crate::Effect::Move | crate::Effect::Shift)
                    && self
                        .board
                        .piece_at(s.take)
                        .ok()
                        .flatten()
                        .is_some_and(|p| p.kind == CORPSE)
            })
        };
        if hunger < 2 || !actions.iter().any(corpse) {
            return;
        }
        actions.retain(|a| {
            if hunger == 3 {
                corpse(a)
            } else {
                !a.steps().any(|s| {
                    !matches!(s.effect, crate::Effect::Move | crate::Effect::Shift)
                        && self
                            .board
                            .piece_at(s.take)
                            .ok()
                            .flatten()
                            .is_some_and(|p| p.kind != CORPSE && p.owner != piece.owner)
                })
            }
        });
    }
    pub(super) fn apply_mechanics_action(&mut self, action: &DefaultAction) -> Result<(), Error> {
        let mut retaliated = false;
        for step in action.steps() {
            let captor = self
                .board
                .piece_at(step.from)?
                .ok_or(Error::IllegalAction)?;
            let victim = self
                .board
                .piece_at(step.take)?
                .filter(|_| !matches!(step.effect, crate::Effect::Move | crate::Effect::Shift));
            if let Some(v) = victim {
                if v.kind != CORPSE {
                    if captor.owner == PLAYER {
                        self.mechanics.killed = true;
                        self.initiative_turns_left = 0;
                    } else {
                        self.enemy_initiative_turns_left = 0;
                    }
                    let retaliates =
                        self.self_defense_target == Some(v.id) && captor.owner == ENEMY;
                    if self.status(v.id).ingredient && !self.is_royal(v.id) && v.kind != WAREHOUSE {
                        self.make_corpse(v.id)?;
                        if retaliates {
                            self.kill_direct(captor.id)?;
                            retaliated = true;
                        }
                        break;
                    }
                    if retaliates {
                        retaliated = true;
                    }
                } else {
                    let mut corpse = v;
                    corpse.owner = 1 - captor.owner;
                    self.board.set_piece(step.take, Some(corpse))?;
                }
            }
            apply_default_action(&mut self.board, &DefaultAction::one(step.clone()))?;
            if let Some(v) = victim {
                if v.kind == CORPSE {
                    self.eat(captor.id)?;
                }
                if v.kind == WAREHOUSE {
                    self.explode(step.take, v.owner)?;
                }
                if captor.kind == EMPEROR && captor.owner == ENEMY && v.owner == PLAYER {
                    self.absorbed.entry(captor.id).or_default().insert(v.kind);
                }
            }
            if retaliated {
                self.kill_direct(captor.id)?;
                break;
            }
        }
        self.sync_players();
        self.update_result(if retaliated {
            ResultReason::SelfDefense
        } else {
            ResultReason::RoyalCaptured
        });
        Ok(())
    }
    pub fn make_corpse(&mut self, id: PieceId) -> Result<(), Error> {
        if self.is_royal(id) {
            return Err(Error::ProtectedRoyal);
        }
        let (pos, mut p) = self.located(id).ok_or(Error::IllegalAction)?;
        if p.kind == CORPSE || p.kind == WAREHOUSE {
            return Err(Error::IllegalAction);
        }
        p.kind = CORPSE;
        self.board.set_piece(pos, Some(p))?;
        self.wards.remove(&id);
        self.events
            .push(format!("{}에 시체가 남았습니다.", square(pos)));
        self.sync_players();
        Ok(())
    }
    pub(super) fn kill_direct(&mut self, id: PieceId) -> Result<(), Error> {
        if let Some((pos, p)) = self.located(id) {
            self.board.set_piece(pos, None)?;
            if p.kind == WAREHOUSE {
                self.explode(pos, p.owner)?;
            }
        }
        self.sync_players();
        Ok(())
    }
    pub(super) fn explode(&mut self, at: Position, owner: PlayerId) -> Result<(), Error> {
        if owner == PLAYER {
            self.mechanics.ammo = self.mechanics.ammo.saturating_sub(200);
        }
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let p = Position::new(at.file + dx, at.rank + dy);
            if let Ok(Some(v)) = self.board.piece_at(p) {
                self.kill_direct(v.id)?;
            }
        }
        self.events
            .push(format!("{} 창고 폭발: 상하좌우 기물 사망", square(at)));
        Ok(())
    }
    pub fn ammo_capacity(&self) -> u32 {
        self.players
            .iter()
            .filter(|p| p.kind == WAREHOUSE && p.position.is_some())
            .count() as u32
            * 200
    }
    pub fn worsen_hunger(&mut self, id: PieceId) {
        let s = self.status_mut(id);
        s.hunger = s.hunger.saturating_add(1).min(3);
    }
    pub fn resolve_mind(&mut self, id: PieceId) -> Result<(), Error> {
        let s = self.status(id);
        if s.madness >= 3 {
            let (pos, mut p) = self.located(id).ok_or(Error::IllegalAction)?;
            p.owner = 1 - p.owner;
            self.board.set_piece(pos, Some(p))?;
            let s = self.status_mut(id);
            s.madness = 0;
            s.blessed = true;
            if p.owner == ENEMY {
                if let Some(roster) = self.players.iter_mut().find(|p| p.id == id) {
                    roster.persistent.insert("defected".into(), 1);
                }
            } else if !self.players.iter().any(|p| p.id == id) {
                let origin = (0..8)
                    .rev()
                    .flat_map(|y| (0..8).map(move |x| Position::new(x, y)))
                    .find(|pos| !self.players.iter().any(|p| p.origin == *pos))
                    .ok_or(Error::IllegalAction)?;
                self.players.push(PlayerPiece {
                    id,
                    kind: p.kind,
                    origin,
                    position: Some(pos),
                    moved: true,
                    upgrades: vec![],
                    persistent: BTreeMap::new(),
                });
            }
            self.events.push(format!(
                "{}이 황충의 축복을 받고 전향했습니다.",
                piece_name(p.kind)
            ));
        }
        if s.guilt >= 2 {
            let s = self.status_mut(id);
            s.guilt = 0;
            s.disabled = true;
            self.events
                .push("죄책감: 영구 전투불능. 휴식에서 고해성사가 필요합니다.".into());
        }
        self.sync_players();
        self.update_result(ResultReason::RoyalCaptured);
        self.refresh_actions();
        Ok(())
    }
    pub(super) fn once(&mut self, id: PieceId, key: &str) -> bool {
        self.mechanics.used.insert((id, key.into()))
    }
    pub(super) fn nearby(
        &self,
        id: PieceId,
        owner: PlayerId,
        range: u8,
        nonroyal: bool,
    ) -> Vec<PieceId> {
        let Some((at, _)) = self.located(id) else {
            return vec![];
        };
        self.living(owner)
            .into_iter()
            .filter(|other| {
                *other != id
                    && (!nonroyal || !self.is_royal(*other))
                    && self
                        .located(*other)
                        .is_some_and(|(p, _)| distance(at, p) <= range)
            })
            .collect()
    }
    pub(super) fn choose(&mut self, prompt: &str, kind: &str, targets: Vec<PieceId>, amount: u8) {
        if targets.is_empty() {
            return;
        }
        if self.turn == ENEMY {
            let target = targets[0];
            if kind == "corpse" {
                if let Some((at, p)) = self.located(target) {
                    for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                        let to = Position::new(at.file + dx, at.rank + dy);
                        if self.board.piece_at(to).ok() == Some(None) {
                            let _ = self.board.set_piece(at, None);
                            let _ = self.board.set_piece(to, Some(p));
                            break;
                        }
                    }
                }
            } else if kind == "agile" {
                self.grant_agile(target, amount);
            } else if kind == "hunger" {
                self.worsen_hunger(target);
            }
            return;
        }
        self.mechanics.choices.push(ChoiceView {
            prompt: prompt.into(),
            kind: kind.into(),
            targets,
            cells: vec![],
            amount,
        });
    }
    pub fn eat(&mut self, id: PieceId) -> Result<(), Error> {
        let (at, p) = self.located(id).ok_or(Error::IllegalAction)?;
        self.status_mut(id).hunger = 0;
        if self.status(id).blessed {
            self.grant_agile(id, 3);
        } else {
            if self.mechanics.rng.index(2) == 0 {
                self.status_mut(id).madness += 1;
            } else {
                self.status_mut(id).guilt += 1;
            }
            self.resolve_mind(id)?;
        }
        self.mechanics.corpse_count[p.owner as usize] =
            self.mechanics.corpse_count[p.owner as usize].saturating_add(1);
        if p.owner == ENEMY
            && self
                .board
                .cells()
                .iter()
                .flatten()
                .any(|p| p.kind == ETERNAL_GUEST)
            && self.mechanics.corpse_count[1] <= 3
        {
            self.grant_agile(id, 1);
        }
        if matches!(
            p.kind,
            BUTCHER_CART | FUNERAL_CART | SLAUGHTER_CART | FEAST_CART
        ) && self.once(id, "cart")
        {
            if p.owner == PLAYER {
                self.mechanics.draws += 1;
            }
            if p.kind == SLAUGHTER_CART {
                self.grant_agile(id, 1);
            }
            if p.kind == FEAST_CART {
                let ids = self
                    .corpses()
                    .into_iter()
                    .filter(|other| {
                        self.located(*other)
                            .is_some_and(|(pos, _)| distance(at, pos) <= 2)
                    })
                    .collect();
                self.choose("연회의 수레: 옮길 시체", "corpse", ids, 1);
            }
        }
        if matches!(
            p.kind,
            INTOXICATED_PRIEST | FEAST_PRIEST | LOCUST_PRIEST | HUNGER_PRIEST
        ) && self.once(id, "priest")
        {
            let ids = self.nearby(
                id,
                p.owner,
                if p.kind == INTOXICATED_PRIEST { 1 } else { 2 },
                false,
            );
            self.choose(
                "사제: 민첩을 받을 다른 아군",
                "agile",
                ids,
                if p.kind == LOCUST_PRIEST { 2 } else { 1 },
            );
        }
        if matches!(p.kind, ECSTATIC_DANCER | GLUTTON_FOLLOWER) && self.once(id, "excitement") {
            self.grant_agile(id, 1);
        }
        if matches!(p.kind, HUNGER_PRIEST | TEMPTING_DANCER | INVITING_FOLLOWER)
            && self.once(id, "hunger")
        {
            let ids = self.nearby(
                id,
                1 - p.owner,
                if p.kind == TEMPTING_DANCER { 1 } else { 2 },
                true,
            );
            self.choose("허기를 악화시킬 적", "hunger", ids, 1);
        }
        let kings = self
            .living(p.owner)
            .into_iter()
            .filter_map(|id| self.located(id))
            .filter(|(_, p)| matches!(p.kind, SMILING_KING | TOAST_KING | ECSTATIC_KING))
            .collect::<Vec<_>>();
        for (pos, king) in kings {
            if distance(pos, at)
                <= match king.kind {
                    SMILING_KING => 2,
                    TOAST_KING => 3,
                    _ => 8,
                }
                && self.once(king.id, "king")
            {
                if p.owner == PLAYER {
                    self.mechanics.draws += 1;
                    if king.kind == ECSTATIC_KING {
                        self.mechanics.mana += 1;
                    }
                }
                if king.kind != SMILING_KING {
                    self.grant_agile(id, 1);
                }
            }
        }
        if p.owner == PLAYER {
            if self.mechanics.relics.contains(&Relic::SilverCutlery) && self.once(0, "silver") {
                self.mechanics.draws += 1;
            }
            if self.mechanics.relics.contains(&Relic::EmptyGoblet)
                && self.mechanics.turn_used.insert("goblet".into())
            {
                self.mechanics.mana += 1;
            }
            if self.mechanics.relics.contains(&Relic::GoldenTray)
                && self.mechanics.turn_used.insert("tray".into())
            {
                let ids = self.nearby(id, PLAYER, 1, false);
                self.choose("금박 쟁반: 민첩을 받을 아군", "agile", ids, 1);
            }
            if self.mechanics.relics.contains(&Relic::LaughingMask) && self.once(0, "mask") {
                let ids = self.living(ENEMY);
                self.choose("웃는 가면: 허기를 악화시킬 적", "hunger", ids, 1);
            }
            if self.mechanics.relics.contains(&Relic::EternalBell)
                && self.mechanics.corpse_count[0] == 3
            {
                self.mechanics.draws += 2;
                self.mechanics.mana += 1;
            }
            if self.mechanics.frenzy && !self.is_royal(id) {
                self.grant_agile(id, 1);
            }
        }
        Ok(())
    }
    pub fn corpses(&self) -> Vec<PieceId> {
        self.board
            .cells()
            .iter()
            .flatten()
            .filter(|p| p.kind == CORPSE)
            .map(|p| p.id)
            .collect()
    }
    pub fn choose_effect(
        &mut self,
        target: Option<PieceId>,
        destination: Option<usize>,
    ) -> Result<(), Error> {
        let choice = self
            .mechanics
            .choices
            .first()
            .cloned()
            .ok_or(Error::IllegalAction)?;
        if choice.kind == "move_corpse" {
            let c = destination
                .filter(|c| choice.cells.contains(c))
                .ok_or(Error::IllegalAction)?;
            let id = choice.targets[0];
            let (from, p) = self.located(id).ok_or(Error::IllegalAction)?;
            self.board.set_piece(from, None)?;
            self.board
                .set_piece(Position::new((c % 8) as i32, (c / 8) as i32), Some(p))?;
        } else {
            let id = target
                .filter(|id| choice.targets.contains(id))
                .ok_or(Error::IllegalAction)?;
            match choice.kind.as_str() {
                "agile" => self.grant_agile(id, choice.amount),
                "hunger" => self.worsen_hunger(id),
                "corpse" => {
                    self.queue_corpse_move(id)?;
                }
                _ => return Err(Error::IllegalAction),
            }
        }
        self.mechanics.choices.remove(0);
        if self.actions_left == 0
            && self.mechanics.choices.is_empty()
            && self.phase == Phase::Active
        {
            self.end_turn()?;
        }
        Ok(())
    }
    pub fn queue_corpse_move(&mut self, id: PieceId) -> Result<(), Error> {
        let (at, p) = self.located(id).ok_or(Error::IllegalAction)?;
        if p.kind != CORPSE {
            return Err(Error::IllegalAction);
        }
        let cells = [(1, 0), (-1, 0), (0, 1), (0, -1)]
            .into_iter()
            .filter_map(|(x, y)| {
                let to = Position::new(at.file + x, at.rank + y);
                self.board
                    .piece_at(to)
                    .ok()
                    .filter(|p| p.is_none())
                    .map(|_| cell(to))
            })
            .collect::<Vec<_>>();
        if !cells.is_empty() {
            self.mechanics.choices.push(ChoiceView {
                prompt: "시체를 옮길 빈 칸".into(),
                kind: "move_corpse".into(),
                targets: vec![id],
                cells,
                amount: 0,
            });
        }
        Ok(())
    }
    pub(super) fn finish_mechanics_turn(&mut self) -> Result<(), Error> {
        if !self.mechanics.active {
            return Ok(());
        }
        if self.turn == PLAYER && self.mechanics.last_stand && !self.mechanics.killed {
            self.kill_direct(self.king_id)?;
            self.update_result(ResultReason::RoyalCaptured);
        }
        for id in self.living(self.turn) {
            self.status_mut(id).agile = 0;
        }
        self.mechanics.turn_used.clear();
        self.mechanics.frenzy = false;
        self.mechanics.last_stand = false;
        self.mechanics.killed = false;
        if self.turn == ENEMY
            && self
                .board
                .cells()
                .iter()
                .flatten()
                .any(|p| p.kind == ETERNAL_GUEST)
        {
            self.mechanics.cycles = self.mechanics.cycles.saturating_add(1);
            match self.mechanics.cycles {
                2 | 5 => {
                    let ids = self
                        .living(PLAYER)
                        .into_iter()
                        .filter(|id| !self.is_royal(*id))
                        .collect::<Vec<_>>();
                    let max = ids
                        .iter()
                        .map(|id| self.status(*id).hunger)
                        .max()
                        .unwrap_or(0);
                    let ids = ids
                        .into_iter()
                        .filter(|id| self.status(*id).hunger == max)
                        .collect::<Vec<_>>();
                    if !ids.is_empty() {
                        self.mechanics.bell_target = Some(ids[self.mechanics.rng.index(ids.len())]);
                        self.events.push(
                            "만찬의 종소리: 다음 수에 표시된 기물의 허기가 악화됩니다.".into(),
                        );
                    }
                }
                3 | 6 => {
                    if let Some(id) = self.mechanics.bell_target.take() {
                        if self.located(id).is_some() {
                            self.worsen_hunger(id);
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
    pub(super) fn start_mechanics_turn(&mut self) -> Result<(), Error> {
        if !self.mechanics.active {
            return Ok(());
        }
        for id in self.living(self.turn) {
            let n = self.status(id).weak;
            self.status_mut(id).weak = 0;
            for _ in 0..n {
                let Some((at, mut p)) = self.located(id) else {
                    break;
                };
                if let Some(kind) = previous_kind(p.kind) {
                    p.kind = kind;
                    self.board.set_piece(at, Some(p))?;
                    if let Some(permanent) = self.players.iter_mut().find(|p| p.id == id) {
                        permanent.kind = kind;
                        permanent.upgrades.pop();
                    }
                } else {
                    self.kill_direct(id)?;
                    break;
                }
            }
        }
        self.sync_players();
        self.update_result(ResultReason::RoyalCaptured);
        self.refresh_actions();
        Ok(())
    }
    pub fn abilities(&self, id: PieceId) -> Vec<AbilityView> {
        let Some((at, p)) = self.located(id) else {
            return vec![];
        };
        if self.phase != Phase::Active
            || p.owner != self.turn
            || self.status(id).disabled
            || p.kind == CORPSE
            || (self.normal_actions_used >= self.base_actions(p.owner)
                && self.status(id).agile == 0)
        {
            return vec![];
        }
        // Hunger limits abilities whenever a corpse is legally available.
        if self.status(id).hunger >= 2
            && self
                .actions_from(at, false)
                .unwrap_or_default()
                .iter()
                .any(|a| {
                    a.steps().any(|s| {
                        self.board
                            .piece_at(s.take)
                            .ok()
                            .flatten()
                            .is_some_and(|p| p.kind == CORPSE)
                    })
                })
        {
            return vec![];
        }
        if matches!(
            p.kind,
            MASKED_LADY | RED_LADY | BANQUET_LADY | INVITING_LADY
        ) && !self.mechanics.used.contains(&(id, "sacrifice".into()))
        {
            let targets = self
                .nearby(id, p.owner, 1, true)
                .into_iter()
                .filter(|id| {
                    self.status(*id).ingredient
                        && self.located(*id).is_some_and(|(_, v)| {
                            v.kind != WAREHOUSE && (p.kind != MASKED_LADY || is_follower(v.kind))
                        })
                })
                .collect::<Vec<_>>();
            return vec![AbilityView {
                id,
                name: "제물",
                chances: vec![100; targets.len()],
                targets,
            }];
        }
        if !matches!(p.kind, PISTOL_INFANTRY | RIFLEMAN | MORTAR | DRONE | TANK) {
            return vec![];
        }
        if p.owner == PLAYER && self.mechanics.ammo_mode && self.mechanics.ammo == 0 {
            return vec![];
        }
        let mut targets = vec![];
        let mut chances = vec![];
        for other in self.living(1 - p.owner) {
            if self.is_royal(other) && self.located(other).is_none_or(|(_, p)| p.kind != WAREHOUSE)
            {
                continue;
            }
            let (to, _) = self.located(other).unwrap();
            let dx = to.file - at.file;
            let dy = to.rank - at.rank;
            let forward = if p.owner == PLAYER { -1 } else { 1 };
            let direction = if matches!(p.kind, PISTOL_INFANTRY | RIFLEMAN) {
                dy * forward > 0 && (dx == 0 || dx.abs() == dy.abs())
            } else {
                dx == 0 || dy == 0 || dx.abs() == dy.abs()
            };
            if !direction {
                continue;
            }
            let d = distance(at, to);
            if (1..d as i32).any(|i| {
                self.board
                    .piece_at(Position::new(
                        at.file + dx.signum() * i,
                        at.rank + dy.signum() * i,
                    ))
                    .ok()
                    .flatten()
                    .is_some()
            }) {
                continue;
            }
            targets.push(other);
            chances.push(accuracy(100, 1, 5, d));
        }
        vec![AbilityView {
            id,
            name: "사격",
            targets,
            chances,
        }]
    }
    pub fn use_ability(&mut self, id: PieceId, target: PieceId) -> Result<(), Error> {
        let ability = self
            .abilities(id)
            .into_iter()
            .find(|a| a.targets.contains(&target))
            .ok_or(Error::IllegalAction)?;
        let (_, p) = self.located(id).ok_or(Error::IllegalAction)?;
        if ability.name == "제물" {
            self.make_corpse(target)?;
            self.once(id, "sacrifice");
            if p.kind == BANQUET_LADY && p.owner == PLAYER {
                self.mechanics.draws += 1;
            }
            if p.kind == INVITING_LADY {
                let ids = self.nearby(id, 1 - p.owner, 2, true);
                self.choose("초대의 귀부인: 허기를 악화시킬 적", "hunger", ids, 1);
            }
        } else {
            let chance =
                ability.chances[ability.targets.iter().position(|v| *v == target).unwrap()];
            if p.owner == PLAYER && self.mechanics.ammo_mode {
                self.mechanics.ammo -= 1;
            }
            let hit = self.mechanics.rng.index(100) < chance as usize;
            self.events.push(format!(
                "사격: 명중률 {chance}% · {}",
                if hit { "명중" } else { "빗나감" }
            ));
            if hit {
                self.kill_nonroyal(target, true)?;
            }
        }
        self.spend_action(p)?;
        self.sync_players();
        self.update_result(ResultReason::RoyalCaptured);
        Ok(())
    }
    pub fn kill_nonroyal(&mut self, id: PieceId, leave_corpse: bool) -> Result<(), Error> {
        if self.is_royal(id) && self.located(id).is_none_or(|(_, p)| p.kind != WAREHOUSE) {
            return Err(Error::ProtectedRoyal);
        }
        if let Some((_, p)) = self.located(id) {
            if leave_corpse && p.kind != CORPSE && p.kind != WAREHOUSE && self.status(id).ingredient
            {
                self.make_corpse(id)?;
            } else {
                self.kill_direct(id)?;
            }
            if p.owner == ENEMY && p.kind != CORPSE {
                self.mechanics.killed = true;
            }
        }
        self.sync_players();
        self.update_result(ResultReason::RoyalCaptured);
        Ok(())
    }
}
