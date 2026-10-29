//! v0.3 battle rules. Movement programs remain independently defined by each piece.
use super::*;
use crate::content::{
    cards::{CardKind, CardSelection},
    Relic,
};
use crate::{ActionStep, Effect};
use serde::Serialize;
use std::collections::VecDeque;

#[derive(Clone, Debug, Serialize)]
pub struct EffectChoice {
    pub name: String,
    pub targets: Vec<PieceId>,
    pub can_finish: bool,
}
#[derive(Clone)]
enum ChoiceEffect {
    Agility(u8),
    Hunger,
    Corpse,
    Slide(PieceId),
    Formation,
    Relocate(PieceId),
}
#[derive(Clone)]
struct PendingChoice {
    view: EffectChoice,
    effect: ChoiceEffect,
}

#[derive(Clone, Default, Debug, Serialize)]
pub struct PieceStatus {
    pub ingredient: bool,
    pub blessing: bool,
    pub hunger: u8,
    pub madness: u8,
    pub guilt: u8,
    pub incapacitated: bool,
    pub agility: u8,
    pub weakness: u8,
    pub dining: bool,
}
#[derive(Clone)]
pub(super) struct Mechanics {
    pub statuses: HashMap<PieceId, PieceStatus>,
    rng: Rng,
    relics: Vec<Relic>,
    used: HashSet<String>,
    turn_used: HashSet<String>,
    corpse_eaten: [u32; 2],
    pending_draw: u32,
    pending_mana: i32,
    pending_ammo: i32,
    pub player_artillery: bool,
    pub(super) enemy_actions: u8,
    rounds: u32,
    bell_target: Option<PieceId>,
    finished: bool,
    initial_allies: usize,
    last_stand: bool,
    pub(super) killed_this_turn: bool,
    endless: bool,
    ammo: u32,
    choices: VecDeque<PendingChoice>,
}
impl Mechanics {
    pub(super) fn new(rng: Rng) -> Self {
        Self {
            statuses: HashMap::new(),
            rng,
            relics: vec![],
            used: HashSet::new(),
            turn_used: HashSet::new(),
            corpse_eaten: [0, 0],
            pending_draw: 0,
            pending_mana: 0,
            pending_ammo: 0,
            player_artillery: false,
            enemy_actions: 1,
            rounds: 0,
            bell_target: None,
            finished: false,
            initial_allies: 0,
            last_stand: false,
            killed_this_turn: false,
            endless: false,
            ammo: 400,
            choices: VecDeque::new(),
        }
    }
}
fn key(kind: PieceType) -> &'static str {
    pieces::definition(kind).map_or("", |d| d.key)
}
fn distance(a: Position, b: Position) -> i32 {
    (a.file - b.file).abs().max((a.rank - b.rank).abs())
}
fn is_cart(k: &str) -> bool {
    matches!(
        k,
        "butcher_cart" | "funeral_cart" | "slaughter_cart" | "feast_cart"
    )
}
fn is_priest(k: &str) -> bool {
    matches!(
        k,
        "intoxicated_priest" | "feast_priest" | "locust_high_priest" | "hunger_priest"
    )
}
fn is_lady(k: &str) -> bool {
    matches!(
        k,
        "masked_lady" | "red_lady" | "banquet_lady" | "inviting_lady"
    )
}
fn shot_profile(k: &str) -> Option<(u8, u8, i32)> {
    match k {
        "pistol_infantry" => Some((100, 5, 7)),
        "rifleman" => Some((95, 5, 6)),
        "mortar" => Some((80, 0, 5)),
        "drone" => Some((90, 0, 3)),
        "tank" => Some((90, 0, 6)),
        "sniper" => Some((95, 3, 7)),
        "marksman" => Some((100, 2, 7)),
        "commando" => Some((90, 5, 4)),
        "elite_commando" => Some((100, 5, 4)),
        "special_commander" | "shadow_commander" => Some((95, 4, 5)),
        _ => None,
    }
}
impl Battle {
    pub(super) fn initialize_statuses(&mut self) {
        self.mechanics.statuses.clear();
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
                    incapacitated: get("incapacitated") > 0,
                    blessing: get("blessing") > 0,
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
    pub fn configure_v030(&mut self, relics: &[Relic]) {
        self.mechanics.relics = relics.to_vec();
        let boss = self
            .board
            .cells()
            .iter()
            .flatten()
            .any(|p| key(p.kind) == "eternal_guest");
        let enemy_carnival = boss
            || self.board.cells().iter().flatten().any(|p| {
                p.owner == ENEMY
                    && matches!(key(p.kind), "smiling_king" | "toast_lord" | "ecstatic_lord")
            });
        let carnival = relics.contains(&Relic::CarnivalFlare);
        let enemy_vanguard = self.board.cells().iter().flatten().any(|p| {
            p.owner == ENEMY && matches!(key(p.kind), "vanguard_commander" | "mounted_commander")
        });
        self.mechanics.enemy_actions = if enemy_vanguard { 2 } else { 1 };
        for p in self.board.cells().iter().flatten().copied() {
            if p.kind == pieces::CORPSE {
                continue;
            }
            let st = self.mechanics.statuses.entry(p.id).or_default();
            if carnival || enemy_carnival {
                st.ingredient = true;
            }
            if (carnival && p.owner == PLAYER) || (enemy_carnival && p.owner == ENEMY) {
                st.blessing = true;
            }
        }
        let pieces: Vec<_> = self.board.cells().iter().flatten().copied().collect();
        for p in pieces {
            match key(p.kind) {
                "scout" => self.grant_agility(p.id, 1),
                "phantom_scout" => self.grant_agility(p.id, 2),
                "special_commander" => {
                    self.wards.insert(p.id);
                }
                "shadow_commander" => {
                    self.wards.insert(p.id);
                    self.grant_agility(p.id, 1);
                }
                _ => {}
            }
        }
        if relics.contains(&Relic::TornMenu) {
            if let Some(id) = self
                .living(ENEMY)
                .into_iter()
                .find(|(_, p)| !self.is_royal_piece(p.id))
                .map(|(_, p)| p.id)
            {
                self.worsen_hunger(id);
            }
        }
    }
    pub fn set_ammunition(&mut self, amount: u32) {
        self.mechanics.ammo = amount;
    }
    pub fn take_resource_effects(&mut self) -> (u32, i32, i32) {
        (
            std::mem::take(&mut self.mechanics.pending_draw),
            std::mem::take(&mut self.mechanics.pending_mana),
            std::mem::take(&mut self.mechanics.pending_ammo),
        )
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
        if s.incapacitated {
            result.push("영구 전투불능".into());
        }
        if s.dining {
            result.push("식사의 예법".into());
        }
        if self.mechanics.bell_target == Some(id) {
            result.push("다음 수 만찬의 종소리".into());
        }
        result
    }
    pub fn is_royal_piece(&self, id: PieceId) -> bool {
        id == self.king_id
            || self.is_enemy_royal(id)
            || self
                .piece_position(id)
                .is_some_and(|(_, p)| key(p.kind) == "warehouse")
    }
    pub fn piece_position(&self, id: PieceId) -> Option<(Position, Piece)> {
        self.board.cells().iter().enumerate().find_map(|(i, p)| {
            p.filter(|p| p.id == id)
                .map(|p| (Position::new(i as i32 % 8, i as i32 / 8), p))
        })
    }
    fn living(&self, owner: PlayerId) -> Vec<(Position, Piece)> {
        self.board
            .cells()
            .iter()
            .enumerate()
            .filter_map(|(i, p)| {
                p.filter(|p| p.owner == owner && p.kind != pieces::CORPSE)
                    .map(|p| (Position::new(i as i32 % 8, i as i32 / 8), p))
            })
            .collect()
    }
    pub(super) fn side_actions(&self, owner: PlayerId) -> u8 {
        if owner == ENEMY {
            self.mechanics.enemy_actions
        } else {
            self.actions_per_turn
        }
    }
    pub(super) fn agility(&self, id: PieceId) -> u8 {
        self.mechanics.statuses.get(&id).map_or(0, |s| s.agility)
    }
    pub(super) fn can_act(&self, p: Piece) -> bool {
        p.kind != pieces::CORPSE && key(p.kind) != "warehouse" && !self.status(p.id).incapacitated
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
    fn worsen_hunger(&mut self, id: PieceId) {
        let s = self.mechanics.statuses.entry(id).or_default();
        s.hunger = s.hunger.saturating_add(1).min(3);
    }
    pub(super) fn filter_hunger(&self, p: Piece, actions: &mut Vec<DefaultAction>) {
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
    /// Special captures share the legal action pipeline; shooting stays on the origin square.
    pub fn special_actions(&self, from: Position) -> Vec<DefaultAction> {
        let Some(p) = self.board.piece_at(from).ok().flatten() else {
            return vec![];
        };
        if !self.can_act(p) {
            return vec![];
        }
        let k = key(p.kind);
        let s = self.status(p.id);
        let mut out = vec![];
        for (i, q) in self.board.cells().iter().enumerate() {
            let Some(q) = q else {
                continue;
            };
            let to = Position::new(i as i32 % 8, i as i32 / 8);
            let d = distance(from, to);
            if q.kind == pieces::CORPSE {
                let adjacent = d == 1 && s.dining;
                if adjacent {
                    out.push(DefaultAction::one(ActionStep::new(
                        from,
                        to,
                        to,
                        Effect::Take,
                    )));
                }
            } else {
                let sacrifice = q.owner == p.owner
                    && is_lady(k)
                    && d == 1
                    && !self.is_royal_piece(q.id)
                    && (k != "masked_lady" || pieces::is_pawn(q.kind))
                    && !self.mechanics.used.contains(&format!("lady:{}", p.id));
                let shooting = q.owner != p.owner
                    && q.owner < 2
                    && self.hit_chance(from, to).is_some()
                    && (p.owner != PLAYER
                        || !matches!(k, "rifleman" | "mortar" | "drone" | "tank")
                        || self.mechanics.ammo
                            >= if matches!(k, "mortar" | "tank") { 2 } else { 1 });
                if sacrifice || shooting {
                    out.push(DefaultAction::one(ActionStep::new(
                        from,
                        to,
                        from,
                        Effect::Catch,
                    )));
                }
            }
        }
        out
    }
    pub fn hit_chance(&self, from: Position, to: Position) -> Option<u8> {
        let p = self.board.piece_at(from).ok().flatten()?;
        let (base, falloff, range) = shot_profile(key(p.kind))?;
        let dx = to.file - from.file;
        let dy = to.rank - from.rank;
        let d = distance(from, to);
        if d == 0 || d > range {
            return None;
        }
        let forward = if p.owner == PLAYER { -1 } else { 1 };
        let k = key(p.kind);
        let aligned = if k == "mortar" {
            d >= 2
        } else if k == "drone" {
            true
        } else if k == "tank" {
            dx == 0 || dy == 0
        } else if matches!(
            k,
            "pistol_infantry" | "rifleman" | "commando" | "elite_commando"
        ) {
            dy.signum() == forward && (dx == 0 || dx.abs() == dy.abs())
        } else {
            dx == 0 || dy == 0 || dx.abs() == dy.abs()
        };
        if !aligned {
            return None;
        }
        if !matches!(k, "mortar" | "drone") {
            for n in 1..d {
                let at = Position::new(from.file + dx.signum() * n, from.rank + dy.signum() * n);
                if self.board.piece_at(at).ok().flatten().is_some() {
                    return None;
                }
            }
        }
        let bonus = if p.owner == PLAYER {
            [
                (Relic::FireControl, 10),
                (Relic::BallisticComputer, 5),
                (Relic::ReconScope, 15),
            ]
            .iter()
            .filter(|(r, _)| self.mechanics.relics.contains(r))
            .map(|(_, n)| n)
            .sum::<i32>()
        } else {
            0
        };
        Some((i32::from(base) - i32::from(falloff) * (d - 1) + bonus).clamp(0, 100) as u8)
    }
    pub(super) fn persistent_outcome(&self) -> Vec<PlayerPiece> {
        let mut out = self.players.clone();
        for p in &mut out {
            if let Some(s) = self.mechanics.statuses.get(&p.id) {
                for (k, n) in [
                    ("hunger", i32::from(s.hunger)),
                    ("madness", i32::from(s.madness)),
                    ("guilt", i32::from(s.guilt)),
                    ("incapacitated", i32::from(s.incapacitated)),
                    ("blessing", i32::from(s.blessing)),
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
    pub(super) fn finish_persistent_statuses(&mut self) {
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
    pub(super) fn finish_mechanics_turn(&mut self) -> Result<(), Error> {
        if self.turn == PLAYER && self.mechanics.last_stand && !self.mechanics.killed_this_turn {
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
        let ids: Vec<_> = self.living(self.turn).iter().map(|(_, p)| p.id).collect();
        for id in ids {
            self.mechanics.statuses.entry(id).or_default().agility = 0;
        }
        if self.turn == PLAYER {
            self.mechanics.endless = false;
            self.mechanics.last_stand = false;
            self.mechanics.killed_this_turn = false;
        }
        self.mechanics.turn_used.clear();
        if self.turn == ENEMY {
            self.mechanics.rounds += 1;
            let n = self.mechanics.rounds;
            if n % 3 == 0 {
                if let Some((pos, p)) = self
                    .living(ENEMY)
                    .into_iter()
                    .find(|(_, p)| key(p.kind) == "tribal_dictator")
                {
                    if let Some(id) = self.ally_near(pos, ENEMY, p.id, 8) {
                        self.grant_agility(id, 1);
                        self.events
                            .push("부족의 독재자: 가장 가까운 부하에 민첩 1.".into());
                    }
                }
            }
            if self
                .board
                .cells()
                .iter()
                .flatten()
                .any(|p| key(p.kind) == "eternal_guest")
            {
                if matches!(n, 2 | 5) {
                    let mut targets = self.living(PLAYER);
                    targets.retain(|(_, p)| !self.is_royal_piece(p.id));
                    let highest = targets
                        .iter()
                        .map(|(_, p)| self.status(p.id).hunger)
                        .max()
                        .unwrap_or(0);
                    targets.retain(|(_, p)| self.status(p.id).hunger == highest);
                    if !targets.is_empty() {
                        let (_, p) = targets[self.mechanics.rng.index(targets.len())];
                        self.mechanics.bell_target = Some(p.id);
                        self.events.push(format!(
                            "만찬의 종소리: 다음 수 {}의 허기가 악화됩니다.",
                            piece_name(p.kind)
                        ));
                    }
                }
                if matches!(n, 3 | 6) {
                    if let Some(id) = self.mechanics.bell_target.take() {
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
    pub(super) fn start_mechanics_turn(&mut self) -> Result<(), Error> {
        let targets = self.living(self.turn);
        for (pos, p) in targets {
            let weakness = self.status(p.id).weakness;
            if weakness == 0 {
                continue;
            }
            self.mechanics.statuses.entry(p.id).or_default().weakness = 0;
            let mut kind = p.kind;
            let mut dead = false;
            for _ in 0..weakness {
                if let Some(parent) = pieces::ALL
                    .iter()
                    .find(|d| d.upgrades.iter().any(|u| u.target == kind) && d.kind != kind)
                {
                    kind = parent.kind;
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
    pub(super) fn apply_mechanics_action(&mut self, action: &DefaultAction) -> Result<bool, Error> {
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
            let victim = self.board.piece_at(step.take)?;
            let capture = !matches!(step.effect, Effect::Move | Effect::Shift) && victim.is_some();
            if capture {
                let victim = victim.unwrap();
                if step.effect == Effect::Catch
                    && victim.owner == captor.owner
                    && is_lady(key(captor.kind))
                {
                    self.make_corpse(step.take)?;
                    self.mechanics.used.insert(format!("lady:{}", captor.id));
                    if key(captor.kind) == "banquet_lady" && captor.owner == PLAYER {
                        self.mechanics.pending_draw += 1;
                    }
                    if key(captor.kind) == "inviting_lady" {
                        self.hunger_near(step.from, captor.owner, 2);
                    }
                    self.events.push(format!(
                        "{}: 아군을 제물로 삼아 시체를 남겼습니다.",
                        piece_name(captor.kind)
                    ));
                    break;
                }
                if step.effect == Effect::Catch && shot_profile(key(captor.kind)).is_some() {
                    let chance = self.hit_chance(step.from, step.take).unwrap_or(0);
                    if captor.owner == PLAYER
                        && matches!(key(captor.kind), "rifleman" | "mortar" | "drone" | "tank")
                    {
                        let cost = if matches!(key(captor.kind), "mortar" | "tank") {
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
                    if self.mechanics.rng.index(100) >= usize::from(chance) {
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
                    if self.wards.remove(&victim.id) {
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
                    self.eat_corpse(captor.id)?;
                } else if self.status(victim.id).ingredient && !self.is_royal_piece(victim.id) {
                    self.make_corpse(step.take)?;
                    self.record_kill(captor, victim);
                    // A live ingredient capture ends here. The new corpse needs its own action.
                    if victim.owner == PLAYER && self.self_defense_target == Some(victim.id) {
                        self.kill_at(step.from, false)?;
                    }
                    break;
                } else {
                    apply_default_action(&mut self.board, &DefaultAction::one(step.clone()))?;
                    self.record_kill(captor, victim);
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
    pub(super) fn record_kill(&mut self, captor: Piece, victim: Piece) {
        if captor.owner == PLAYER && victim.owner == ENEMY {
            self.mechanics.killed_this_turn = true;
            self.initiative_turns_left = 0;
        }
        if captor.owner == ENEMY && victim.owner == PLAYER {
            self.enemy_initiative_turns_left = 0;
        }
    }
    fn make_corpse(&mut self, at: Position) -> Result<(), Error> {
        let p = self.board.piece_at(at)?.ok_or(Error::IllegalAction)?;
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
    fn kill_at(&mut self, at: Position, ingredient: bool) -> Result<(), Error> {
        let Some(p) = self.board.piece_at(at)? else {
            return Ok(());
        };
        if ingredient
            && self.status(p.id).ingredient
            && p.kind != pieces::CORPSE
            && !self.is_royal_piece(p.id)
        {
            self.make_corpse(at)?;
        } else {
            self.board.set_piece(at, None)?;
        }
        if key(p.kind) == "warehouse" {
            self.warehouse_explosion(at, p.owner)?;
        }
        Ok(())
    }
    fn warehouse_explosion(&mut self, at: Position, owner: PlayerId) -> Result<(), Error> {
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
    fn hunger_near(&mut self, at: Position, owner: PlayerId, range: i32) {
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
    fn ally_near(
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
    fn eat_corpse(&mut self, id: PieceId) -> Result<(), Error> {
        let Some((at, p)) = self.piece_position(id) else {
            return Ok(());
        };
        let owner = p.owner;
        if owner > 1 {
            return Ok(());
        }
        self.mechanics.statuses.entry(id).or_default().hunger = 0;
        if self.status(id).blessing {
            self.grant_agility(id, 3);
            self.events
                .push(format!("{} 과흥분: 민첩 3.", piece_name(p.kind)));
        } else {
            if self.mechanics.rng.index(2) == 0 {
                self.mechanics.statuses.entry(id).or_default().madness += 1;
            } else {
                self.mechanics.statuses.entry(id).or_default().guilt += 1;
            }
            self.resolve_conscience(id)?;
        }
        self.mechanics.corpse_eaten[usize::from(owner)] += 1;
        let eaten = self.mechanics.corpse_eaten[usize::from(owner)];
        let k = key(p.kind);
        let first = self.mechanics.used.insert(format!("eat:{id}"));
        if first {
            if is_cart(k) && owner == PLAYER {
                self.mechanics.pending_draw += 1;
            }
            if matches!(k, "slaughter_cart" | "ecstatic_dancer" | "glutton_follower") {
                self.grant_agility(id, 1);
            }
            if is_priest(k) {
                let range = if k == "intoxicated_priest" { 1 } else { 2 };
                self.choose_ally(
                    at,
                    owner,
                    id,
                    range,
                    if k == "locust_high_priest" { 2 } else { 1 },
                    "사제: 민첩을 받을 아군 선택",
                );
            }
            if matches!(k, "hunger_priest" | "inviting_follower") {
                self.hunger_near(at, owner, 2);
            }
            if k == "tempting_dancer" {
                self.hunger_near(at, owner, 1);
            }
            if k == "feast_cart" {
                if owner == PLAYER {
                    let targets = self.corpse_targets(at, 2);
                    self.enqueue(
                        "연회의 수레: 이동할 시체 선택",
                        targets,
                        ChoiceEffect::Corpse,
                        false,
                    );
                } else {
                    self.slide_near_corpse(at, 2)?;
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
        if owner == ENEMY
            && eaten <= 3
            && self
                .living(ENEMY)
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
    fn resolve_conscience(&mut self, id: PieceId) -> Result<(), Error> {
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
                s.incapacitated = true;
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
        if self.status(id).incapacitated {
            let lost = self.agility(id);
            self.mechanics.statuses.entry(id).or_default().agility = 0;
            if self
                .piece_position(id)
                .is_some_and(|(_, p)| p.owner == self.turn)
            {
                self.actions_left = self.actions_left.saturating_sub(lost);
            }
            self.events
                .push("죄책감: 영구 전투불능. 휴식의 고해성사로만 회복할 수 있습니다.".into());
        }
        Ok(())
    }
    fn slide_near_corpse(&mut self, at: Position, range: i32) -> Result<(), Error> {
        let corpses: Vec<_> = self
            .board
            .cells()
            .iter()
            .enumerate()
            .filter_map(|(i, p)| {
                p.filter(|p| p.kind == pieces::CORPSE)
                    .map(|p| (Position::new(i as i32 % 8, i as i32 / 8), p))
            })
            .collect();
        for (pos, p) in corpses {
            if distance(at, pos) > range {
                continue;
            }
            for (dx, dy) in [(0, -1), (1, 0), (0, 1), (-1, 0)] {
                let to = Position::new(pos.file + dx, pos.rank + dy);
                if self.board.contains(to) && self.board.piece_at(to)?.is_none() {
                    self.board.set_piece(pos, None)?;
                    self.board.set_piece(to, Some(p))?;
                    return Ok(());
                }
            }
        }
        Ok(())
    }
    pub fn pending_choice(&self) -> Option<EffectChoice> {
        self.mechanics.choices.front().map(|c| c.view.clone())
    }
    fn enqueue(
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
    fn corpse_targets(&self, at: Position, range: i32) -> Vec<PieceId> {
        self.board
            .cells()
            .iter()
            .enumerate()
            .filter_map(|(i, p)| {
                p.filter(|p| {
                    p.kind == pieces::CORPSE
                        && distance(at, Position::new(i as i32 % 8, i as i32 / 8)) <= range
                })
            })
            .filter(|p| {
                self.piece_position(p.id)
                    .is_some_and(|(pos, _)| !self.empty_adjacent(pos).is_empty())
            })
            .map(|p| p.id)
            .collect()
    }
    fn empty_adjacent(&self, pos: Position) -> Vec<PieceId> {
        [(1, 0), (-1, 0), (0, 1), (0, -1)]
            .into_iter()
            .filter_map(|(dx, dy)| {
                let to = Position::new(pos.file + dx, pos.rank + dy);
                self.board
                    .index(to)
                    .filter(|_| self.board.piece_at(to).ok().flatten().is_none())
                    .map(|i| 10000 + i as u32)
            })
            .collect()
    }
    fn choose_ally(
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
    fn queue_formation(&mut self) {
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
                ChoiceEffect::Hunger => self.worsen_hunger(target),
                ChoiceEffect::Corpse => {
                    let (pos, _) = self.piece_position(target).ok_or("시체가 없습니다.")?;
                    let cells = self.empty_adjacent(pos);
                    self.enqueue(
                        "연회의 수레: 인접한 빈 칸 선택",
                        cells,
                        ChoiceEffect::Slide(target),
                        false,
                    );
                }
                ChoiceEffect::Slide(id) | ChoiceEffect::Relocate(id) => {
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
            }
        }
        self.sync_players();
        self.update_result(ResultReason::RoyalCaptured);
        if self.mechanics.choices.is_empty()
            && self.actions_left == 0
            && self.phase == Phase::Active
        {
            self.end_turn().map_err(|e| format!("{e:?}"))?;
        }
        Ok(())
    }
    pub fn v030_card_targets(&self, kind: CardKind) -> Vec<PieceId> {
        use CardKind::*;
        let mut out = vec![];
        for p in self.board.cells().iter().flatten() {
            let corpse = p.kind == pieces::CORPSE;
            let ally = p.owner == PLAYER && !corpse;
            let enemy = p.owner == ENEMY && !corpse;
            let nonroyal = !self.is_royal_piece(p.id);
            let yes = match kind {
                Stimpack | ExcitedSteps | DiningEtiquette | LateDinner | SmokeScreen => ally,
                BloodyInvitation | MealTime | TornMask => enemy,
                CrackedConscience => {
                    enemy && (self.status(p.id).madness > 0 || self.status(p.id).guilt > 0)
                }
                Sacrifice => ally && nonroyal,
                BackstageSacrifice => ally && nonroyal && pieces::is_pawn(p.kind),
                BodySlam => (ally || enemy) && nonroyal,
                Masquerade | IntermissionWaltz | Infiltration | Extraction => ally && nonroyal,
                Frontline => ally,
                DeliveredSupper | LastSupper => ally || corpse,
                ClearTablecloth => corpse,
                AimedShot => {
                    (enemy && nonroyal)
                        || (ally && self.can_act(*p) && shot_profile(key(p.kind)).is_some())
                }
                _ => false,
            };
            if yes {
                out.push(p.id);
            }
        }
        if matches!(
            kind,
            InfantrySupport | ClearTablecloth | Frontline | Infiltration | Extraction
        ) {
            for (i, p) in self.board.cells().iter().enumerate() {
                if p.is_none() && (!matches!(kind, InfantrySupport | Extraction) || i / 8 >= 6) {
                    out.push(10000 + i as u32);
                }
            }
        }
        out
    }
    pub fn v030_card_effect(
        &mut self,
        kind: CardKind,
        selection: &CardSelection,
    ) -> Result<Option<usize>, String> {
        use CardKind::*;
        if self.pending_choice().is_some() {
            return Err("먼저 발동한 능력의 대상을 선택하세요.".into());
        }
        if self.phase != Phase::Active || self.turn != PLAYER {
            return Err("플레이어 전투 턴에만 사용할 수 있습니다.".into());
        }
        let ids = if selection.targets.is_empty() {
            selection.target.into_iter().collect::<Vec<_>>()
        } else {
            selection.targets.clone()
        };
        let candidates = self.v030_card_targets(kind);
        if ids.iter().any(|id| !candidates.contains(id)) {
            return Err("카드 효과에 맞는 대상을 선택하세요.".into());
        }
        if ids.iter().copied().collect::<HashSet<_>>().len() != ids.len() {
            return Err("같은 대상을 중복 선택할 수 없습니다.".into());
        }
        let one = || -> Result<PieceId, String> {
            if ids.len() != 1 {
                Err("대상을 하나 선택하세요.".into())
            } else {
                Ok(ids[0])
            }
        };
        let err = |e: Error| format!("전투 효과 오류: {e:?}");
        let mut removed = None;
        match kind {
            Stimpack => {
                let id = one()?;
                self.grant_agility(id, 4);
                self.mechanics.statuses.entry(id).or_default().weakness += 1;
            }
            ExcitedSteps => {
                let id = one()?;
                self.grant_agility(id, 1);
            }
            DiningEtiquette => {
                let id = one()?;
                self.mechanics.statuses.entry(id).or_default().dining = true;
            }
            BloodyInvitation | MealTime => {
                let id = one()?;
                let hungry = self.status(id).hunger >= 2;
                self.worsen_hunger(id);
                if kind == MealTime && hungry {
                    self.mechanics.pending_draw += 1;
                }
            }
            LateDinner => {
                let id = one()?;
                let s = self.mechanics.statuses.entry(id).or_default();
                if s.hunger == 0 {
                    return Err("허기 상태의 아군을 선택하세요.".into());
                }
                s.hunger -= 1;
            }
            TornMask => {
                let id = one()?;
                self.mechanics.statuses.entry(id).or_default().blessing = false;
            }
            CrackedConscience => {
                let id = one()?;
                let s = self.mechanics.statuses.entry(id).or_default();
                if s.madness > 0 {
                    s.madness += 1;
                } else if s.guilt > 0 {
                    s.guilt += 1;
                } else {
                    return Err("광기 또는 죄책감이 있는 적을 선택하세요.".into());
                }
                self.resolve_conscience(id).map_err(err)?;
            }
            Sacrifice => {
                let id = one()?;
                let (pos, _) = self.piece_position(id).ok_or("대상 없음")?;
                self.kill_at(pos, true).map_err(err)?;
                self.mechanics.pending_mana += 3;
                removed = Some(1);
            }
            BackstageSacrifice => {
                let id = one()?;
                if !self.mechanics.used.insert("backstage".into()) {
                    return Err("무대 뒤의 희생은 전투당 한 번만 사용할 수 있습니다.".into());
                }
                let (pos, _) = self.piece_position(id).ok_or("대상 없음")?;
                self.make_corpse(pos).map_err(err)?;
                self.mechanics.pending_draw += 2;
                removed = Some(1);
            }
            InfantrySupport => {
                let id = one()?;
                let at = Position::new(((id - 10000) % 8) as i32, ((id - 10000) / 8) as i32);
                let next_id = self
                    .players
                    .iter()
                    .map(|p| p.id)
                    .chain(self.board.cells().iter().flatten().map(|p| p.id))
                    .max()
                    .unwrap_or(100)
                    + 1;
                let p = Piece {
                    id: next_id,
                    kind: pieces::AXE_INFANTRY,
                    owner: PLAYER,
                };
                self.board
                    .set_piece(at, Some(p))
                    .map_err(|e| format!("{e:?}"))?;
                if self.mechanics.relics.contains(&Relic::CarnivalFlare)
                    || self
                        .living(ENEMY)
                        .iter()
                        .any(|(_, p)| key(p.kind) == "eternal_guest")
                {
                    self.mechanics
                        .statuses
                        .entry(next_id)
                        .or_default()
                        .ingredient = true;
                }
            }
            BodySlam => {
                let allies: Vec<_> = ids
                    .iter()
                    .filter_map(|id| self.piece_position(*id).filter(|(_, p)| p.owner == PLAYER))
                    .collect();
                let enemies: Vec<_> = ids
                    .iter()
                    .filter_map(|id| self.piece_position(*id).filter(|(_, p)| p.owner == ENEMY))
                    .collect();
                if allies.is_empty() || allies.len() != enemies.len() {
                    return Err(
                        "희생할 아군과 제거할 적을 같은 수만큼 선택하세요. 로얄은 제외합니다."
                            .into(),
                    );
                }
                for (pos, _) in allies.iter().chain(enemies.iter()) {
                    self.kill_at(*pos, true).map_err(err)?;
                }
                self.mechanics.killed_this_turn = true;
                removed = Some(enemies.len());
            }
            LastStand => {
                let alive = self.living(PLAYER);
                if alive.len() * 2 > self.mechanics.initial_allies {
                    return Err("아군이 전투 시작 수의 절반 이하일 때 사용할 수 있습니다.".into());
                }
                for (pos, p) in alive {
                    self.grant_agility(p.id, 3);
                    if let Some(target) = pieces::definition(p.kind)
                        .and_then(|d| d.upgrades.first())
                        .map(|u| u.target)
                    {
                        self.board
                            .set_piece(pos, Some(Piece { kind: target, ..p }))
                            .map_err(|e| format!("{e:?}"))?;
                        if let Some(owned) = self.players.iter_mut().find(|q| q.id == p.id) {
                            owned.kind = target;
                            owned.upgrades.push(key(target).into());
                        }
                    }
                }
                self.mechanics.last_stand = true;
            }
            Masquerade | IntermissionWaltz => {
                if ids.len() != 2 {
                    return Err("아군 비로얄 기물 두 개를 선택하세요.".into());
                }
                let (a, pa) = self.piece_position(ids[0]).ok_or("대상 없음")?;
                let (b, pb) = self.piece_position(ids[1]).ok_or("대상 없음")?;
                if kind == IntermissionWaltz && distance(a, b) != 1 {
                    return Err("서로 인접한 아군 두 개를 선택하세요.".into());
                }
                self.board
                    .set_piece(a, Some(pb))
                    .map_err(|e| format!("{e:?}"))?;
                self.board
                    .set_piece(b, Some(pa))
                    .map_err(|e| format!("{e:?}"))?;
            }
            ClearTablecloth | Frontline | Infiltration | Extraction => {
                if ids.len() != 2 {
                    return Err("기물 하나와 빈 칸 하나를 선택하세요.".into());
                }
                let piece_id = *ids
                    .iter()
                    .find(|id| **id < 10000)
                    .ok_or("이동할 기물을 선택하세요.")?;
                let cell = *ids
                    .iter()
                    .find(|id| **id >= 10000)
                    .ok_or("빈 칸을 선택하세요.")?
                    - 10000;
                let to = Position::new((cell % 8) as i32, (cell / 8) as i32);
                let (from, p) = self.piece_position(piece_id).ok_or("대상 없음")?;
                if kind == ClearTablecloth {
                    if (from.file - to.file).abs() + (from.rank - to.rank).abs() != 1 {
                        return Err("시체와 상하좌우로 인접한 빈 칸을 선택하세요.".into());
                    }
                    if self.mechanics.turn_used.contains("tablecloth") {
                        return Err("식탁보 걷기는 턴당 한 번입니다.".into());
                    }
                    self.mechanics.turn_used.insert("tablecloth".into());
                }
                if kind == Infiltration && distance(from, to) > 2 {
                    return Err("2칸 이내의 빈 칸을 선택하세요.".into());
                }
                self.board
                    .set_piece(from, None)
                    .map_err(|e| format!("{e:?}"))?;
                self.board
                    .set_piece(to, Some(p))
                    .map_err(|e| format!("{e:?}"))?;
                if kind == Frontline {
                    self.queue_formation();
                }
            }
            DeliveredSupper => {
                if ids.len() != 2 {
                    return Err("아군과 2칸 이내의 시체를 하나씩 선택하세요.".into());
                }
                let ally = ids
                    .iter()
                    .filter_map(|id| self.piece_position(*id))
                    .find(|(_, p)| p.owner == PLAYER)
                    .ok_or("아군을 선택하세요.")?;
                let corpse = ids
                    .iter()
                    .filter_map(|id| self.piece_position(*id))
                    .find(|(_, p)| p.kind == pieces::CORPSE)
                    .ok_or("시체를 선택하세요.")?;
                if distance(ally.0, corpse.0) > 2 {
                    return Err("시체가 아군으로부터 2칸을 초과했습니다.".into());
                }
                self.board
                    .set_piece(corpse.0, None)
                    .map_err(|e| format!("{e:?}"))?;
                self.eat_corpse(ally.1.id).map_err(err)?;
            }
            LastSupper => {
                let corpses: Vec<_> = ids
                    .iter()
                    .filter_map(|id| self.piece_position(*id))
                    .filter(|(_, p)| p.kind == pieces::CORPSE)
                    .collect();
                let allies: Vec<_> = ids
                    .iter()
                    .filter_map(|id| self.piece_position(*id))
                    .filter(|(_, p)| p.owner == PLAYER)
                    .collect();
                if corpses.is_empty() || corpses.len() > 3 {
                    return Err("소비할 시체 1~3개와 민첩을 받을 아군을 선택하세요.".into());
                }
                let recipients = if allies.is_empty() {
                    self.living(PLAYER)
                } else {
                    allies
                };
                if recipients.is_empty() {
                    return Err("민첩을 받을 아군이 없습니다.".into());
                }
                for (i, (pos, _)) in corpses.iter().enumerate() {
                    self.board
                        .set_piece(*pos, None)
                        .map_err(|e| format!("{e:?}"))?;
                    self.grant_agility(recipients[i % recipients.len()].1.id, 1);
                }
            }
            EndlessEcstasy => {
                if !self.mechanics.used.insert("endless".into()) {
                    return Err("환희는 끝나지 않는다: 전투당 한 번입니다.".into());
                }
                self.mechanics.endless = true;
            }
            Volley => {
                let mut victims = HashSet::new();
                for (from, p) in self.living(PLAYER) {
                    if !self.can_act(p) {
                        continue;
                    }
                    for a in self.actions_from(from, false).map_err(err)? {
                        for s in a.steps() {
                            if !matches!(s.effect, Effect::Move | Effect::Shift) {
                                if let Some(q) =
                                    self.board.piece_at(s.take).map_err(|e| format!("{e:?}"))?
                                {
                                    if q.owner == ENEMY && !self.is_royal_piece(q.id) {
                                        victims.insert(q.id);
                                    }
                                }
                            }
                        }
                    }
                }
                if victims.is_empty() {
                    return Err("아군의 포획 범위 안에 적이 없습니다.".into());
                }
                for id in &victims {
                    if let Some((pos, _)) = self.piece_position(*id) {
                        self.kill_at(pos, true).map_err(err)?;
                    }
                }
                self.mechanics.killed_this_turn = true;
                removed = Some(victims.len());
            }
            SmokeScreen => {
                let id = one()?;
                self.grant_ward(id).map_err(err)?;
                self.grant_agility(id, 1);
            }
            AimedShot => {
                if ids.len() != 2 {
                    return Err("사격할 아군과 사선 안의 적을 선택하세요.".into());
                }
                let (from, p) = ids
                    .iter()
                    .filter_map(|id| self.piece_position(*id))
                    .find(|(_, p)| p.owner == PLAYER)
                    .ok_or("아군 사수를 선택하세요.")?;
                let (to, q) = ids
                    .iter()
                    .filter_map(|id| self.piece_position(*id))
                    .find(|(_, p)| p.owner == ENEMY)
                    .ok_or("적을 선택하세요.")?;
                if self.hit_chance(from, to).is_none() {
                    return Err("사거리와 사선을 확인하세요.".into());
                }
                self.kill_at(to, true).map_err(err)?;
                self.record_kill(p, q);
                removed = Some(1);
            }
            _ => return Err("이 카드는 기존 카드 효과를 사용합니다.".into()),
        }
        self.sync_players();
        self.update_result(ResultReason::RoyalCaptured);
        Ok(removed)
    }
}

#[cfg(test)]
mod smoke {
    use super::*;
    fn fixture(kind: PieceType) -> Battle {
        let mut roster = Battle::standard_roster();
        roster.retain(|p| p.id == 1 || p.id == 5);
        roster[0].kind = kind;
        roster[0].position = Some(Position::new(3, 4));
        Battle::new(BattleStart {
            players: roster,
            encounter: Encounter {
                enemies: vec![
                    EnemyPiece {
                        id: 101,
                        kind: PAWN,
                        position: Position::new(3, 3),
                    },
                    EnemyPiece {
                        id: 100,
                        kind: KING,
                        position: Position::new(7, 0),
                    },
                ],
                royal_id: 100,
                additional_royal_ids: vec![],
            },
        })
        .unwrap()
    }
    #[test]
    fn ingredient_capture_stays_then_corpse_capture_grants_extra_actions() {
        let mut b = fixture(ROOK);
        b.configure_v030(&[Relic::CarnivalFlare]);
        let from = Position::new(3, 4);
        let to = Position::new(3, 3);
        let a = b
            .legal_actions(from)
            .unwrap()
            .into_iter()
            .find(|a| a.first.move_to == to)
            .unwrap();
        b.apply_piece_action(&a).unwrap();
        assert_eq!(b.board.piece_at(from).unwrap().unwrap().id, 1);
        assert_eq!(b.board.piece_at(to).unwrap().unwrap().kind, pieces::CORPSE);
        assert_eq!(b.board.piece_at(to).unwrap().unwrap().owner, 2);
        b.end_turn().unwrap();
        b.mechanics.statuses.entry(1).or_default().hunger = 3;
        let actions = b.legal_actions(from).unwrap();
        assert!(actions.iter().all(|a| a.first.take == to));
        b.apply_piece_action(&actions[0]).unwrap();
        assert_eq!(b.board.piece_at(to).unwrap().unwrap().id, 1);
        assert_eq!(b.status(1).hunger, 0);
        assert_eq!(b.status(1).agility, 3);
        assert_eq!(b.actions_left, 3);
        assert_eq!(b.turn, PLAYER);
    }
    #[test]
    fn conscience_and_hunger_persist_once() {
        let mut b = fixture(ROOK);
        b.mechanics.statuses.entry(1).or_default().guilt = 2;
        b.resolve_conscience(1).unwrap();
        assert!(b.status(1).incapacitated);
        assert!(b.legal_actions(Position::new(3, 4)).unwrap().is_empty());
        b.mechanics.statuses.entry(1).or_default().hunger = 1;
        b.finish_persistent_statuses();
        b.finish_persistent_statuses();
        let out = b.outcome();
        let p = out.iter().find(|p| p.id == 1).unwrap();
        assert_eq!(p.persistent.get("hunger"), Some(&2));
        assert_eq!(p.persistent.get("incapacitated"), Some(&1));
        let mut b = fixture(ROOK);
        b.mechanics.statuses.entry(1).or_default().madness = 3;
        b.resolve_conscience(1).unwrap();
        assert_eq!(b.piece_position(1).unwrap().1.owner, ENEMY);
        assert!(b.status(1).blessing);
        assert_eq!(b.status(1).madness, 0);
    }
    #[test]
    fn shooting_requires_and_spends_ammunition_without_moving() {
        let mut b = fixture(pieces::RIFLEMAN);
        b.configure_v030(&[Relic::FireControl]);
        b.grant_ward(101).unwrap();
        let from = Position::new(3, 4);
        b.set_ammunition(0);
        assert!(b.special_actions(from).is_empty());
        b.set_ammunition(1);
        let shot = b
            .special_actions(from)
            .into_iter()
            .find(|a| a.first.effect == Effect::Catch && a.first.take == Position::new(3, 3))
            .unwrap();
        b.apply_piece_action(&shot).unwrap();
        assert_eq!(b.board.piece_at(from).unwrap().unwrap().id, 1);
        assert_eq!(b.take_resource_effects().2, -1);
        assert!(!b.has_ward(101));
        assert_eq!(
            b.board.piece_at(Position::new(3, 3)).unwrap().unwrap().kind,
            PAWN
        );
    }
}
