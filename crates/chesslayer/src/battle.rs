//! Chesslayer battle rules and persistent player piece state.
use crate::{
    apply_default_action, ActionError, Board, BoardError, DefaultAction, GameResult, Piece,
    PieceId, PieceType, PlayerId, Position, ResultReason,
};
use chessembly::{Error as DslError, Program};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::sync::Arc;
mod avenger;
mod boss;
mod combat_state;
pub use combat_state::{classification, CombatStats, PieceClass};
mod effects;
mod mechanics;
mod movement;
pub use boss::BossState;
pub(crate) use mechanics::shot_preview;
pub use mechanics::{EffectChoice, PieceStatus};
pub const PLAYER: PlayerId = 0;
pub const ENEMY: PlayerId = 1;
// Keep the existing crate-root API through battle's re-exports.
pub use crate::content::pieces::{
    piece_name, AMAZON, BISHOP, BOUNCING_BISHOP, BOUNCING_PAWN, BOUNCING_QUEEN, BOUNCING_ROOK,
    KING, KNIGHT, PAWN, QUEEN, ROOK, TEMPEST_BISHOP, TEMPEST_KNIGHT, TEMPEST_PAWN, TEMPEST_QUEEN,
    TEMPEST_ROOK,
};
use crate::content::{pieces, Rng, INITIATIVE_TURNS};

#[derive(Debug, Clone, Copy)]
pub struct BattleOptions {
    pub enemy_racing_goal: Option<i32>,
    pub actions_per_turn: u8,
    pub self_defense: bool,
    pub initiative: bool,
    pub enemy_initiative: bool,
}
impl Default for BattleOptions {
    fn default() -> Self {
        Self {
            enemy_racing_goal: None,
            actions_per_turn: 1,
            self_defense: false,
            initiative: false,
            enemy_initiative: false,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerPiece {
    pub id: PieceId,
    /// Permanent kind. Combat promotions exist only on the battle board.
    pub kind: PieceType,
    /// Permanent home, independent of the current kind, position or survival.
    pub origin: Position,
    pub position: Option<Position>,
    pub moved: bool,
    pub upgrades: Vec<String>,
    pub persistent: BTreeMap<String, i32>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnemyPiece {
    pub id: PieceId,
    pub kind: PieceType,
    pub position: Position,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Encounter {
    pub enemies: Vec<EnemyPiece>,
    pub royal_id: PieceId,
    /// Additional enemies that must also be captured to win.
    pub additional_royal_ids: Vec<PieceId>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattleStart {
    pub players: Vec<PlayerPiece>,
    pub encounter: Encounter,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Active,
    Finished,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionSource {
    PieceMove,
    ResolvedCardEffect,
}
/// Read-only, board-relative action details for the inspection UI.
pub struct ActionPreview {
    pub destination: Position,
    pub captures: Vec<Position>,
    pub route: Option<Vec<Position>>,
}
/// Resolved presentation data. These records never participate in battle rules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CombatEvent {
    Shotgun {
        actor_id: PieceId,
        from: Position,
        toward: Position,
        rays: Vec<ShotgunRay>,
        hits: Vec<ShotgunHit>,
    },
    ChainPull {
        boss_id: PieceId,
        boss_at: Position,
        piece_id: PieceId,
        from: Position,
        to: Position,
    },
    StatusTriggered {
        piece_id: PieceId,
        at: Position,
        status: &'static str,
    },
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShotgunRay {
    pub direction: (i32, i32),
    /// Traversed cells, including the first blocker and never extending past it.
    pub cells: Vec<Position>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShotgunHit {
    pub at: Position,
    pub piece_id: PieceId,
    pub result: ShotgunResult,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ShotgunResult {
    Hit,
    Miss,
    Ward,
    Corpse,
}
/// Can only be obtained from this battle while it is immutably borrowed.
pub(crate) struct GeneratedAction<'a> {
    battle: &'a Battle,
    action: DefaultAction,
}
impl GeneratedAction<'_> {
    pub(crate) fn action(&self) -> &DefaultAction {
        &self.action
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Board(BoardError),
    Dsl(DslError),
    Effect(ActionError),
    InvalidSetup,
    WrongTurn,
    Finished,
    IllegalAction,
    ProtectedRoyal,
}
impl From<BoardError> for Error {
    fn from(e: BoardError) -> Self {
        Self::Board(e)
    }
}
impl From<DslError> for Error {
    fn from(e: DslError) -> Self {
        Self::Dsl(e)
    }
}
impl From<ActionError> for Error {
    fn from(e: ActionError) -> Self {
        Self::Effect(e)
    }
}
#[derive(Clone)]
pub struct Battle {
    pub board: Board,
    combat_stats: BTreeMap<PieceId, CombatStats>,
    card_movement: bool,
    en_passant: Option<(PieceId, Position)>,
    pub enemy_racing_goal: Option<i32>,
    pub turn: PlayerId,
    pub result: GameResult,
    pub phase: Phase,
    pub royal_id: PieceId,
    additional_royal_ids: Vec<PieceId>,
    pub king_id: PieceId,
    pub actions_left: u8,
    normal_actions_used: u8,
    limiter_turn: bool,
    forced_turn_end: bool,
    pub events: Vec<String>,
    pub combat_events: Vec<CombatEvent>,
    actions_per_turn: u8,
    wards: HashSet<PieceId>,
    kind_passives: HashMap<PieceId, KindPassive>,
    gambler_dice: HashMap<PieceId, u8>,
    temporary_upgrades: HashSet<PieceId>,
    limiter: HashMap<PieceId, LimiterState>,
    self_defense_target: Option<PieceId>,
    initiative_turns_left: u8,
    enemy_initiative_turns_left: u8,
    absorbed: HashMap<PieceId, HashSet<PieceType>>,
    boss: Option<BossState>,
    pub(crate) boss_id: PieceId,
    players: Vec<PlayerPiece>,
    dead_at_start: HashSet<PieceId>,
    programs: Arc<BTreeMap<PieceType, Program>>,
    initial_double_step: Arc<Program>,
    moved_ids: HashSet<PieceId>,
    mechanics: mechanics::Mechanics,
    sandbox: bool,
}
#[derive(Clone, Copy)]
struct LimiterState {
    owner: PlayerId,
    moves_left: u8,
}
#[derive(Clone, Copy, Default)]
struct KindPassive {
    agility: u8,
    ward: bool,
}
impl Battle {
    pub(crate) fn force_search_shot(&mut self, hit: bool) {
        self.mechanics.force_search_shot(hit);
    }
    pub(crate) fn force_search_boss_shots(&mut self, hits: Vec<bool>) {
        self.mechanics.force_search_boss_shots(hits);
    }
    pub(crate) fn take_search_boss_shot_chance(&mut self) -> Option<u8> {
        self.mechanics.take_search_boss_shot_chance()
    }
    pub(crate) fn force_search_random(&mut self, index: usize) {
        self.mechanics.force_search_random(index);
    }
    pub(crate) fn take_search_random_arity(&mut self) -> Option<usize> {
        self.mechanics.take_search_random_arity()
    }
    pub(crate) fn search_hash(&self) -> u64 {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        self.board.cells().hash(&mut h);
        self.combat_stats.hash(&mut h);
        self.card_movement.hash(&mut h);
        self.en_passant.hash(&mut h);
        self.king_id.hash(&mut h);
        self.royal_id.hash(&mut h);
        self.boss_id.hash(&mut h);
        self.additional_royal_ids.hash(&mut h);
        self.enemy_racing_goal.hash(&mut h);
        self.turn.hash(&mut h);
        self.actions_left.hash(&mut h);
        self.normal_actions_used.hash(&mut h);
        self.limiter_turn.hash(&mut h);
        self.forced_turn_end.hash(&mut h);
        self.initiative_turns_left.hash(&mut h);
        self.enemy_initiative_turns_left.hash(&mut h);
        self.self_defense_target.hash(&mut h);
        if let Some(boss) = &self.boss {
            boss.hp.hash(&mut h);
            boss.max_health.hash(&mut h);
            boss.ammo.hash(&mut h);
            boss.hook.hash(&mut h);
            boss.movement_used.hash(&mut h);
            boss.special_used.hash(&mut h);
        }
        for set in [&self.wards, &self.temporary_upgrades, &self.moved_ids] {
            let mut ids: Vec<_> = set.iter().collect();
            ids.sort();
            ids.hash(&mut h);
        }
        let mut dice: Vec<_> = self.gambler_dice.iter().collect();
        dice.sort_by_key(|(id, _)| **id);
        dice.hash(&mut h);
        let mut limiter: Vec<_> = self.limiter.iter().collect();
        limiter.sort_by_key(|(id, _)| **id);
        for (id, state) in limiter {
            id.hash(&mut h);
            state.owner.hash(&mut h);
            state.moves_left.hash(&mut h);
        }
        let mut absorbed: Vec<_> = self.absorbed.iter().collect();
        absorbed.sort_by_key(|(id, _)| **id);
        for (id, kinds) in absorbed {
            id.hash(&mut h);
            let mut kinds: Vec<_> = kinds.iter().collect();
            kinds.sort();
            kinds.hash(&mut h);
        }
        self.mechanics.search_hash(&mut h);
        h.finish()
    }
    pub fn standard_roster() -> Vec<PlayerPiece> {
        let back = [ROOK, KNIGHT, BISHOP, QUEEN, KING, BISHOP, KNIGHT, ROOK];
        (0..16)
            .map(|i| PlayerPiece {
                id: i + 1,
                kind: if i < 8 { back[i as usize] } else { PAWN },
                origin: Position::new((i % 8) as i32, if i < 8 { 7 } else { 6 }),
                position: Some(Position::new((i % 8) as i32, if i < 8 { 7 } else { 6 })),
                moved: false,
                upgrades: vec![],
                persistent: BTreeMap::new(),
            })
            .collect()
    }
    pub fn new(start: BattleStart) -> Result<Self, Error> {
        Self::with_options(start, BattleOptions::default(), &mut Rng::new(0))
    }
    pub fn with_options(
        start: BattleStart,
        options: BattleOptions,
        rng: &mut Rng,
    ) -> Result<Self, Error> {
        Self::build(start, options, rng, false)
    }
    fn build(
        start: BattleStart,
        options: BattleOptions,
        rng: &mut Rng,
        sandbox: bool,
    ) -> Result<Self, Error> {
        if options.actions_per_turn == 0 {
            return Err(Error::InvalidSetup);
        }
        let mut events = vec![];
        let mut board = Board::new(8, 8)?;
        board.set_catalog(
            pieces::catalog(),
            vec!["player".into(), "enemy".into(), "corpse".into()],
            vec![1, -1, 1],
        )?;
        let mut ids = HashSet::new();
        let mut origins = HashSet::new();
        let mut king_id = None;
        for p in &start.players {
            if !ids.insert(p.id)
                || !valid_kind(p.kind)
                || board.index(p.origin).is_none()
                || !origins.insert(p.origin)
            {
                return Err(Error::InvalidSetup);
            }
            if (p
                .persistent
                .get("royal")
                .map(|&v| v != 0)
                .unwrap_or_else(|| pieces::is_royal(p.kind)))
                && king_id.replace(p.id).is_some()
            {
                return Err(Error::InvalidSetup);
            }
            if let Some(pos) = p.position {
                if board.piece_at(pos)?.is_some() {
                    return Err(Error::InvalidSetup);
                }
                board.set_piece(
                    pos,
                    Some(Piece {
                        kind: p.kind,
                        owner: PLAYER,
                        id: p.id,
                    }),
                )?
            }
        }
        let king_id = king_id
            .or_else(|| {
                start
                    .players
                    .iter()
                    .find(|p| pieces::definition(p.kind).is_some_and(|d| d.key == "warehouse"))
                    .map(|p| p.id)
            })
            .or(if sandbox { Some(0) } else { None })
            .ok_or(Error::InvalidSetup)?;
        let mut players = start.players;
        for p in &mut players {
            p.persistent.remove("hunger_worsened");
        }
        let mut required_royals = start.encounter.additional_royal_ids.clone();
        required_royals.push(start.encounter.royal_id);
        if required_royals
            .iter()
            .copied()
            .collect::<HashSet<_>>()
            .len()
            != required_royals.len()
        {
            return Err(Error::InvalidSetup);
        }
        let mut found_royals = HashSet::new();
        let mut enemy_positions = HashSet::new();
        for e in &start.encounter.enemies {
            if !ids.insert(e.id) || !valid_kind(e.kind) || !enemy_positions.insert(e.position) {
                return Err(Error::InvalidSetup);
            }
            if required_royals.contains(&e.id) {
                found_royals.insert(e.id);
            }
            if let Some(old) = board.piece_at(e.position)? {
                if old.owner == ENEMY {
                    return Err(Error::InvalidSetup);
                }
                if let Some(p) = players.iter_mut().find(|p| p.id == old.id) {
                    p.position = None;
                    events.push(format!(
                        "역습: {}의 아군 {}이 제거되었습니다.",
                        square(e.position),
                        piece_name(p.kind)
                    ));
                }
            }
            board.set_piece(
                e.position,
                Some(Piece {
                    kind: e.kind,
                    owner: ENEMY,
                    id: e.id,
                }),
            )?
        }
        if !sandbox && found_royals.len() != required_royals.len() {
            return Err(Error::InvalidSetup);
        }
        let programs = pieces::ALL
            .iter()
            .map(|d| Program::parse(&(d.movement)()).map(|program| (d.kind, program)))
            .collect::<Result<_, _>>()?;
        let initial_double_step = Program::parse(pieces::movement::INITIAL_DOUBLE_STEP)?;
        let moved_ids = players.iter().filter(|p| p.moved).map(|p| p.id).collect();
        let dead_at_start = players
            .iter()
            .filter(|p| p.position.is_none())
            .map(|p| p.id)
            .collect();
        let mut battle = Self {
            combat_stats: BTreeMap::new(),
            card_movement: false,
            en_passant: None,
            enemy_racing_goal: options.enemy_racing_goal,
            events,
            combat_events: vec![],
            wards: HashSet::new(),
            kind_passives: HashMap::new(),
            gambler_dice: HashMap::new(),
            temporary_upgrades: HashSet::new(),
            limiter: HashMap::new(),
            self_defense_target: None,
            initiative_turns_left: if options.initiative && !options.enemy_initiative {
                INITIATIVE_TURNS
            } else {
                0
            },
            enemy_initiative_turns_left: if options.enemy_initiative && !options.initiative {
                INITIATIVE_TURNS
            } else {
                0
            },
            absorbed: HashMap::new(),
            boss_id: start.encounter.royal_id,
            boss: start
                .encounter
                .enemies
                .iter()
                .any(|p| p.id == start.encounter.royal_id && p.kind == pieces::TRIBAL_DICTATOR)
                .then_some(BossState {
                    hp: 5,
                    max_health: 5,
                    ammo: 2,
                    hook: None,
                    movement_used: false,
                    special_used: false,
                }),
            actions_left: options.actions_per_turn,
            normal_actions_used: 0,
            limiter_turn: false,
            forced_turn_end: false,
            actions_per_turn: options.actions_per_turn,
            board,
            turn: PLAYER,
            result: GameResult::Ongoing,
            phase: Phase::Active,
            royal_id: start.encounter.royal_id,
            additional_royal_ids: start.encounter.additional_royal_ids,
            king_id,
            players,
            dead_at_start,
            programs: Arc::new(programs),
            initial_double_step: Arc::new(initial_double_step),
            moved_ids,
            mechanics: mechanics::Mechanics::new(rng.clone()),
            sandbox,
        };
        if let Some(boss) = &battle.boss {
            battle.combat_stats.insert(
                battle.boss_id,
                CombatStats {
                    health: boss.hp,
                    max_health: boss.hp,
                    ..CombatStats::default()
                },
            );
        }
        battle.initialize_statuses();
        battle.roll_all_gambler_dice();
        battle.trigger_death_thresholds()?;
        battle.mechanics.enemy_actions = options.actions_per_turn;
        battle.update_result(ResultReason::Counterattack);
        if options.self_defense && battle.phase == Phase::Active {
            let living: Vec<_> = battle
                .board
                .cells()
                .iter()
                .flatten()
                .filter(|p| p.owner == PLAYER)
                .collect();
            let target = living[rng.index(living.len())];
            battle.self_defense_target = Some(target.id);
            battle.events.push(format!(
                "호신용품: 아군 {}이 보호 대상으로 선택되었습니다. 적에게 포획되면 포획자도 사망합니다.",
                piece_name(target.kind)
            ));
        }
        Ok(battle)
    }
    /// Isolated board using the normal movement and mechanics pipeline.
    pub fn sandbox() -> Result<Self, Error> {
        Self::build(
            BattleStart {
                players: vec![],
                encounter: Encounter {
                    enemies: vec![],
                    royal_id: 0,
                    additional_royal_ids: vec![],
                },
            },
            BattleOptions::default(),
            &mut Rng::new(0),
            true,
        )
    }
    pub fn sandbox_place(
        &mut self,
        id: PieceId,
        kind: PieceType,
        owner: PlayerId,
        at: Position,
    ) -> Result<(), Error> {
        if !self.sandbox
            || owner > ENEMY
            || !valid_kind(kind)
            || self.board.piece_at(at)?.is_some()
            || self.board.cells().iter().flatten().any(|p| p.id == id)
        {
            return Err(Error::InvalidSetup);
        }
        self.board.set_piece(at, Some(Piece { id, kind, owner }))?;
        if owner == PLAYER {
            self.players.push(PlayerPiece {
                id,
                kind,
                origin: at,
                position: Some(at),
                moved: false,
                upgrades: vec![],
                persistent: BTreeMap::new(),
            });
            self.sandbox_ally_added();
        }
        self.mechanics.statuses.insert(id, PieceStatus::default());
        self.roll_gambler_die(Piece { id, kind, owner });
        Ok(())
    }
    pub fn sandbox_remove(&mut self, at: Position) -> Result<(), Error> {
        if !self.sandbox {
            return Err(Error::IllegalAction);
        }
        let piece = self.board.piece_at(at)?.ok_or(Error::IllegalAction)?;
        self.board.set_piece(at, None)?;
        self.players.retain(|p| p.id != piece.id);
        self.mechanics.statuses.remove(&piece.id);
        self.mechanics.pending_agility.remove(&piece.id);
        self.moved_ids.remove(&piece.id);
        self.wards.remove(&piece.id);
        self.limiter.remove(&piece.id);
        self.gambler_dice.remove(&piece.id);
        Ok(())
    }
    pub fn sandbox_change_kind(&mut self, at: Position, kind: PieceType) -> Result<(), Error> {
        if !self.sandbox || !valid_kind(kind) {
            return Err(Error::IllegalAction);
        }
        let mut piece = self.board.piece_at(at)?.ok_or(Error::IllegalAction)?;
        piece.kind = kind;
        self.board.set_piece(at, Some(piece))?;
        if let Some(p) = self.players.iter_mut().find(|p| p.id == piece.id) {
            p.kind = kind;
        }
        self.mechanics
            .statuses
            .insert(piece.id, PieceStatus::default());
        self.mechanics.pending_agility.remove(&piece.id);
        self.roll_gambler_die(piece);
        Ok(())
    }
    pub fn sandbox_actions(&self, from: Position) -> Result<Vec<DefaultAction>, Error> {
        if !self.sandbox {
            return Err(Error::IllegalAction);
        }
        self.actions_from(from, false)
    }
    pub fn sandbox_apply_action(&mut self, action: &DefaultAction) -> Result<(), Error> {
        if !self.sandbox || self.pending_choice().is_some() {
            return Err(Error::IllegalAction);
        }
        let piece = self
            .board
            .piece_at(action.first.from)?
            .ok_or(Error::IllegalAction)?;
        if !self.sandbox_actions(action.first.from)?.contains(action) {
            return Err(Error::IllegalAction);
        }
        let mut next = self.clone();
        next.turn = piece.owner;
        next.normal_actions_used = 0;
        next.actions_left = u8::MAX;
        next.apply_piece_action_unchecked(action)?;
        next.turn = PLAYER;
        next.reset_turn_budget();
        *self = next;
        Ok(())
    }
    pub fn sandbox_progress_effects(&mut self) -> Result<(), Error> {
        if !self.sandbox || self.pending_choice().is_some() {
            return Err(Error::IllegalAction);
        }
        self.turn = PLAYER;
        self.end_turn()?;
        self.end_turn()?;
        Ok(())
    }
    pub fn sandbox_drone_ability(&mut self, from: Position) -> Result<(), Error> {
        if !self.sandbox || self.pending_choice().is_some() {
            return Err(Error::IllegalAction);
        }
        let piece = self.board.piece_at(from)?.ok_or(Error::IllegalAction)?;
        let mut next = self.clone();
        next.turn = piece.owner;
        next.normal_actions_used = 0;
        next.actions_left = u8::MAX;
        next.use_drone_ability(from)?;
        next.turn = PLAYER;
        next.reset_turn_budget();
        *self = next;
        Ok(())
    }
    fn reset_turn_budget(&mut self) {
        self.normal_actions_used = 0;
        if let Some(boss) = self.boss.as_mut() {
            boss.movement_used = false;
            boss.special_used = false;
        }
        for piece in self
            .board
            .cells()
            .iter()
            .flatten()
            .copied()
            .filter(|piece| piece.owner == self.turn)
        {
            if let Some(bonus) = self.mechanics.pending_agility.remove(&piece.id) {
                let status = self.mechanics.statuses.entry(piece.id).or_default();
                // Match grant_agility's stacking; the budget is summed once below.
                status.agility = status.agility.saturating_add(bonus);
            }
        }
        self.actions_left = self.side_actions(self.turn);
        if self.turn == ENEMY && self.boss.is_some() {
            // The dictator has exactly one movement and one stationary ability;
            // agility and limiter budgets cannot grant a third boss action.
            self.limiter_turn = false;
            return;
        }
        let bonus = self
            .board
            .cells()
            .iter()
            .flatten()
            .filter(|p| p.owner == self.turn)
            .fold(0u8, |n, p| n.saturating_add(self.agility(p.id)));
        self.actions_left = self.actions_left.saturating_add(bonus);
        let remaining_moves = self
            .limiter
            .values()
            .filter(|s| s.owner == self.turn)
            .fold(0u8, |total, s| total.saturating_add(s.moves_left));
        self.limiter_turn = remaining_moves > 0;
        self.actions_left = self.actions_left.saturating_add(remaining_moves);
    }
    /// Run boundary: keep identities, deaths and permanent growth; discard combat state.
    pub(crate) fn reset_after_battle(&mut self) -> Result<(), Error> {
        let mut roster = self.outcome();
        for p in &mut roster {
            p.position = p.position.map(|_| p.origin);
            p.moved = false;
        }
        self.replace_roster_at_rest(roster)
    }
    pub(crate) fn replace_roster_at_rest(&mut self, roster: Vec<PlayerPiece>) -> Result<(), Error> {
        if self.phase != Phase::Finished {
            return Err(Error::IllegalAction);
        }
        let mut next = self.board.clone();
        for rank in 0..8 {
            for file in 0..8 {
                next.set_piece(Position::new(file, rank), None)?;
            }
        }
        for p in &roster {
            if let Some(pos) = p.position {
                if next.piece_at(pos)?.is_some() {
                    return Err(Error::InvalidSetup);
                }
                next.set_piece(
                    pos,
                    Some(Piece {
                        id: p.id,
                        kind: p.kind,
                        owner: PLAYER,
                    }),
                )?;
            }
        }
        self.combat_stats.clear();
        self.wards.clear();
        self.kind_passives.clear();
        self.gambler_dice.clear();
        self.temporary_upgrades.clear();
        self.limiter.clear();
        self.forced_turn_end = false;
        self.self_defense_target = None;
        self.initiative_turns_left = 0;
        self.enemy_initiative_turns_left = 0;
        self.absorbed.clear();
        self.moved_ids = roster.iter().filter(|p| p.moved).map(|p| p.id).collect();
        self.board = next;
        self.players = roster;
        self.clear_fallout();
        self.initialize_statuses();
        Ok(())
    }
    /// Change a permanent kind without moving the piece or resetting combat state.
    pub(crate) fn set_permanent_upgrade(
        &mut self,
        id: PieceId,
        kind: PieceType,
        upgrades: Vec<String>,
        in_combat: bool,
    ) -> Result<(), Error> {
        let player = self
            .players
            .iter()
            .find(|p| p.id == id)
            .ok_or(Error::IllegalAction)?;
        let position = player.position;
        if let Some(at) = position {
            let piece = self.board.piece_at(at)?.ok_or(Error::IllegalAction)?;
            if piece.owner != PLAYER || piece.id != id {
                return Err(Error::IllegalAction);
            }
            self.board.set_piece(at, Some(Piece { kind, ..piece }))?;
        }
        let player = self.players.iter_mut().find(|p| p.id == id).unwrap();
        player.kind = kind;
        player.upgrades = upgrades;
        self.temporary_upgrades.remove(&id);
        if position.is_some() && in_combat {
            self.roll_gambler_die(Piece {
                id,
                kind,
                owner: PLAYER,
            });
        } else {
            self.gambler_dice.remove(&id);
        }
        if let Some(passive) = self.kind_passives.remove(&id) {
            if passive.ward {
                self.wards.remove(&id);
            }
            if let Some(status) = self.mechanics.statuses.get_mut(&id) {
                let removed = status.agility.min(passive.agility);
                status.agility -= removed;
                if position.is_some() && self.phase == Phase::Active && self.turn == PLAYER {
                    self.actions_left = self.actions_left.saturating_sub(removed);
                }
            }
        }
        if position.is_some() && in_combat && self.phase == Phase::Active {
            self.apply_kind_passive(Piece {
                id,
                kind,
                owner: PLAYER,
            });
        }
        Ok(())
    }
    pub fn player_pieces(&self) -> &[PlayerPiece] {
        &self.players
    }
    pub fn has_temporary_upgrade(&self, id: PieceId) -> bool {
        self.temporary_upgrades.contains(&id)
    }
    pub fn absorbed_kinds(&self, id: PieceId) -> Vec<PieceType> {
        let mut kinds: Vec<_> = self
            .absorbed
            .get(&id)
            .into_iter()
            .flatten()
            .copied()
            .collect();
        kinds.sort_unstable();
        kinds
    }
    pub fn outcome(&self) -> Vec<PlayerPiece> {
        self.persistent_outcome()
    }
    pub fn legal_actions(&self, from: Position) -> Result<Vec<DefaultAction>, Error> {
        self.actions_from(from, true)
    }
    pub fn gambler_die(&self, id: PieceId) -> Option<u8> {
        self.piece_position(id)
            .filter(|(_, piece)| matches!(piece.kind, pieces::GAMBLER | pieces::JOCKEY))
            .and_then(|_| self.gambler_dice.get(&id).copied())
    }
    fn roll_gambler_die(&mut self, piece: Piece) {
        if !matches!(piece.kind, pieces::GAMBLER | pieces::JOCKEY) {
            self.gambler_dice.remove(&piece.id);
            return;
        }
        let die = (self.mechanics.rng.index(6) + 1) as u8;
        self.gambler_dice.insert(piece.id, die);
        let kind = [
            pieces::PAWN,
            pieces::KNIGHT,
            pieces::BISHOP,
            pieces::ROOK,
            pieces::QUEEN,
            pieces::KING,
        ][usize::from(die - 1)];
        self.events.push(format!(
            "{}: 주사위 {} · {} 행마",
            piece_name(piece.kind),
            die,
            piece_name(kind)
        ));
    }
    fn roll_all_gambler_dice(&mut self) {
        let pieces: Vec<_> = self.board.cells().iter().flatten().copied().collect();
        for piece in pieces {
            if matches!(piece.kind, pieces::GAMBLER | pieces::JOCKEY) {
                self.roll_gambler_die(piece);
            }
        }
    }
    fn roll_current_side_gambler_dice(&mut self) {
        let pieces: Vec<_> = self
            .board
            .cells()
            .iter()
            .flatten()
            .copied()
            .filter(|p| p.owner == self.turn)
            .collect();
        for piece in pieces {
            if matches!(piece.kind, pieces::GAMBLER | pieces::JOCKEY) {
                self.roll_gambler_die(piece);
            }
        }
    }
    // Inspect each step on its actual intermediate board, including remote/jump captures.
    pub fn apply_piece_action(&mut self, action: &DefaultAction) -> Result<(), Error> {
        if self.phase != Phase::Active {
            return Err(Error::Finished);
        }
        let piece = self
            .board
            .piece_at(action.first.from)?
            .ok_or(Error::IllegalAction)?;
        if piece.owner != self.turn
            && !(self.card_movement && self.turn == PLAYER && self.player_controls(piece))
        {
            return Err(Error::WrongTurn);
        }
        if !self.legal_actions(action.first.from)?.contains(action) {
            return Err(Error::IllegalAction);
        }
        self.apply_piece_action_unchecked(action)
    }
    pub(crate) fn apply_piece_action_unchecked(
        &mut self,
        action: &DefaultAction,
    ) -> Result<(), Error> {
        if self.phase != Phase::Active {
            return Err(Error::Finished);
        }
        let piece = self
            .board
            .piece_at(action.first.from)?
            .ok_or(Error::IllegalAction)?;
        if piece.owner != self.turn
            && !(self.card_movement && self.turn == PLAYER && self.player_controls(piece))
        {
            return Err(Error::WrongTurn);
        }
        let boss_actor = self.is_boss_actor(piece);
        if boss_actor && !self.boss_action_available(action) {
            return Err(Error::IllegalAction);
        }
        // A ward cancels the whole capture action; the attack still spends one action.
        // Shooting spends ammunition and rolls accuracy before a ward can absorb a hit.
        let shooting = action.continuation.is_empty()
            && action.first.effect == crate::Effect::Catch
            && action.first.move_to == action.first.from
            && (action.first.transition.as_deref() == Some("boss_shot")
                || self
                    .hit_chance(action.first.from, action.first.take)
                    .is_some()
                || action
                    .first
                    .transition
                    .as_deref()
                    .and_then(|tag| tag.strip_prefix("boss_copy:"))
                    .and_then(|tag| tag.split_once(':'))
                    .and_then(|(kind, _)| kind.parse::<PieceType>().ok())
                    .and_then(|kind| {
                        self.boss_copied_hit_chance(kind, action.first.from, action.first.take)
                    })
                    .is_some());
        let victim = if shooting {
            None
        } else {
            self.warded_victim(action, piece.owner)?
        };
        if let Some(victim) = victim {
            self.wards.remove(&victim.id);
            self.combat_events.push(CombatEvent::StatusTriggered {
                piece_id: victim.id,
                at: action.first.take,
                status: "ward",
            });
            self.events.push(format!(
                "가호: {}의 포획을 한 번 막았습니다.",
                piece_name(victim.kind)
            ));
        } else {
            self.apply_effect(action, ActionSource::PieceMove)?;
            self.check_king_goal(piece, action.first.from);
        }
        if piece.owner == ENEMY {
            self.en_passant = if self.matches_kind(piece, PAWN)
                && (action.first.from.rank - action.first.move_to.rank).abs() == 2
            {
                Some((
                    piece.id,
                    Position::new(
                        action.first.from.file,
                        (action.first.from.rank + action.first.move_to.rank) / 2,
                    ),
                ))
            } else {
                None
            };
        }
        if boss_actor {
            self.spend_boss_action(action);
        }
        self.forced_turn_end |= shooting && !boss_actor && !self.card_movement;
        if self.card_movement && self.turn == PLAYER && action.first.effect == crate::Effect::Catch
        {
            self.complete_nonmovement_action(piece)
        } else {
            self.complete_piece_action(piece)
        }
    }
    pub(super) fn complete_piece_action(&mut self, piece: Piece) -> Result<(), Error> {
        self.complete_action(piece, true)
    }
    fn complete_nonmovement_action(&mut self, piece: Piece) -> Result<(), Error> {
        self.complete_action(piece, false)
    }
    fn complete_action(&mut self, piece: Piece, moved: bool) -> Result<(), Error> {
        self.mechanics.statuses.entry(piece.id).or_default().dining = false;
        if moved {
            self.moved_ids.insert(piece.id);
        }
        if self.card_movement && self.turn == PLAYER {
            if moved {
                self.stats_mut(piece.id).move_rights =
                    self.stats(piece.id).move_rights.saturating_sub(1);
            } else {
                self.normal_actions_used = self.normal_actions_used.saturating_add(1);
            }
            self.forced_turn_end = false;
            return Ok(());
        }
        let limiter_expired = if let Some(state) = self.limiter.get_mut(&piece.id) {
            state.moves_left = state.moves_left.saturating_sub(1);
            state.moves_left == 0
        } else {
            if self.normal_actions_used >= self.side_actions(piece.owner)
                && self.agility(piece.id) > 0
            {
                self.mechanics.statuses.entry(piece.id).or_default().agility -= 1;
            } else {
                self.normal_actions_used = self.normal_actions_used.saturating_add(1);
            }
            false
        };
        if moved {
            if let Some(p) = self.players.iter_mut().find(|p| p.id == piece.id) {
                p.moved = true;
            }
        }
        self.actions_left = self.actions_left.saturating_sub(1);
        if self.is_boss_actor(piece) && self.boss_actions_spent() {
            self.forced_turn_end = true;
        }
        if limiter_expired && self.phase == Phase::Active {
            self.remove_for_limiter(&[piece.id])?;
            self.events
                .push("리미터 해제: 이동 횟수를 모두 사용해 기물이 사망했습니다.".into());
            if self.phase == Phase::Active {
                self.forced_turn_end = true;
                self.finish_forced_turn()?;
            }
        } else {
            let side = self.turn;
            self.finish_forced_turn()?;
            if self.turn == side
                && self.phase == Phase::Active
                && self.actions_left == 0
                && self.pending_choice().is_none()
            {
                self.end_turn()?;
            }
        }
        Ok(())
    }
    pub fn end_turn(&mut self) -> Result<(), Error> {
        if self.pending_choice().is_some() {
            return Err(Error::IllegalAction);
        }
        if self.phase != Phase::Active {
            return Err(Error::Finished);
        }
        self.forced_turn_end = false;
        if self.turn == PLAYER {
            self.en_passant = None;
        }
        if self.turn == ENEMY && self.initiative_turns_left > 0 {
            self.initiative_turns_left -= 1;
            if self.initiative_turns_left == 0 {
                self.events
                    .push("선공권: 적 턴 10회가 지나 보호가 끝났습니다.".into());
            }
        }
        if self.turn == PLAYER && self.enemy_initiative_turns_left > 0 {
            self.enemy_initiative_turns_left -= 1;
            if self.enemy_initiative_turns_left == 0 {
                self.events
                    .push("선공권: 아군 턴 10회가 지나 적의 보호가 끝났습니다.".into());
            }
        }
        self.finish_stats_turn()?;
        self.finish_mechanics_turn()?;
        if self.phase != Phase::Active {
            return Ok(());
        }
        self.turn = if self.turn == PLAYER { ENEMY } else { PLAYER };
        self.start_stats_turn()?;
        self.start_mechanics_turn()?;
        if self.phase != Phase::Active {
            return Ok(());
        }
        self.roll_current_side_gambler_dice();
        self.forced_turn_end = false;
        self.reset_turn_budget();
        Ok(())
    }
    pub(crate) fn finish_forced_turn(&mut self) -> Result<(), Error> {
        if self.forced_turn_end && self.phase == Phase::Active && self.pending_choice().is_none() {
            self.end_turn()?;
        }
        Ok(())
    }
    pub fn limiter_status(&self, id: PieceId) -> Option<(bool, u8)> {
        self.limiter.get(&id).map(|s| (true, s.moves_left))
    }
}
fn push_unique_action(
    result: &mut Vec<DefaultAction>,
    seen: &mut Option<HashMap<u64, Vec<usize>>>,
    action: DefaultAction,
) {
    // Small move lists are cheaper to scan. Build the index only for crowded positions.
    if result.len() < 16 {
        if !result.contains(&action) {
            result.push(action);
        }
        return;
    }
    let seen = seen.get_or_insert_with(|| {
        let mut index: HashMap<u64, Vec<usize>> = HashMap::new();
        for (i, existing) in result.iter().enumerate() {
            index.entry(action_hash(existing)).or_default().push(i);
        }
        index
    });
    let hash = action_hash(&action);
    let indices = seen.entry(hash).or_default();
    if indices.iter().any(|&i| result[i] == action) {
        return;
    }
    indices.push(result.len());
    result.push(action);
}
fn action_hash(action: &DefaultAction) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    for step in action.steps() {
        step.from.hash(&mut hasher);
        step.take.hash(&mut hasher);
        step.move_to.hash(&mut hasher);
        std::mem::discriminant(&step.effect).hash(&mut hasher);
        step.transition.hash(&mut hasher);
    }
    hasher.finish()
}
fn valid_kind(k: PieceType) -> bool {
    pieces::definition(k).is_some()
}
pub fn square(p: Position) -> String {
    format!("{}{}", (b'a' + p.file as u8) as char, 8 - p.rank)
}

#[cfg(test)]
mod dedupe_tests {
    use super::*;

    #[test]
    fn keeps_first_route_and_distinct_transition_after_indexing() {
        let mut result = Vec::new();
        let mut seen = None;
        for file in 0..16 {
            let mut action = DefaultAction::one(crate::ActionStep::new(
                Position::new(file, 0),
                Position::new(file, 1),
                Position::new(file, 1),
                crate::Effect::Move,
            ));
            action.route = Some(vec![Position::new(file, 1)]);
            push_unique_action(&mut result, &mut seen, action);
        }
        let mut duplicate = result[0].clone();
        duplicate.route = Some(vec![Position::new(7, 7)]);
        push_unique_action(&mut result, &mut seen, duplicate.clone());
        assert_eq!(result.len(), 16);
        assert_eq!(result[0].route, Some(vec![Position::new(0, 1)]));
        duplicate.first.transition = Some("queen".into());
        push_unique_action(&mut result, &mut seen, duplicate);
        assert_eq!(result.len(), 17);
    }
}
