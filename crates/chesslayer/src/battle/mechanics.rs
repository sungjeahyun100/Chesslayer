//! v0.3 battle rules. Movement programs remain independently defined by each piece.
mod choices;
mod combat;
mod relics;
mod shooting;
mod statuses;
use super::*;
use crate::content::Relic;
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
pub(in crate::battle) enum ChoiceEffect {
    Agility(u8),
    MoveRights(u32),
    Hunger,
    RelieveHunger,
    Madness(u8),
    Remove,
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
    pub incapacitated: u8,
    pub incapacitated_fresh: bool,
    pub pest: u8,
    pub spirit_bomb: bool,
    pub corpse_bags: u8,
    pub agility: u8,
    pub weakness: u8,
    pub dining: bool,
    pub death_threshold: bool,
}
#[derive(Clone)]
pub(super) struct Mechanics {
    pub statuses: HashMap<PieceId, PieceStatus>,
    // Corpse rewards activate only when the recipient's side next starts a turn.
    pub(super) pending_agility: HashMap<PieceId, u8>,
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
    plague: [u32; 64],
    bell_targets: [Option<PieceId>; 2],
    finished: bool,
    initial_allies: usize,
    last_stand: bool,
    pub(super) killed_this_turn: bool,
    endless: bool,
    ammo: u32,
    choices: VecDeque<PendingChoice>,
    pub(super) search_shot_outcome: Option<bool>,
    search_boss_shots: Option<VecDeque<bool>>,
    search_boss_shot_chance: Option<u8>,
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
        self.plague.hash(h);
        self.bell_targets.hash(h);
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
        for map in [&self.aimed, &self.propellant, &self.pending_agility] {
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
    pub(super) fn force_search_boss_shots(&mut self, hits: Vec<bool>) {
        self.search_boss_shots = Some(hits.into());
        self.search_boss_shot_chance = None;
    }
    pub(super) fn take_search_boss_shot_chance(&mut self) -> Option<u8> {
        self.search_boss_shots = None;
        self.search_boss_shot_chance.take()
    }
    pub(super) fn boss_shot_hit(&mut self, chance: u8) -> bool {
        if let Some(hits) = &mut self.search_boss_shots {
            // Keep later engine randomness aligned with the live transition.
            // The search branches only replace this roll's hit/miss result.
            self.rng.index(100);
            return hits.pop_front().unwrap_or_else(|| {
                self.search_boss_shot_chance.get_or_insert(chance);
                false
            });
        }
        self.search_shot_outcome
            .take()
            .unwrap_or_else(|| self.rng.index(100) < chance as usize)
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
            pending_agility: HashMap::new(),
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
            plague: [0; 64],
            bell_targets: [None; 2],
            finished: false,
            initial_allies: 0,
            last_stand: false,
            killed_this_turn: false,
            endless: false,
            ammo: 400,
            choices: VecDeque::new(),
            search_shot_outcome: None,
            search_boss_shots: None,
            search_boss_shot_chance: None,
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
pub(in crate::battle) enum BlastShape {
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
        Some(95)
    );
    assert_eq!(
        shot_preview(pieces::PISTOL_INFANTRY, ENEMY, from, Position::new(4, 7)),
        Some(70)
    );
    for kind in [pieces::MACHINE_GUN_DRONE, pieces::GRENADE_LAUNCHER_DRONE] {
        assert_eq!(
            shot_preview(kind, ENEMY, from, Position::new(4, 1)),
            Some(90)
        );
        assert_eq!(
            shot_preview(kind, ENEMY, from, Position::new(4, 4)),
            Some(75)
        );
        assert_eq!(shot_preview(kind, ENEMY, from, Position::new(4, 5)), None);
    }
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
        "pistol_infantry" => Some((100, 5, 7)),
        "rifleman" => Some((100, 5, 6)),
        "mortar" => Some((80, 0, 5)),
        "machine_gun_drone" | "grenade_launcher_drone" => Some((90, 5, 4)),
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
    pub(super) fn sandbox_ally_added(&mut self) {
        self.mechanics.initial_allies = self.mechanics.initial_allies.saturating_add(1);
    }
    pub(super) fn clear_fallout(&mut self) {
        self.mechanics.fallout = [0; 64];
        self.mechanics.plague = [0; 64];
    }
    pub(super) fn apply_kind_passive(&mut self, piece: Piece) {
        let mut passive = KindPassive::default();
        match key(piece.kind) {
            "scout" => passive.agility = 1,
            "phantom_scout" => passive.agility = 2,
            "special_commander" => passive.ward = true,
            "shadow_commander" => {
                passive.ward = true;
                passive.agility = 1;
            }
            _ => {}
        }
        if passive.ward {
            self.wards.insert(piece.id);
        }
        if passive.agility > 0 {
            self.grant_agility(piece.id, passive.agility);
        }
        if passive.ward || passive.agility > 0 {
            self.kind_passives.insert(piece.id, passive);
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
    pub fn is_royal_piece(&self, id: PieceId) -> bool {
        if self.sandbox {
            return self.piece_position(id).is_some_and(|(_, piece)| {
                pieces::is_royal(piece.kind) || key(piece.kind) == "warehouse"
            });
        }
        id == self.king_id
            || self.is_enemy_royal(id)
            || self.piece_position(id).is_some_and(|(_, p)| {
                key(p.kind) == "warehouse" || (p.owner == PLAYER && pieces::is_royal(p.kind))
            })
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
        self.apply_plague_on_entry(id);
        self.events.push(format!("예비 폰 배치: {}.", square(at)));
        Ok(())
    }
    pub(super) fn side_actions(&self, owner: PlayerId) -> u8 {
        if owner == ENEMY {
            if self.boss.is_some() {
                2
            } else {
                self.mechanics.enemy_actions
            }
        } else {
            self.actions_per_turn
        }
    }
    pub(super) fn can_act(&self, p: Piece) -> bool {
        p.kind != pieces::CORPSE
            && key(p.kind) != "warehouse"
            && self.status(p.id).incapacitated == 0
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
        if s.corpse_bags > 0 && k == "slaughter_cart" {
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let mut to = Position::new(from.file + dx, from.rank + dy);
                while self.board.contains(to) {
                    let occupied = self.board.piece_at(to).ok().flatten();
                    out.push(DefaultAction::one(ActionStep::new(
                        from,
                        to,
                        from,
                        Effect::Catch,
                    )));
                    if occupied.is_some_and(|q| q.kind != pieces::CORPSE) {
                        break;
                    }
                    to = Position::new(to.file + dx, to.rank + dy);
                }
            }
        }
        if s.corpse_bags > 0 && k == "feast_cart" {
            out.push(DefaultAction::one(ActionStep::new(
                from,
                from,
                from,
                Effect::Catch,
            )));
        }
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
                let sacrifice = q.owner < 2
                    && is_lady(k)
                    && d == 1
                    && !self.is_royal_piece(q.id)
                    && (k != "masked_lady" || pieces::is_pawn(q.kind))
                    && !self.mechanics.turn_used.contains(&format!("lady:{}", p.id));
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
    fn hungry_follower_and_upgrades_do_not_jump_over_named_families() {
        let from = Position::new(3, 4);
        let beyond = Position::new(3, 2);
        for kind in [
            pieces::HUNGRY_FOLLOWER,
            pieces::GLUTTON_FOLLOWER,
            pieces::INVITING_FOLLOWER,
        ] {
            let mut b = fixture(kind);
            b.board
                .set_piece(
                    Position::new(3, 3),
                    Some(Piece {
                        id: 101,
                        kind: pieces::TOAST_LORD,
                        owner: ENEMY,
                    }),
                )
                .unwrap();
            assert!(
                !b.legal_actions(from)
                    .unwrap()
                    .iter()
                    .any(|a| a.first.move_to == beyond),
                "kind {kind} must not leap"
            );
        }
    }
    #[test]
    fn named_families_jump_over_hungry_follower_and_upgrades() {
        let from = Position::new(3, 4);
        for kind in [
            pieces::INTOXICATED_PRIEST,
            pieces::FEAST_PRIEST,
            pieces::LOCUST_HIGH_PRIEST,
            pieces::HUNGER_PRIEST,
            pieces::BUTCHER_CART,
            pieces::FUNERAL_CART,
            pieces::SLAUGHTER_CART,
            pieces::FEAST_CART,
            pieces::MASKED_LADY,
            pieces::RED_LADY,
            pieces::BANQUET_LADY,
            pieces::INVITING_LADY,
            pieces::TOAST_LORD,
            pieces::ECSTATIC_LORD,
            pieces::ETERNAL_GUEST,
        ] {
            let diagonal = matches!(
                kind,
                pieces::INTOXICATED_PRIEST
                    | pieces::FEAST_PRIEST
                    | pieces::LOCUST_HIGH_PRIEST
                    | pieces::HUNGER_PRIEST
            );
            let blocker = if diagonal {
                Position::new(4, 3)
            } else {
                Position::new(3, 3)
            };
            let beyond = if diagonal {
                Position::new(5, 2)
            } else {
                Position::new(3, 2)
            };
            for follower in [
                pieces::HUNGRY_FOLLOWER,
                pieces::GLUTTON_FOLLOWER,
                pieces::INVITING_FOLLOWER,
            ] {
                let mut b = Battle::sandbox().unwrap();
                b.sandbox_place(1, kind, PLAYER, from).unwrap();
                b.sandbox_place(2, follower, PLAYER, blocker).unwrap();
                let action = b
                    .sandbox_actions(from)
                    .unwrap()
                    .into_iter()
                    .find(|a| a.first.move_to == beyond)
                    .unwrap_or_else(|| panic!("kind {kind} must jump over follower {follower}"));
                b.sandbox_apply_action(&action).unwrap();
                assert_eq!(b.board.piece_at(blocker).unwrap().unwrap().kind, follower);

                let mut enemy = Battle::sandbox().unwrap();
                enemy.sandbox_place(1, kind, PLAYER, from).unwrap();
                enemy.sandbox_place(2, follower, ENEMY, blocker).unwrap();
                assert!(
                    !enemy
                        .sandbox_actions(from)
                        .unwrap()
                        .iter()
                        .any(|a| a.first.move_to == beyond),
                    "kind {kind} must not jump over enemy follower {follower}"
                );
            }
        }
        for kind in [
            pieces::FRENZIED_DANCER,
            pieces::MASKED_DANCER,
            pieces::ECSTATIC_DANCER,
            pieces::TEMPTING_DANCER,
        ] {
            let mut b = Battle::sandbox().unwrap();
            b.sandbox_place(1, kind, PLAYER, from).unwrap();
            b.sandbox_place(2, pieces::HUNGRY_FOLLOWER, PLAYER, Position::new(3, 3))
                .unwrap();
            assert!(b
                .sandbox_actions(from)
                .unwrap()
                .iter()
                .any(|a| a.first.move_to == Position::new(4, 2)));
        }
    }
    #[test]
    fn follower_jump_still_respects_other_blockers_and_can_capture_after_landing() {
        let from = Position::new(3, 4);
        let mut b = Battle::sandbox().unwrap();
        b.sandbox_place(1, pieces::BUTCHER_CART, PLAYER, from)
            .unwrap();
        b.sandbox_place(2, pieces::HUNGRY_FOLLOWER, PLAYER, Position::new(3, 3))
            .unwrap();
        b.sandbox_place(3, pieces::PAWN, ENEMY, Position::new(3, 2))
            .unwrap();
        let capture = b
            .sandbox_actions(from)
            .unwrap()
            .into_iter()
            .find(|a| a.first.move_to == Position::new(3, 2))
            .unwrap();
        b.sandbox_apply_action(&capture).unwrap();
        assert_eq!(
            b.board.piece_at(Position::new(3, 3)).unwrap().unwrap().kind,
            pieces::HUNGRY_FOLLOWER
        );
        assert_eq!(
            b.board.piece_at(Position::new(3, 2)).unwrap().unwrap().kind,
            pieces::BUTCHER_CART
        );

        let mut blocked = fixture(pieces::BUTCHER_CART);
        assert!(!blocked
            .legal_actions(from)
            .unwrap()
            .iter()
            .any(|a| a.first.move_to == Position::new(3, 2)));
        blocked.board.set_piece(Position::new(3, 3), None).unwrap();
        blocked
            .board
            .set_piece(
                Position::new(3, 2),
                Some(Piece {
                    id: 102,
                    kind: pieces::HUNGRY_FOLLOWER,
                    owner: ENEMY,
                }),
            )
            .unwrap();
        blocked
            .board
            .set_piece(
                Position::new(3, 3),
                Some(Piece {
                    id: 101,
                    kind: pieces::PAWN,
                    owner: PLAYER,
                }),
            )
            .unwrap();
        assert!(!blocked
            .legal_actions(from)
            .unwrap()
            .iter()
            .any(|a| a.first.move_to == Position::new(3, 1)));
    }
    #[test]
    fn funeral_cart_keeps_its_corpse_passage_when_jumping_a_follower() {
        let from = Position::new(3, 4);
        let mut b = Battle::sandbox().unwrap();
        b.sandbox_place(1, pieces::FUNERAL_CART, PLAYER, from)
            .unwrap();
        b.board
            .set_piece(
                Position::new(3, 3),
                Some(Piece {
                    id: 3,
                    kind: pieces::CORPSE,
                    owner: 2,
                }),
            )
            .unwrap();
        b.sandbox_place(2, pieces::HUNGRY_FOLLOWER, PLAYER, Position::new(3, 2))
            .unwrap();
        assert!(b
            .sandbox_actions(from)
            .unwrap()
            .iter()
            .any(|a| a.first.move_to == Position::new(3, 1)));
    }
    #[test]
    fn lord_jump_ends_on_the_square_after_the_follower() {
        let from = Position::new(3, 4);
        let mut b = Battle::sandbox().unwrap();
        b.sandbox_place(1, pieces::TOAST_LORD, PLAYER, from)
            .unwrap();
        b.sandbox_place(2, pieces::HUNGRY_FOLLOWER, PLAYER, Position::new(3, 3))
            .unwrap();
        let actions = b.sandbox_actions(from).unwrap();
        assert!(actions
            .iter()
            .any(|a| a.first.move_to == Position::new(3, 2)));
        assert!(!actions
            .iter()
            .any(|a| a.first.move_to == Position::new(3, 1)));
    }
    #[test]
    fn glutton_receives_one_move_right_per_corpse_without_negative_effects() {
        let mut b = fixture(pieces::GLUTTON_FOLLOWER);
        b.eat_corpse(1).unwrap();
        b.eat_corpse(1).unwrap();
        assert_eq!(b.stats(1).move_rights, 2);
        let mut dancer = fixture(pieces::ECSTATIC_DANCER);
        dancer.eat_corpse(1).unwrap();
        assert_eq!(dancer.stats(1).move_rights, 2);
        assert_eq!(dancer.status(1).madness + dancer.status(1).guilt, 0);
    }
    #[test]
    fn lady_can_make_enemy_pawn_a_corpse() {
        let mut b = fixture(pieces::MASKED_LADY);
        let from = Position::new(3, 4);
        let to = Position::new(3, 3);
        let action = b
            .legal_actions(from)
            .unwrap()
            .into_iter()
            .find(|a| a.first.effect == Effect::Catch && a.first.take == to)
            .unwrap();
        b.apply_piece_action(&action).unwrap();
        assert_eq!(b.board.piece_at(to).unwrap().unwrap().kind, pieces::CORPSE);
    }
    #[test]
    fn spirit_bomb_carries_move_rights_across_turns() {
        let mut b = fixture(pieces::ROOK);
        b.enable_card_movement();
        b.grant_move_rights(1, 2).unwrap();
        b.avenger_effect(crate::content::cards::CardKind::SpiritBomb, None, None)
            .unwrap();
        b.end_turn().unwrap();
        b.end_turn().unwrap();
        assert_eq!(b.stats(1).move_rights, 2);
    }
    #[test]
    fn follower_last_rank_chooses_a_nonroyal_enemy_to_remove() {
        let mut b = fixture(pieces::PROCESSION_FOLLOWER);
        let from = Position::new(3, 1);
        let actor = b.board.piece_at(Position::new(3, 4)).unwrap().unwrap();
        b.board.set_piece(Position::new(3, 4), None).unwrap();
        b.board.set_piece(from, Some(actor)).unwrap();
        let action = b
            .legal_actions(from)
            .unwrap()
            .into_iter()
            .find(|a| a.first.move_to == Position::new(3, 0))
            .unwrap();
        b.apply_piece_action(&action).unwrap();
        let choice = b.pending_choice().unwrap();
        assert_eq!(choice.targets, vec![101]);
        b.resolve_choice(101).unwrap();
        assert!(b.piece_position(101).is_none());
        assert_eq!(
            b.piece_position(1).unwrap().1.kind,
            pieces::PROCESSION_FOLLOWER
        );
    }
    #[test]
    fn corpse_throw_creates_plague_and_entry_applies_both_statuses() {
        let mut b = fixture(pieces::SLAUGHTER_CART);
        b.mechanics.statuses.entry(1).or_default().corpse_bags = 1;
        let from = Position::new(3, 4);
        let to = Position::new(4, 4);
        let action = b
            .legal_actions(from)
            .unwrap()
            .into_iter()
            .find(|a| a.first.effect == Effect::Catch && a.first.take == to)
            .unwrap();
        b.apply_piece_action(&action).unwrap();
        assert_eq!(b.status(1).corpse_bags, 0);
        assert_eq!(
            b.plague_cells()
                .iter()
                .find(|(at, _)| *at == to)
                .map(|(_, n)| *n),
            Some(2)
        );
        let enemy = b.board.piece_at(Position::new(3, 3)).unwrap().unwrap();
        b.board.set_piece(Position::new(3, 3), None).unwrap();
        b.board.set_piece(to, Some(enemy)).unwrap();
        b.apply_plague_on_entry(enemy.id);
        assert!(b.status(enemy.id).ingredient);
        assert_eq!(b.status(enemy.id).pest, 3);
    }
    #[test]
    fn pest_expiry_on_enemy_royal_finishes_battle_immediately() {
        let mut b = fixture(pieces::ROOK);
        b.mechanics.statuses.entry(100).or_default().pest = 1;
        b.end_turn().unwrap();
        assert!(b.piece_position(100).is_none());
        assert_eq!(b.phase, Phase::Finished);
        assert!(matches!(b.result, GameResult::Win { winner: PLAYER, .. }));
    }
    #[test]
    fn ordinary_lord_marks_both_sides_as_ingredients_without_blessing() {
        let mut b = fixture(pieces::ROOK);
        b.board
            .set_piece(
                Position::new(3, 3),
                Some(Piece {
                    id: 101,
                    kind: pieces::TOAST_LORD,
                    owner: ENEMY,
                }),
            )
            .unwrap();
        b.configure_v030(&[]);
        assert!(b.status(1).ingredient);
        assert!(b.status(101).ingredient);
        assert!(!b.status(1).blessing);
        assert!(!b.status(101).blessing);
    }
    #[test]
    fn incapacitation_keeps_its_full_stack_on_the_application_turn() {
        let mut b = fixture(pieces::ROOK);
        let status = b.mechanics.statuses.entry(1).or_default();
        status.incapacitated = 3;
        status.incapacitated_fresh = true;
        b.end_turn().unwrap();
        assert_eq!(b.status(1).incapacitated, 3);
        b.end_turn().unwrap();
        b.end_turn().unwrap();
        assert_eq!(b.status(1).incapacitated, 2);
    }
    #[test]
    fn ingredient_capture_stays_then_corpse_capture_ends_turn_after_effects() {
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
        assert_eq!(b.mechanics.corpse_eaten[0], 1);
        assert_eq!(b.status(1).agility, 0);
        assert_eq!(b.turn, ENEMY);
    }
    #[test]
    fn pending_blessing_agility_stacks_and_is_paid_once_on_either_side() {
        for (id, owner) in [(1, PLAYER), (101, ENEMY)] {
            let mut b = fixture(ROOK);
            b.turn = owner;
            b.mechanics.statuses.entry(id).or_default().blessing = true;
            // Multiple corpse effects before the turn boundary accumulate.
            b.eat_corpse(id).unwrap();
            b.eat_corpse(id).unwrap();
            assert_eq!(b.status(id).agility, 0);
            assert_eq!(b.mechanics.pending_agility.get(&id), Some(&6));
            let mut without_reward = b.clone();
            without_reward.mechanics.pending_agility.clear();
            assert_ne!(b.search_hash(), without_reward.search_hash());
            b.end_turn().unwrap();
            assert_eq!(b.status(id).agility, 0);
            assert_eq!(b.mechanics.pending_agility.get(&id), Some(&6));
            b.grant_agility(id, 1);
            b.end_turn().unwrap();
            assert_eq!(b.turn, owner);
            assert_eq!(b.status(id).agility, 7);
            assert_eq!(b.actions_left, 8);
            assert!(!b.mechanics.pending_agility.contains_key(&id));
            b.reset_turn_budget();
            assert_eq!(b.status(id).agility, 7);
            assert_eq!(b.actions_left, 8, "recalculation must not pay again");
            b.end_turn().unwrap();
            b.end_turn().unwrap();
            assert_eq!(b.status(id).agility, 0);
            assert_eq!(b.actions_left, 1);
        }
    }
    #[test]
    fn conscience_and_hunger_persist_once() {
        let mut b = fixture(ROOK);
        b.mechanics.statuses.entry(1).or_default().guilt = 2;
        b.resolve_conscience(1).unwrap();
        assert!(b.status(1).incapacitated > 0);
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
        b.set_ammunition(0);
        assert_eq!(b.apply_piece_action(&shot), Err(Error::IllegalAction));
        assert_eq!(b.turn, PLAYER);
        b.set_ammunition(1);
        b.apply_piece_action(&shot).unwrap();
        assert_eq!(b.board.piece_at(from).unwrap().unwrap().id, 1);
        assert_eq!(b.take_resource_effects().2, -1);
        assert!(!b.has_ward(101));
        assert_eq!(
            b.board.piece_at(Position::new(3, 3)).unwrap().unwrap().kind,
            PAWN
        );
    }
    #[test]
    fn weakness_follows_the_recorded_cross_squad_upgrade_path() {
        let mut battle = fixture(pieces::SNIPER);
        battle
            .players
            .iter_mut()
            .find(|p| p.id == 1)
            .unwrap()
            .upgrades = vec!["rifleman".into(), "sniper".into()];
        battle.mechanics.statuses.entry(1).or_default().weakness = 2;
        battle.start_mechanics_turn().unwrap();
        assert_eq!(battle.piece_position(1).unwrap().1.kind, BISHOP);
        let owned = battle.players.iter().find(|p| p.id == 1).unwrap();
        assert!(owned.upgrades.is_empty());
        assert!(pieces::is_infantry(pieces::SNIPER));
        assert!(!pieces::is_infantry(BISHOP));
    }
    #[test]
    fn eternal_guest_grants_the_first_three_corpse_bonuses_on_either_side() {
        for owner in [PLAYER, ENEMY] {
            let mut players = Battle::standard_roster();
            for p in &mut players {
                if p.id != 5 {
                    p.position = None;
                }
            }
            if owner == PLAYER {
                players[4].kind = pieces::ETERNAL_GUEST;
            }
            let mut battle = Battle::new(BattleStart {
                players,
                encounter: Encounter {
                    enemies: vec![EnemyPiece {
                        id: 100,
                        kind: if owner == ENEMY {
                            pieces::ETERNAL_GUEST
                        } else {
                            KING
                        },
                        position: Position::new(4, 0),
                    }],
                    royal_id: 100,
                    additional_royal_ids: vec![],
                },
            })
            .unwrap();
            let guest = if owner == PLAYER { 5 } else { 100 };
            battle.turn = owner;
            for n in 1..=4 {
                battle.eat_corpse(guest).unwrap();
                assert_eq!(battle.status(guest).agility, n.min(3));
            }
        }
    }
    #[test]
    fn missed_shot_ends_turn_despite_remaining_actions() {
        let mut b = fixture(pieces::RIFLEMAN);
        b.set_ammunition(1);
        b.actions_left = 3;
        b.force_search_shot(false);
        let from = Position::new(3, 4);
        let shot = b
            .special_actions(from)
            .into_iter()
            .find(|a| a.first.take == Position::new(3, 3))
            .unwrap();
        b.apply_piece_action(&shot).unwrap();
        assert_eq!(b.turn, ENEMY);
        assert_eq!(
            b.board.piece_at(Position::new(3, 3)).unwrap().unwrap().id,
            101
        );
    }
    #[test]
    fn pistol_range_falloff_precedes_accuracy_bonuses() {
        let mut b = Battle::sandbox().unwrap();
        let from = Position::new(3, 6);
        let to = Position::new(3, 2);
        b.sandbox_place(1, pieces::PISTOL_INFANTRY, PLAYER, from)
            .unwrap();
        b.sandbox_place(2, PAWN, ENEMY, to).unwrap();
        assert_eq!(b.hit_chance(from, to), Some(85));
        b.configure_v030(&[Relic::FireControl]);
        assert_eq!(b.hit_chance(from, to), Some(95));
    }
    #[test]
    fn card_shot_and_card_corpse_capture_end_turn() {
        let mut shot = fixture(pieces::RIFLEMAN);
        shot.set_ammunition(1);
        shot.actions_left = 3;
        shot.force_search_shot(false);
        shot.free_counterattack(1, 101, true).unwrap();
        shot.finish_forced_turn().unwrap();
        assert_eq!(shot.turn, ENEMY);

        let mut corpse = fixture(ROOK);
        let from = Position::new(3, 4);
        let to = Position::new(3, 3);
        corpse
            .board
            .set_piece(
                to,
                Some(Piece {
                    id: 101,
                    kind: pieces::CORPSE,
                    owner: 2,
                }),
            )
            .unwrap();
        corpse.actions_left = 3;
        let action = DefaultAction::one(ActionStep::new(from, to, to, Effect::Take));
        corpse.apply_resolved_card_effect(&action).unwrap();
        corpse.finish_forced_turn().unwrap();
        assert_eq!(corpse.turn, ENEMY);
    }
    #[test]
    fn death_threshold_persists_then_triggers_once_before_next_action() {
        let mut first = fixture(ROOK);
        first.grant_limiter(1).unwrap();
        first.grant_death_threshold(1).unwrap();
        assert!(first.piece_position(1).is_some());
        assert!(first.status(1).death_threshold);
        let players = first.outcome();
        assert_eq!(
            players
                .iter()
                .find(|p| p.id == 1)
                .unwrap()
                .persistent
                .get("death_threshold"),
            Some(&1)
        );
        let second = Battle::new(BattleStart {
            players,
            encounter: Encounter {
                enemies: vec![EnemyPiece {
                    id: 100,
                    kind: KING,
                    position: Position::new(7, 0),
                }],
                royal_id: 100,
                additional_royal_ids: vec![],
            },
        })
        .unwrap();
        assert!(second.piece_position(1).is_none());
        assert!(!second.status(1).death_threshold);
        assert!(!second
            .outcome()
            .iter()
            .find(|p| p.id == 1)
            .unwrap()
            .persistent
            .contains_key("death_threshold"));
        assert_eq!(
            second
                .events
                .iter()
                .filter(|event| event.contains("죽음의 문턱"))
                .count(),
            1
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
