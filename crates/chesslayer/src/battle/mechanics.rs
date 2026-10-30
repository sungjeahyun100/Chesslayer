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
    ReservePawn,
    Revenge(PieceId),
    LastDefense(PieceId),
    ExtraMove,
    ExtraMoveDestination(PieceId),
}
#[derive(Clone)]
struct PendingChoice {
    view: EffectChoice,
    effect: ChoiceEffect,
}

#[derive(Clone, Default, Debug, Hash, Serialize)]
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
    pub(super) rng: Rng,
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
    fallout: [u32; 64],
    bell_target: Option<PieceId>,
    finished: bool,
    initial_allies: usize,
    last_stand: bool,
    pub(super) killed_this_turn: bool,
    endless: bool,
    ammo: u32,
    choices: VecDeque<PendingChoice>,
    pub(super) search_shot_outcome: Option<bool>,
    search_random_index: Option<usize>,
    search_random_arity: Option<usize>,
    temporary_pieces: HashSet<PieceId>,
    mobilization_left: u8,
    revenge_ready: bool,
    relief_ready: bool,
    human_wave_left: u8,
    wave_movers: HashSet<PieceId>,
    last_defense_ready: bool,
    pub(super) reckless: HashSet<PieceId>,
    aimed: HashMap<PieceId, u8>,
    propellant: HashMap<PieceId, u8>,
    fire_data_sharing: bool,
    shared_target: Option<PieceId>,
    entrenched_pending: HashSet<PieceId>,
    entrenched_ready: HashSet<PieceId>,
    pub(super) suppressed: HashSet<PieceId>,
    shot_counts: HashMap<PieceId, u32>,
    shot_hits_this_turn: u8,
    regular_losses: u8,
    revived_once: bool,
    pub(super) turn_moved: HashSet<PieceId>,
}
impl Mechanics {
    pub(super) fn search_hash(&self, h: &mut impl Hasher) {
        self.rng.hash(h);
        self.ammo.hash(h);
        self.enemy_actions.hash(h);
        self.rounds.hash(h);
        self.fallout.hash(h);
        self.bell_target.hash(h);
        self.player_artillery.hash(h);
        self.corpse_eaten.hash(h);
        self.pending_draw.hash(h);
        self.pending_mana.hash(h);
        self.pending_ammo.hash(h);
        self.killed_this_turn.hash(h);
        self.endless.hash(h);
        self.last_stand.hash(h);
        self.relics.hash(h);
        self.mobilization_left.hash(h);
        self.revenge_ready.hash(h);
        self.relief_ready.hash(h);
        self.human_wave_left.hash(h);
        self.last_defense_ready.hash(h);
        self.fire_data_sharing.hash(h);
        self.shared_target.hash(h);
        self.shot_hits_this_turn.hash(h);
        self.regular_losses.hash(h);
        self.revived_once.hash(h);
        let mut moved: Vec<_> = self.turn_moved.iter().collect();
        moved.sort();
        moved.hash(h);
        for set in [
            &self.temporary_pieces,
            &self.reckless,
            &self.entrenched_pending,
            &self.entrenched_ready,
            &self.suppressed,
            &self.wave_movers,
        ] {
            let mut ids: Vec<_> = set.iter().collect();
            ids.sort();
            ids.hash(h);
        }
        for map in [&self.aimed, &self.propellant] {
            let mut values: Vec<_> = map.iter().collect();
            values.sort_by_key(|(id, _)| **id);
            values.hash(h);
        }
        let mut counts: Vec<_> = self.shot_counts.iter().collect();
        counts.sort_by_key(|(id, _)| **id);
        counts.hash(h);
        let mut statuses: Vec<_> = self.statuses.iter().collect();
        statuses.sort_by_key(|(id, _)| **id);
        for (id, status) in statuses {
            id.hash(h);
            status.hash(h);
        }
        let mut used: Vec<_> = self.used.iter().collect();
        used.sort();
        used.hash(h);
        let mut turn_used: Vec<_> = self.turn_used.iter().collect();
        turn_used.sort();
        turn_used.hash(h);
    }
    pub(super) fn force_search_shot(&mut self, hit: bool) {
        self.search_shot_outcome = Some(hit);
    }
    pub(super) fn force_search_random(&mut self, index: usize) {
        self.search_random_index = Some(index);
        self.search_random_arity = None;
    }
    pub(super) fn take_search_random_arity(&mut self) -> Option<usize> {
        self.search_random_index = None;
        self.search_random_arity.take()
    }
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
            fallout: [0; 64],
            bell_target: None,
            finished: false,
            initial_allies: 0,
            last_stand: false,
            killed_this_turn: false,
            endless: false,
            ammo: 400,
            choices: VecDeque::new(),
            search_shot_outcome: None,
            search_random_index: None,
            search_random_arity: None,
            temporary_pieces: HashSet::new(),
            mobilization_left: 0,
            revenge_ready: false,
            relief_ready: false,
            human_wave_left: 0,
            wave_movers: HashSet::new(),
            last_defense_ready: false,
            reckless: HashSet::new(),
            aimed: HashMap::new(),
            propellant: HashMap::new(),
            fire_data_sharing: false,
            shared_target: None,
            entrenched_pending: HashSet::new(),
            entrenched_ready: HashSet::new(),
            suppressed: HashSet::new(),
            shot_counts: HashMap::new(),
            shot_hits_this_turn: 0,
            regular_losses: 0,
            revived_once: false,
            turn_moved: HashSet::new(),
        }
    }
}
fn key(kind: PieceType) -> &'static str {
    pieces::definition(kind).map_or("", |d| d.key)
}
fn distance(a: Position, b: Position) -> i32 {
    (a.file - b.file).abs().max((a.rank - b.rank).abs())
}
#[derive(Clone, Copy)]
enum BlastShape {
    Cross,
    King,
    Nuclear,
}
const FALLOUT_ROUNDS: u32 = 2;
fn rotate45(direction: (i32, i32), steps: i32) -> (i32, i32) {
    const DIRECTIONS: [(i32, i32); 8] = [
        (0, -1),
        (1, -1),
        (1, 0),
        (1, 1),
        (0, 1),
        (-1, 1),
        (-1, 0),
        (-1, -1),
    ];
    let index = DIRECTIONS
        .iter()
        .position(|&d| d == direction)
        .expect("shot direction");
    DIRECTIONS[(index as i32 + steps).rem_euclid(8) as usize]
}
fn penetration_chance(initial: u8, decay: u8, successful_hits: u8) -> u8 {
    initial.saturating_sub(decay.saturating_mul(successful_hits))
}
#[cfg(test)]
#[test]
fn pistol_and_rifle_accuracy_keep_existing_ranges() {
    let from = Position::new(4, 0);
    assert_eq!(
        shot_preview(pieces::PISTOL_INFANTRY, ENEMY, from, Position::new(4, 1)),
        Some(100)
    );
    assert_eq!(
        shot_preview(pieces::PISTOL_INFANTRY, ENEMY, from, Position::new(4, 2)),
        Some(90)
    );
    assert_eq!(
        shot_preview(pieces::RIFLEMAN, ENEMY, from, Position::new(4, 1)),
        Some(95)
    );
    assert_eq!(
        shot_preview(pieces::RIFLEMAN, ENEMY, from, Position::new(4, 6)),
        Some(70)
    );
    assert_eq!(
        shot_preview(pieces::RIFLEMAN, ENEMY, from, Position::new(4, 7)),
        None
    );
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
        "pistol_infantry" => Some((100, 10, 7)),
        "rifleman" => Some((100, 5, 6)),
        "mortar" => Some((80, 0, 5)),
        "machine_gun_drone" | "grenade_launcher_drone" => Some((90, 0, 4)),
        "tank" | "he_tank" | "ap_tank" | "fragment_tank" | "double_barrel" => Some((90, 0, 7)),
        "sniper" => Some((95, 3, 7)),
        "marksman" => Some((100, 2, 7)),
        "commando" => Some((90, 5, 4)),
        "elite_commando" => Some((100, 5, 4)),
        "special_commander" | "shadow_commander" => Some((95, 4, 5)),
        "tribal_dictator" => Some((100, 25, 3)),
        _ => None,
    }
}
/// Geometry and base accuracy for a shot on an otherwise empty board.
pub(crate) fn shot_preview(
    kind: PieceType,
    owner: PlayerId,
    from: Position,
    to: Position,
) -> Option<u8> {
    let k = key(kind);
    let (base, falloff, range) = shot_profile(k)?;
    let dx = to.file - from.file;
    let dy = to.rank - from.rank;
    let d = distance(from, to);
    if d == 0 || d > range {
        return None;
    }
    let forward = if owner == PLAYER { -1 } else { 1 };
    let aligned = if k == "mortar" {
        d >= 2
    } else if matches!(
        k,
        "machine_gun_drone"
            | "grenade_launcher_drone"
            | "tank"
            | "he_tank"
            | "ap_tank"
            | "fragment_tank"
    ) {
        dx == 0 || dy == 0
    } else if k == "double_barrel" {
        return None;
    } else if matches!(
        k,
        "pistol_infantry" | "rifleman" | "commando" | "elite_commando"
    ) {
        dy.signum() == forward && (dx == 0 || dx.abs() == dy.abs())
    } else {
        dx == 0 || dy == 0 || dx.abs() == dy.abs()
    };
    // Rifle's listed base is the zero-distance value; other weapon profiles
    // retain their existing adjacent-square baseline.
    let falloff_steps = if k == "rifleman" { d } else { d - 1 };
    aligned.then(|| (i32::from(base) - i32::from(falloff) * falloff_steps).clamp(0, 100) as u8)
}
impl Battle {
    fn blast_cells(&self, at: Position, shape: BlastShape) -> Vec<Position> {
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
    fn explode(
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
    /// A reusable penetration path. Empty cells do not consume probability;
    /// the path stops at its first failed hit.
    fn penetrate_paths(
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
    fn penetrate(
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
    pub(super) fn sandbox_ally_added(&mut self) {
        self.mechanics.initial_allies = self.mechanics.initial_allies.saturating_add(1);
    }
    pub(super) fn clear_fallout(&mut self) {
        self.mechanics.fallout = [0; 64];
    }
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
        if relics.contains(&Relic::RecruitmentOffice) {
            if let Some(cell) = self.friendly_empty_cells(true).first().copied() {
                let cell = cell - 10000;
                let _ =
                    self.spawn_reserve_pawn(Position::new((cell % 8) as i32, (cell / 8) as i32));
            }
        }
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
    pub fn sandbox_set_relic(&mut self, relic: Relic, enabled: bool) -> Result<(), String> {
        if !self.sandbox {
            return Err("테스트장에서만 유물을 설정할 수 있습니다.".into());
        }
        let owned = self.mechanics.relics.contains(&relic);
        if enabled && !owned {
            self.mechanics.relics.push(relic);
            if relic == Relic::RecruitmentOffice {
                if let Some(cell) = self.friendly_empty_cells(true).first().copied() {
                    let cell = cell - 10000;
                    self.spawn_reserve_pawn(Position::new((cell % 8) as i32, (cell / 8) as i32))?;
                }
            }
        } else if !enabled {
            self.mechanics.relics.retain(|r| *r != relic);
        }
        Ok(())
    }
    pub fn sandbox_relics(&self) -> &[Relic] {
        &self.mechanics.relics
    }
    pub fn relic_status(&self, relic: Relic) -> Option<String> {
        match relic {
            Relic::RecruitmentOffice => Some(format!(
                "임시 폰 {}기 생존",
                self.mechanics
                    .temporary_pieces
                    .iter()
                    .filter(|id| self.piece_position(**id).is_some())
                    .count()
            )),
            Relic::WarBonds => Some(format!(
                "정규 아군 포획 누적 {}회",
                self.mechanics.regular_losses
            )),
            Relic::PopulationMobilization => Some(
                if self.mechanics.revived_once {
                    "이번 전투 발동 완료"
                } else {
                    "첫 부활 대기 중"
                }
                .into(),
            ),
            Relic::BallisticComputer => Some(format!(
                "사격한 기물 {}기",
                self.mechanics.shot_counts.len()
            )),
            Relic::AmmunitionLine => Some(format!(
                "이번 턴 명중 {}회",
                self.mechanics.shot_hits_this_turn
            )),
            Relic::GunMount => Some(format!(
                "이번 턴 이동한 아군 {}기",
                self.mechanics.turn_moved.len()
            )),
            _ => None,
        }
    }
    pub fn set_ammunition(&mut self, amount: u32) {
        self.mechanics.ammo = amount;
    }
    pub(crate) fn pending_ammo_spend(&self) -> u32 {
        self.mechanics.pending_ammo.min(0).unsigned_abs()
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
    pub fn is_royal_piece(&self, id: PieceId) -> bool {
        if self.sandbox {
            return self.piece_position(id).is_some_and(|(_, piece)| {
                pieces::is_royal(piece.kind) || key(piece.kind) == "warehouse"
            });
        }
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
    fn friendly_empty_cells(&self, rear_only: bool) -> Vec<PieceId> {
        self.board
            .cells()
            .iter()
            .enumerate()
            .filter(|(i, p)| p.is_none() && *i / 8 >= if rear_only { 6 } else { 4 })
            .map(|(i, _)| 10000 + i as PieceId)
            .collect()
    }
    fn spawn_reserve_pawn(&mut self, at: Position) -> Result<(), String> {
        if at.rank < 4
            || self
                .board
                .piece_at(at)
                .map_err(|e| format!("{e:?}"))?
                .is_some()
        {
            return Err("아군 진영의 빈칸을 선택하세요.".into());
        }
        let id = self
            .players
            .iter()
            .map(|p| p.id)
            .chain(self.board.cells().iter().flatten().map(|p| p.id))
            .max()
            .unwrap_or(100)
            .max(100)
            + 1;
        self.board
            .set_piece(
                at,
                Some(Piece {
                    id,
                    kind: PAWN,
                    owner: PLAYER,
                }),
            )
            .map_err(|e| format!("{e:?}"))?;
        self.mechanics.temporary_pieces.insert(id);
        self.events.push(format!("예비 폰 배치: {}.", square(at)));
        Ok(())
    }
    fn counterattack_candidates(&self, enemy_id: PieceId, shooting: bool) -> Vec<PieceId> {
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
    fn free_counterattack(
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
        self.events.push(format!("{} 반격.", piece_name(p.kind)));
        Ok(())
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
        if p.kind == pieces::TRIBAL_DICTATOR {
            let mut actions = self.boss_special_actions(from);
            actions.extend(self.boss_copied_special_actions(from));
            return actions;
        }
        let k = key(p.kind);
        let s = self.status(p.id);
        let mut out = vec![];
        if matches!(
            p.kind,
            pieces::DRONE
                | pieces::C4_DRONE
                | pieces::MACHINE_GUN_DRONE
                | pieces::TACTICAL_NUKE_DRONE
                | pieces::GRENADE_LAUNCHER_DRONE
        ) {
            out.push(DefaultAction::one(ActionStep::new(
                from,
                from,
                from,
                Effect::Catch,
            )));
        }
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
                        || !matches!(
                            k,
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
                        || self.mechanics.ammo
                            >= if matches!(
                                k,
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
                            });
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
        let ids: Vec<_> = self.living(self.turn).iter().map(|(_, p)| p.id).collect();
        for id in ids {
            self.mechanics.statuses.entry(id).or_default().agility = 0;
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
            let n = self.mechanics.rounds;
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
                        let draw = self.mechanics.rng.index(targets.len());
                        let selected = self.mechanics.search_random_index.take().unwrap_or(draw)
                            % targets.len();
                        self.mechanics.search_random_arity = Some(targets.len());
                        let (_, p) = targets[selected];
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
        if self.boss_copied_action(action)? {
            return Ok(true);
        }
        if self.boss_action(action)? {
            return Ok(true);
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
                    let roll = self.mechanics.rng.index(100);
                    let hit = self
                        .mechanics
                        .search_shot_outcome
                        .take()
                        .unwrap_or(roll < usize::from(chance));
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
                } else if !self.boss_hit(victim, Some(captor.id)) {
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
    pub(super) fn record_kill(&mut self, captor: Piece, victim: Piece) {
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
    fn make_corpse(&mut self, at: Position) -> Result<(), Error> {
        let p = self.board.piece_at(at)?.ok_or(Error::IllegalAction)?;
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
    pub(super) fn kill_at(&mut self, at: Position, ingredient: bool) -> Result<(), Error> {
        self.kill_at_by(at, ingredient, None).map(|_| ())
    }
    pub(super) fn kill_at_by(
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
                ChoiceEffect::ReservePawn => {
                    let cell = target.checked_sub(10000).ok_or("빈 칸을 선택하세요.")?;
                    let at = Position::new((cell % 8) as i32, (cell / 8) as i32);
                    self.spawn_reserve_pawn(at)?;
                }
                ChoiceEffect::Revenge(enemy_id) => {
                    self.free_counterattack(target, enemy_id, false)?;
                }
                ChoiceEffect::LastDefense(enemy_id) => {
                    self.free_counterattack(target, enemy_id, true)?;
                }
                ChoiceEffect::ExtraMove => {
                    let (from, _) = self.piece_position(target).ok_or("기물이 없습니다.")?;
                    let destinations: Vec<_> = self
                        .actions_from(from, false)
                        .map_err(|e| format!("{e:?}"))?
                        .into_iter()
                        .filter(|a| a.first.move_to != from)
                        .filter_map(|a| {
                            a.steps()
                                .last()
                                .and_then(|s| self.board.index(s.move_to))
                                .map(|i| 10000 + i as u32)
                        })
                        .collect();
                    self.enqueue(
                        "인해전술: 이동할 칸 선택",
                        destinations,
                        ChoiceEffect::ExtraMoveDestination(target),
                        false,
                    );
                }
                ChoiceEffect::ExtraMoveDestination(id) => {
                    let (from, _) = self.piece_position(id).ok_or("기물이 없습니다.")?;
                    let cell = target.checked_sub(10000).ok_or("이동할 칸을 선택하세요.")?;
                    let at = Position::new((cell % 8) as i32, (cell / 8) as i32);
                    let action = self
                        .actions_from(from, false)
                        .map_err(|e| format!("{e:?}"))?
                        .into_iter()
                        .find(|a| {
                            a.steps()
                                .last()
                                .is_some_and(|s| s.move_to == at && s.move_to != from)
                        })
                        .ok_or("합법적인 이동을 선택하세요.")?;
                    self.mechanics.wave_movers.insert(id);
                    self.apply_effect(&action, ActionSource::PieceMove)
                        .map_err(|e| format!("{e:?}"))?;
                    self.moved_ids.insert(id);
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
                Sacrifice => ally && nonroyal && !self.mechanics.temporary_pieces.contains(&p.id),
                BackstageSacrifice => {
                    ally && nonroyal
                        && pieces::is_pawn(p.kind)
                        && !self.mechanics.temporary_pieces.contains(&p.id)
                }
                BodySlam => (ally || enemy) && nonroyal,
                Masquerade | IntermissionWaltz | Infiltration | Extraction => ally && nonroyal,
                Frontline => ally,
                DeliveredSupper | LastSupper => ally || corpse,
                ClearTablecloth => corpse,
                AimedShot => {
                    (enemy && nonroyal)
                        || (ally && self.can_act(*p) && shot_profile(key(p.kind)).is_some())
                }
                EmergencyConscription => false,
                AssaultOrder => {
                    ally && self.board.cells().iter().enumerate().any(|(i, q)| {
                        q.is_none()
                            && i / 8 + 1
                                == self
                                    .piece_position(p.id)
                                    .map_or(0, |(at, _)| at.rank as usize)
                            && i % 8
                                == self
                                    .piece_position(p.id)
                                    .map_or(9, |(at, _)| at.file as usize)
                    })
                }
                Expendables => ally && nonroyal && !self.mechanics.temporary_pieces.contains(&p.id),
                RecklessCharge => {
                    ally && self.piece_position(p.id).is_some_and(|(from, _)| {
                        self.piece_actions(p.kind, from)
                            .ok()
                            .is_some_and(|actions| {
                                actions.iter().any(|a| {
                                    if !a.continuation.is_empty() || a.first.effect != Effect::Move
                                    {
                                        return false;
                                    }
                                    let one = a.first.move_to;
                                    let dx = one.file - from.file;
                                    let dy = one.rank - from.rank;
                                    let to = Position::new(one.file + dx, one.rank + dy);
                                    dx.abs().max(dy.abs()) == 1
                                        && self.board.contains(to)
                                        && self.board.piece_at(one).ok().flatten().is_none()
                                        && self.board.piece_at(to).ok().flatten().is_none()
                                })
                            })
                    })
                }
                AimCorrection | RapidReload => ally && shot_profile(key(p.kind)).is_some(),
                EntrenchedDefense => {
                    ally && shot_profile(key(p.kind)).is_some_and(|(_, _, range)| range < 7)
                }
                ExtraPropellant => {
                    ally && matches!(p.kind, pieces::AP_TANK | pieces::FRAGMENT_TANK)
                        && self.piece_position(p.id).is_some_and(|(from, _)| {
                            self.living(ENEMY).into_iter().any(|(to, _)| {
                                self.hit_chance(from, to).is_some()
                                    && self
                                        .shot_effect_preview(p.kind, from, to)
                                        .1
                                        .iter()
                                        .any(|path| !path.is_empty())
                            })
                        })
                }
                SuppressionFire => enemy,
                FocusFire => (ally && shot_profile(key(p.kind)).is_some()) || enemy,
                _ => false,
            };
            if yes {
                out.push(p.id);
            }
        }
        if kind == EmergencyConscription {
            let dead: Vec<_> = self
                .players
                .iter()
                .filter(|p| {
                    p.position.is_none()
                        && !self.dead_at_start.contains(&p.id)
                        && pieces::is_pawn(p.kind)
                })
                .map(|p| p.id)
                .collect();
            if !self.friendly_empty_cells(false).is_empty() {
                out.extend(dead);
            }
        }
        if kind == RapidReload {
            out.extend(self.living(ENEMY).into_iter().map(|(_, p)| p.id));
        }
        if matches!(
            kind,
            InfantrySupport
                | ClearTablecloth
                | Frontline
                | Infiltration
                | Extraction
                | EmergencyConscription
        ) {
            for (i, p) in self.board.cells().iter().enumerate() {
                if p.is_none()
                    && (!matches!(kind, InfantrySupport | Extraction) || i / 8 >= 6)
                    && (kind != EmergencyConscription || (i / 8 >= 4 && !out.is_empty()))
                {
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
            EmergencyConscription => {
                if ids.len() != 2 {
                    return Err("사망한 폰과 아군 진영 빈칸을 선택하세요.".into());
                }
                let dead_id = *ids
                    .iter()
                    .find(|id| **id < 10000)
                    .ok_or("사망한 폰을 선택하세요.")?;
                let cell = *ids
                    .iter()
                    .find(|id| **id >= 10000)
                    .ok_or("빈칸을 선택하세요.")?
                    - 10000;
                let at = Position::new((cell % 8) as i32, (cell / 8) as i32);
                let pawn = self
                    .players
                    .iter()
                    .find(|p| p.id == dead_id && p.position.is_none() && pieces::is_pawn(p.kind))
                    .ok_or("이번 전투에서 사망한 폰을 선택하세요.")?
                    .kind;
                self.board
                    .set_piece(
                        at,
                        Some(Piece {
                            id: dead_id,
                            kind: pawn,
                            owner: PLAYER,
                        }),
                    )
                    .map_err(|e| format!("{e:?}"))?;
                self.sync_players();
                if !self.mechanics.revived_once {
                    self.mechanics.revived_once = true;
                    if self
                        .mechanics
                        .relics
                        .contains(&Relic::PopulationMobilization)
                    {
                        let cells = self.friendly_empty_cells(false);
                        self.enqueue(
                            "인구 동원령: 예비 폰을 배치할 칸 선택",
                            cells,
                            ChoiceEffect::ReservePawn,
                            false,
                        );
                    }
                }
            }
            GeneralMobilization => {
                if self.mechanics.mobilization_left > 0 {
                    return Err("총동원령이 이미 유지되고 있습니다.".into());
                }
                self.mechanics.mobilization_left = 3;
            }
            ComradeRevenge => {
                if self.mechanics.revenge_ready {
                    return Err("전우의 복수가 이미 대기 중입니다.".into());
                }
                self.mechanics.revenge_ready = true;
            }
            ReliefForce => {
                if self.mechanics.relief_ready {
                    return Err("교대 병력이 이미 대기 중입니다.".into());
                }
                self.mechanics.relief_ready = true;
            }
            HumanWave => {
                if self.mechanics.human_wave_left > 0 {
                    return Err("인해전술이 이미 유지되고 있습니다.".into());
                }
                self.mechanics.human_wave_left = 2;
                self.mechanics.wave_movers.clear();
            }
            LastDefense => {
                if self.mechanics.last_defense_ready {
                    return Err("최후의 방어선이 이미 대기 중입니다.".into());
                }
                self.mechanics.last_defense_ready = true;
            }
            FireDataSharing => {
                if self.mechanics.fire_data_sharing {
                    return Err("사격 제원 공유가 이미 유지되고 있습니다.".into());
                }
                self.mechanics.fire_data_sharing = true;
                self.mechanics.shared_target = None;
            }
            AssaultOrder => {
                if ids.is_empty() || ids.len() > 3 {
                    return Err("이동할 아군 1~3기를 선택하세요.".into());
                }
                for id in ids {
                    let (from, p) = self.piece_position(id).ok_or("기물이 없습니다.")?;
                    let to = Position::new(from.file, from.rank - 1);
                    if !self.board.contains(to)
                        || self
                            .board
                            .piece_at(to)
                            .map_err(|e| format!("{e:?}"))?
                            .is_some()
                    {
                        return Err("전방 빈칸으로 이동할 수 없습니다.".into());
                    }
                    self.board
                        .set_piece(from, None)
                        .map_err(|e| format!("{e:?}"))?;
                    self.board
                        .set_piece(to, Some(p))
                        .map_err(|e| format!("{e:?}"))?;
                    self.mechanics.turn_moved.insert(id);
                    self.moved_ids.insert(id);
                }
            }
            Expendables => {
                let id = one()?;
                let (at, _) = self.piece_position(id).ok_or("기물이 없습니다.")?;
                self.kill_at(at, false).map_err(err)?;
                self.mechanics.pending_draw += 3;
                removed = Some(1);
            }
            RecklessCharge => {
                if !self.mechanics.reckless.insert(one()?) {
                    return Err("이미 무모한 돌격이 적용된 기물입니다.".into());
                }
            }
            AimCorrection => {
                let id = one()?;
                let value = self.mechanics.aimed.entry(id).or_default();
                *value = value.saturating_add(25).min(100);
            }
            ExtraPropellant => {
                let id = one()?;
                let value = self.mechanics.propellant.entry(id).or_default();
                *value = value.saturating_add(20).min(100);
            }
            EntrenchedDefense => {
                let id = one()?;
                if self.mechanics.turn_moved.contains(&id) {
                    return Err("이번 턴 이동하지 않은 사수를 선택하세요.".into());
                }
                if !self.mechanics.entrenched_pending.insert(id) {
                    return Err("이미 진지를 유지 중인 사수입니다.".into());
                }
            }
            SuppressionFire => {
                if !self.mechanics.suppressed.insert(one()?) {
                    return Err("이미 이동이 제압된 적입니다.".into());
                }
            }
            RapidReload => {
                if ids.len() != 2 {
                    return Err("사수와 적을 한 기씩 선택하세요.".into());
                }
                let shooter = *ids
                    .iter()
                    .find(|id| {
                        self.piece_position(**id)
                            .is_some_and(|(_, p)| p.owner == PLAYER)
                    })
                    .ok_or("사수를 선택하세요.")?;
                let victim = *ids
                    .iter()
                    .find(|id| {
                        self.piece_position(**id)
                            .is_some_and(|(_, p)| p.owner == ENEMY)
                    })
                    .ok_or("적을 선택하세요.")?;
                self.free_counterattack(shooter, victim, true)?;
            }
            FocusFire => {
                if ids.len() != 3 {
                    return Err("사수 두 기와 적 하나를 선택하세요.".into());
                }
                let shooters: Vec<_> = ids
                    .iter()
                    .copied()
                    .filter(|id| {
                        self.piece_position(*id)
                            .is_some_and(|(_, p)| p.owner == PLAYER)
                    })
                    .collect();
                let victim = *ids
                    .iter()
                    .find(|id| {
                        self.piece_position(**id)
                            .is_some_and(|(_, p)| p.owner == ENEMY)
                    })
                    .ok_or("적을 선택하세요.")?;
                if shooters.len() != 2
                    || shooters
                        .iter()
                        .any(|id| !self.counterattack_candidates(victim, true).contains(id))
                {
                    return Err("같은 적을 사격할 수 있는 사수 두 기를 선택하세요.".into());
                }
                self.free_counterattack(shooters[0], victim, true)?;
                if self
                    .piece_position(victim)
                    .is_some_and(|(_, p)| p.owner == ENEMY)
                    && self.piece_position(shooters[1]).is_some()
                {
                    self.free_counterattack(shooters[1], victim, true)?;
                }
            }
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
                self.mechanics.temporary_pieces.insert(next_id);
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
                self.kill_at(corpse.0, false).map_err(err)?;
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
                    self.kill_at(*pos, false).map_err(err)?;
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
                let killed = self.kill_at_by(to, true, Some(p.id)).map_err(err)?;
                if killed {
                    self.record_kill(p, q);
                }
                removed = Some(usize::from(killed));
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

#[cfg(test)]
mod ordnance_tests {
    use super::*;
    fn p(file: i32, rank: i32) -> Position {
        Position::new(file, rank)
    }

    #[test]
    fn drone_tree_keeps_movement_and_nuclear_blast_has_seventeen_cells() {
        for kind in [
            pieces::C4_DRONE,
            pieces::MACHINE_GUN_DRONE,
            pieces::TACTICAL_NUKE_DRONE,
            pieces::GRENADE_LAUNCHER_DRONE,
        ] {
            assert_eq!(
                (pieces::definition(kind).unwrap().movement)(),
                (pieces::definition(pieces::DRONE).unwrap().movement)()
            );
        }
        let battle = Battle::sandbox().unwrap();
        assert_eq!(battle.blast_preview(pieces::DRONE, p(3, 3)).len(), 5);
        assert_eq!(battle.blast_preview(pieces::C4_DRONE, p(3, 3)).len(), 9);
        assert_eq!(
            battle
                .blast_preview(pieces::TACTICAL_NUKE_DRONE, p(3, 3))
                .len(),
            17
        );
        assert_eq!(
            battle
                .blast_preview(pieces::TACTICAL_NUKE_DRONE, p(0, 0))
                .len(),
            6
        );
    }

    #[test]
    fn self_destruct_removes_both_sides_and_simultaneous_royals_favor_enemy() {
        let mut players = Battle::standard_roster();
        players.retain(|piece| piece.id == 1 || piece.id == 5);
        players[0].kind = pieces::TACTICAL_NUKE_DRONE;
        players[0].position = Some(p(3, 4));
        players[1].position = Some(p(4, 4));
        let mut battle = Battle::new(BattleStart {
            players,
            encounter: Encounter {
                enemies: vec![EnemyPiece {
                    id: 100,
                    kind: KING,
                    position: p(3, 3),
                }],
                royal_id: 100,
                additional_royal_ids: vec![],
            },
        })
        .unwrap();
        battle.use_drone_ability(p(3, 4)).unwrap();
        assert!(battle.board.piece_at(p(3, 4)).unwrap().is_none());
        assert!(battle.board.piece_at(p(4, 4)).unwrap().is_none());
        assert!(battle.board.piece_at(p(3, 3)).unwrap().is_none());
        assert!(matches!(
            battle.result,
            GameResult::Win { winner: ENEMY, .. }
        ));
        assert_eq!(battle.fallout_cells().len(), 17);
        battle.reset_after_battle().unwrap();
        assert!(battle.fallout_cells().is_empty());
    }

    #[test]
    fn fallout_damages_on_own_turn_end_and_expires_after_two_rounds() {
        let mut battle = Battle::sandbox().unwrap();
        battle
            .sandbox_place(1, pieces::TACTICAL_NUKE_DRONE, PLAYER, p(3, 3))
            .unwrap();
        battle.sandbox_drone_ability(p(3, 3)).unwrap();
        assert_eq!(battle.fallout_remaining(p(3, 3)), 2);
        battle
            .sandbox_place(2, pieces::PAWN, PLAYER, p(3, 3))
            .unwrap();
        battle.sandbox_progress_effects().unwrap();
        assert!(battle.board.piece_at(p(3, 3)).unwrap().is_none());
        assert_eq!(battle.fallout_remaining(p(3, 3)), 1);
        battle.sandbox_progress_effects().unwrap();
        assert_eq!(battle.fallout_remaining(p(3, 3)), 0);
        battle
            .sandbox_place(3, pieces::PAWN, PLAYER, p(3, 3))
            .unwrap();
        battle.sandbox_progress_effects().unwrap();
        assert!(battle.board.piece_at(p(3, 3)).unwrap().is_some());
    }

    #[test]
    fn high_explosive_shot_hits_allies_and_enemies_without_extra_rolls() {
        let mut battle = Battle::sandbox().unwrap();
        battle
            .sandbox_place(1, pieces::HE_TANK, PLAYER, p(3, 4))
            .unwrap();
        battle
            .sandbox_place(2, pieces::PAWN, ENEMY, p(3, 2))
            .unwrap();
        battle
            .sandbox_place(3, pieces::PAWN, PLAYER, p(4, 2))
            .unwrap();
        battle
            .sandbox_place(4, pieces::PAWN, ENEMY, p(2, 2))
            .unwrap();
        battle.force_search_shot(true);
        let action = battle
            .sandbox_actions(p(3, 4))
            .unwrap()
            .into_iter()
            .find(|a| a.first.effect == Effect::Catch && a.first.take == p(3, 2))
            .unwrap();
        battle.sandbox_apply_action(&action).unwrap();
        for pos in [p(3, 2), p(4, 2), p(2, 2)] {
            assert!(battle.board.piece_at(pos).unwrap().is_none());
        }
        assert!(battle.board.piece_at(p(3, 4)).unwrap().is_some());
    }

    #[test]
    fn missed_high_explosive_shot_does_not_explode() {
        let mut battle = Battle::sandbox().unwrap();
        battle
            .sandbox_place(1, pieces::HE_TANK, PLAYER, p(3, 4))
            .unwrap();
        battle
            .sandbox_place(2, pieces::PAWN, ENEMY, p(3, 2))
            .unwrap();
        battle
            .sandbox_place(3, pieces::PAWN, PLAYER, p(4, 2))
            .unwrap();
        battle.force_search_shot(false);
        let action = battle
            .sandbox_actions(p(3, 4))
            .unwrap()
            .into_iter()
            .find(|a| a.first.effect == Effect::Catch && a.first.take == p(3, 2))
            .unwrap();
        battle.sandbox_apply_action(&action).unwrap();
        assert!(battle.board.piece_at(p(3, 2)).unwrap().is_some());
        assert!(battle.board.piece_at(p(4, 2)).unwrap().is_some());
    }

    #[test]
    fn grenade_launcher_and_double_barrel_share_the_hit_blast() {
        for (kind, target, neighbor) in [
            (pieces::GRENADE_LAUNCHER_DRONE, p(3, 2), p(4, 2)),
            (pieces::DOUBLE_BARREL, p(4, 2), p(5, 2)),
        ] {
            let mut battle = Battle::sandbox().unwrap();
            battle.sandbox_place(1, kind, PLAYER, p(3, 4)).unwrap();
            battle
                .sandbox_place(2, pieces::PAWN, ENEMY, target)
                .unwrap();
            battle
                .sandbox_place(3, pieces::PAWN, PLAYER, neighbor)
                .unwrap();
            battle.force_search_shot(true);
            let action = battle
                .sandbox_actions(p(3, 4))
                .unwrap()
                .into_iter()
                .find(|a| a.first.effect == Effect::Catch && a.first.take == target)
                .unwrap();
            battle.sandbox_apply_action(&action).unwrap();
            assert!(
                battle.board.piece_at(target).unwrap().is_none(),
                "kind {kind}"
            );
            assert!(
                battle.board.piece_at(neighbor).unwrap().is_none(),
                "kind {kind}"
            );
        }
    }

    #[test]
    fn double_barrel_shoots_a_tempest_rook_destination() {
        let mut battle = Battle::sandbox().unwrap();
        battle
            .sandbox_place(1, pieces::DOUBLE_BARREL, PLAYER, p(3, 4))
            .unwrap();
        battle
            .sandbox_place(2, pieces::PAWN, ENEMY, p(4, 2))
            .unwrap();
        assert_eq!(battle.hit_chance(p(3, 4), p(4, 2)), Some(90));
        battle.sandbox_change_kind(p(3, 4), pieces::TANK).unwrap();
        assert_eq!(battle.hit_chance(p(3, 4), p(4, 2)), None);

        let mut edge_battle = Battle::sandbox().unwrap();
        edge_battle
            .sandbox_place(1, pieces::DOUBLE_BARREL, PLAYER, p(3, 4))
            .unwrap();
        edge_battle
            .sandbox_place(2, pieces::PAWN, ENEMY, p(4, 0))
            .unwrap();
        assert_eq!(edge_battle.hit_chance(p(3, 4), p(4, 0)), Some(90));
    }

    #[test]
    fn tanks_can_shoot_pieces_on_the_far_board_edge() {
        for kind in [
            pieces::TANK,
            pieces::HE_TANK,
            pieces::AP_TANK,
            pieces::FRAGMENT_TANK,
        ] {
            for (from, to) in [(p(3, 7), p(3, 0)), (p(0, 4), p(7, 4)), (p(3, 4), p(3, 0))] {
                let mut battle = Battle::sandbox().unwrap();
                battle.sandbox_place(1, kind, PLAYER, from).unwrap();
                battle.sandbox_place(2, pieces::PAWN, ENEMY, to).unwrap();
                assert_eq!(battle.hit_chance(from, to), Some(90), "kind {kind}");
                assert!(
                    battle.sandbox_actions(from).unwrap().iter().any(|action| {
                        action.first.effect == Effect::Catch && action.first.take == to
                    }),
                    "kind {kind}"
                );
            }
        }
    }

    #[test]
    fn tank_shot_to_board_edge_still_stops_at_an_intervening_piece() {
        let mut battle = Battle::sandbox().unwrap();
        battle
            .sandbox_place(1, pieces::TANK, PLAYER, p(3, 7))
            .unwrap();
        battle
            .sandbox_place(2, pieces::PAWN, PLAYER, p(3, 4))
            .unwrap();
        battle
            .sandbox_place(3, pieces::PAWN, ENEMY, p(3, 0))
            .unwrap();
        assert_eq!(battle.hit_chance(p(3, 7), p(3, 0)), None);
    }

    #[test]
    fn penetration_counts_hits_not_empty_cells_and_fragments_rotate_relative_to_shot() {
        assert_eq!(
            (0..5)
                .map(|n| penetration_chance(80, 20, n))
                .collect::<Vec<_>>(),
            [80, 60, 40, 20, 0]
        );
        assert_eq!(
            (0..4)
                .map(|n| penetration_chance(75, 25, n))
                .collect::<Vec<_>>(),
            [75, 50, 25, 0]
        );
        let mut battle = Battle::sandbox().unwrap();
        battle
            .sandbox_place(1, pieces::AP_TANK, PLAYER, p(3, 6))
            .unwrap();
        battle
            .sandbox_place(2, pieces::PAWN, ENEMY, p(3, 1))
            .unwrap();
        let shooter = battle.board.piece_at(p(3, 6)).unwrap().unwrap();
        battle.force_search_shot(true);
        battle
            .penetrate(p(3, 4), (0, -1), 80, 20, None, shooter)
            .unwrap();
        assert!(battle.board.piece_at(p(3, 1)).unwrap().is_none());
        let (_, paths) = battle.shot_effect_preview(pieces::FRAGMENT_TANK, p(3, 6), p(3, 4));
        assert_eq!(
            paths.iter().map(|path| path[0]).collect::<Vec<_>>(),
            [p(3, 3), p(2, 3), p(4, 3)]
        );
    }

    #[test]
    fn artillery_boss_enemy_drone_can_choose_self_destruct() {
        let index = crate::content::encounters::ENCOUNTERS
            .iter()
            .position(|entry| entry.id == "3F-B01")
            .unwrap();
        let mut battle = Battle::new(BattleStart {
            players: Battle::standard_roster(),
            encounter: crate::content::encounters::encounter(index),
        })
        .unwrap();
        battle.turn = ENEMY;
        let (from, _) = battle
            .living(ENEMY)
            .into_iter()
            .find(|(_, piece)| piece.kind == pieces::DRONE)
            .unwrap();
        assert!(battle.legal_actions(from).unwrap().iter().any(|action| {
            action.first.effect == Effect::Catch
                && action.first.take == from
                && action.first.move_to == from
        }));
    }
}
