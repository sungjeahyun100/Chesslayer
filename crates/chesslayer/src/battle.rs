//! Chesslayer battle rules and persistent player piece state.
use crate::{
    apply_default_action, ActionError, Board, BoardError, DefaultAction, GameResult, Piece,
    PieceId, PieceType, PlayerId, Position, ResultReason,
};
use chessembly::{Error as DslError, Program};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::sync::Arc;
mod boss;
mod mechanics;
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
    pub actions_per_turn: u8,
    pub self_defense: bool,
    pub initiative: bool,
    pub enemy_initiative: bool,
}
impl Default for BattleOptions {
    fn default() -> Self {
        Self {
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
    pub turn: PlayerId,
    pub result: GameResult,
    pub phase: Phase,
    pub royal_id: PieceId,
    additional_royal_ids: Vec<PieceId>,
    pub king_id: PieceId,
    pub actions_left: u8,
    normal_actions_used: u8,
    limiter_turn: bool,
    pub events: Vec<String>,
    actions_per_turn: u8,
    wards: HashSet<PieceId>,
    temporary_upgrades: HashSet<PieceId>,
    limiter: HashMap<PieceId, LimiterState>,
    self_defense_target: Option<PieceId>,
    initiative_turns_left: u8,
    enemy_initiative_turns_left: u8,
    absorbed: HashMap<PieceId, HashSet<PieceType>>,
    boss: Option<BossState>,
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
impl Battle {
    pub(crate) fn force_search_shot(&mut self, hit: bool) {
        self.mechanics.force_search_shot(hit);
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
        self.turn.hash(&mut h);
        self.actions_left.hash(&mut h);
        self.normal_actions_used.hash(&mut h);
        self.limiter_turn.hash(&mut h);
        self.initiative_turns_left.hash(&mut h);
        self.enemy_initiative_turns_left.hash(&mut h);
        self.self_defense_target.hash(&mut h);
        if let Some(boss) = &self.boss {
            boss.hp.hash(&mut h);
            boss.ammo.hash(&mut h);
            boss.hook.hash(&mut h);
        }
        for set in [&self.wards, &self.temporary_upgrades, &self.moved_ids] {
            let mut ids: Vec<_> = set.iter().collect();
            ids.sort();
            ids.hash(&mut h);
        }
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
            if pieces::is_royal(p.kind) && king_id.replace(p.id).is_some() {
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
            events,
            wards: HashSet::new(),
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
            boss: start
                .encounter
                .enemies
                .iter()
                .any(|p| p.id == start.encounter.royal_id && p.kind == pieces::TRIBAL_DICTATOR)
                .then_some(BossState {
                    hp: 5,
                    ammo: 2,
                    hook: None,
                }),
            actions_left: options.actions_per_turn,
            normal_actions_used: 0,
            limiter_turn: false,
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
        battle.initialize_statuses();
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
        self.moved_ids.remove(&piece.id);
        self.wards.remove(&piece.id);
        self.limiter.remove(&piece.id);
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
        self.actions_left = self.side_actions(self.turn);
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
        self.wards.clear();
        self.temporary_upgrades.clear();
        self.limiter.clear();
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
    pub(crate) fn generated_legal_actions(
        &self,
        from: Position,
    ) -> Result<Vec<GeneratedAction<'_>>, Error> {
        Ok(self
            .legal_actions(from)?
            .into_iter()
            .map(|action| GeneratedAction {
                battle: self,
                action,
            })
            .collect())
    }
    /// Selection and application share one exclusive borrow, so the board and turn cannot
    /// change between generation and application. Only this crate can use the unchecked path.
    pub(crate) fn select_and_apply<F>(
        &mut self,
        select: F,
    ) -> Result<Option<(DefaultAction, Piece, Option<Piece>)>, Error>
    where
        F: for<'a> FnOnce(&'a Battle) -> Result<Option<GeneratedAction<'a>>, Error>,
    {
        let Some(selected) = select(self)? else {
            return Ok(None);
        };
        if !std::ptr::eq(selected.battle, self) {
            return Err(Error::IllegalAction);
        }
        let action = selected.action;
        let piece = self
            .board
            .piece_at(action.first.from)?
            .ok_or(Error::IllegalAction)?;
        let captured = action.steps().find_map(|s| {
            self.board
                .piece_at(s.take)
                .ok()
                .flatten()
                .filter(|p| p.owner != piece.owner)
        });
        self.apply_piece_action_unchecked(&action)?;
        Ok(Some((action, piece, captured)))
    }
    /// Inspect another side's current movement without granting it a turn or an action.
    pub fn preview_actions(&self, from: Position) -> Result<Vec<ActionPreview>, Error> {
        let actions = self.actions_from(from, false)?;
        let owner = self
            .board
            .piece_at(from)?
            .ok_or(Error::IllegalAction)?
            .owner;
        actions
            .into_iter()
            .map(|action| {
                let destination = action.steps().last().unwrap().move_to;
                let mut captures = Vec::new();
                if action.continuation.is_empty() {
                    let step = &action.first;
                    if !matches!(step.effect, crate::Effect::Move | crate::Effect::Shift)
                        && self
                            .board
                            .piece_at(step.take)?
                            .is_some_and(|p| p.owner != owner)
                    {
                        captures.push(step.take);
                    }
                } else {
                    let mut board = self.board.clone();
                    for step in action.steps() {
                        if !matches!(step.effect, crate::Effect::Move | crate::Effect::Shift)
                            && board.piece_at(step.take)?.is_some_and(|p| p.owner != owner)
                        {
                            captures.push(step.take);
                        }
                        apply_default_action(&mut board, &DefaultAction::one(step.clone()))?;
                    }
                }
                Ok(ActionPreview {
                    destination,
                    captures,
                    route: action.route,
                })
            })
            .collect()
    }
    fn actions_from(
        &self,
        from: Position,
        enforce_turn: bool,
    ) -> Result<Vec<DefaultAction>, Error> {
        if self.phase != Phase::Active {
            return Err(Error::Finished);
        }
        let piece = self.board.piece_at(from)?.ok_or(Error::IllegalAction)?;
        if enforce_turn && piece.owner != self.turn {
            return Err(Error::WrongTurn);
        }
        if (enforce_turn && self.pending_choice().is_some()) || !self.can_act(piece) {
            return Ok(vec![]);
        }
        if enforce_turn
            && self.normal_actions_used >= self.side_actions(piece.owner)
            && self.agility(piece.id) == 0
            && !self.limiter.contains_key(&piece.id)
        {
            return Ok(vec![]);
        }
        if self
            .limiter
            .get(&piece.id)
            .is_some_and(|s| s.moves_left == 0)
        {
            return Ok(vec![]);
        }
        if enforce_turn
            && self.limiter_turn
            && !self.limiter.contains_key(&piece.id)
            && self.agility(piece.id) == 0
            && self.normal_actions_used >= self.actions_per_turn
        {
            return Ok(vec![]);
        }
        let mut actions = self.piece_actions(piece.kind, from)?;
        actions.extend(self.special_actions(from));
        if piece.owner == PLAYER
            && self.turn == PLAYER
            && self.mechanics.reckless.contains(&piece.id)
        {
            let extra: Vec<_> = actions
                .iter()
                .filter_map(|a| {
                    if !a.continuation.is_empty() || a.first.effect != crate::Effect::Move {
                        return None;
                    }
                    let one = a.first.move_to;
                    let dx = one.file - from.file;
                    let dy = one.rank - from.rank;
                    if dx.abs().max(dy.abs()) != 1 {
                        return None;
                    }
                    let to = Position::new(one.file + dx, one.rank + dy);
                    if !self.board.contains(to)
                        || self.board.piece_at(to).ok().flatten().is_some()
                        || self.board.piece_at(one).ok().flatten().is_some()
                    {
                        return None;
                    }
                    Some(DefaultAction::one(crate::ActionStep::new(
                        from,
                        to,
                        to,
                        crate::Effect::Move,
                    )))
                })
                .collect();
            actions.extend(extra);
        }
        if piece.kind == pieces::EMPEROR || piece.kind == pieces::TRIBAL_DICTATOR {
            if let Some(absorbed) = self.absorbed.get(&piece.id) {
                let mut kinds: Vec<_> = absorbed.iter().copied().collect();
                kinds.sort_unstable();
                for kind in kinds {
                    if piece.kind == pieces::TRIBAL_DICTATOR {
                        for mut action in self.boss_copied_movement_actions(kind, from)? {
                            let original = action.first.transition.take().unwrap_or_default();
                            action.first.transition = Some(format!("boss_copy:{kind}:{original}"));
                            actions.push(action);
                        }
                    } else {
                        actions.extend(self.piece_actions(kind, from)?);
                    }
                }
            }
        }
        if pieces::definition(piece.kind).is_some_and(|d| d.initial_double_step)
            && !self.moved_ids.contains(&piece.id)
            && from.rank == if piece.owner == PLAYER { 6 } else { 1 }
        {
            let double = self.initial_double_step.execute(&self.board, from)?;
            if let Some(second) = double.get(1) {
                actions.push(second.clone())
            }
        }
        if self.mechanics.suppressed.contains(&piece.id) {
            actions.retain(|a| a.steps().last().is_some_and(|s| s.move_to == from));
        }
        let definition = pieces::definition(piece.kind).ok_or(Error::InvalidSetup)?;
        let last_rank = if piece.owner == PLAYER { 0 } else { 7 };
        // A promotion is part of the validated move, including captures of the royal.
        // No unpromoted action is offered on the final rank for these families.
        let mut result = Vec::new();
        let mut seen: Option<HashMap<u64, Vec<usize>>> = None;
        for action in actions {
            if piece.owner == ENEMY
                && self.initiative_turns_left > 0
                && self.captures_player(&action)?
            {
                continue;
            }
            if piece.owner == PLAYER
                && self.enemy_initiative_turns_left > 0
                && self.captures_enemy(&action)?
            {
                continue;
            }
            let destination = action.steps().last().unwrap().move_to;
            if destination.rank == last_rank && !definition.promotions.is_empty() {
                for &kind in definition.promotions {
                    let mut promoted = action.clone();
                    let step = promoted
                        .continuation
                        .last_mut()
                        .unwrap_or(&mut promoted.first);
                    step.transition = Some(pieces::definition(kind).unwrap().key.into());
                    push_unique_action(&mut result, &mut seen, promoted);
                }
            } else {
                push_unique_action(&mut result, &mut seen, action);
            }
        }
        self.filter_hunger(piece, &mut result);
        Ok(result)
    }
    fn piece_actions(&self, kind: PieceType, from: Position) -> Result<Vec<DefaultAction>, Error> {
        let definition = pieces::definition(kind).ok_or(Error::InvalidSetup)?;
        if let Some(calculate) = definition.native_movement {
            Ok(calculate(&self.board, from)?)
        } else {
            Ok(self
                .programs
                .get(&kind)
                .ok_or(Error::InvalidSetup)?
                .execute(&self.board, from)?)
        }
    }
    // Inspect each step on its actual intermediate board, including remote/jump captures.
    fn captures_player(&self, action: &DefaultAction) -> Result<bool, Error> {
        self.captures_owner(action, PLAYER)
    }
    fn captures_enemy(&self, action: &DefaultAction) -> Result<bool, Error> {
        self.captures_owner(action, ENEMY)
    }
    fn captures_owner(&self, action: &DefaultAction, owner: PlayerId) -> Result<bool, Error> {
        if action.continuation.is_empty() {
            let step = &action.first;
            return Ok(
                !matches!(step.effect, crate::Effect::Move | crate::Effect::Shift)
                    && self
                        .board
                        .piece_at(step.take)?
                        .is_some_and(|p| p.owner == owner),
            );
        }
        let mut next = self.board.clone();
        for step in action.steps() {
            if !matches!(step.effect, crate::Effect::Move | crate::Effect::Shift)
                && next.piece_at(step.take)?.is_some_and(|p| p.owner == owner)
            {
                return Ok(true);
            }
            apply_default_action(&mut next, &DefaultAction::one(step.clone()))?;
        }
        Ok(false)
    }
    pub fn apply_piece_action(&mut self, action: &DefaultAction) -> Result<(), Error> {
        if self.phase != Phase::Active {
            return Err(Error::Finished);
        }
        let piece = self
            .board
            .piece_at(action.first.from)?
            .ok_or(Error::IllegalAction)?;
        if piece.owner != self.turn {
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
        if piece.owner != self.turn {
            return Err(Error::WrongTurn);
        }
        // A ward cancels the whole capture action; the attack still spends one action.
        // Shooting spends ammunition and rolls accuracy before a ward can absorb a hit.
        let shooting = action.continuation.is_empty()
            && action.first.effect == crate::Effect::Catch
            && (self
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
            self.events.push(format!(
                "가호: {}의 포획을 한 번 막았습니다.",
                piece_name(victim.kind)
            ));
        } else {
            self.apply_effect(action, ActionSource::PieceMove)?;
        }
        self.complete_piece_action(piece)
    }
    pub(super) fn complete_piece_action(&mut self, piece: Piece) -> Result<(), Error> {
        self.mechanics.statuses.entry(piece.id).or_default().dining = false;
        self.moved_ids.insert(piece.id);
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
        if let Some(p) = self.players.iter_mut().find(|p| p.id == piece.id) {
            p.moved = true;
        }
        self.actions_left = self.actions_left.saturating_sub(1);
        if limiter_expired && self.phase == Phase::Active {
            self.remove_for_limiter(&[piece.id])?;
            self.events
                .push("리미터 해제: 이동 횟수를 모두 사용해 기물이 사망했습니다.".into());
            if self.phase == Phase::Active {
                self.end_turn()?;
            }
        } else if self.phase == Phase::Active
            && self.actions_left == 0
            && self.pending_choice().is_none()
        {
            self.end_turn()?;
        }
        Ok(())
    }
    fn warded_victim(
        &self,
        action: &DefaultAction,
        owner: PlayerId,
    ) -> Result<Option<Piece>, Error> {
        if action.continuation.is_empty() {
            let step = &action.first;
            return Ok(
                if matches!(step.effect, crate::Effect::Shift | crate::Effect::Move) {
                    None
                } else {
                    self.board
                        .piece_at(step.take)?
                        .filter(|p| p.owner != owner && self.wards.contains(&p.id))
                },
            );
        }
        let mut board = self.board.clone();
        for step in action.steps() {
            if !matches!(step.effect, crate::Effect::Shift | crate::Effect::Move) {
                if let Some(victim) = board
                    .piece_at(step.take)?
                    .filter(|p| p.owner != owner && self.wards.contains(&p.id))
                {
                    return Ok(Some(victim));
                }
            }
            apply_default_action(&mut board, &DefaultAction::one(step.clone()))?;
        }
        Ok(None)
    }
    pub fn end_turn(&mut self) -> Result<(), Error> {
        if self.pending_choice().is_some() {
            return Err(Error::IllegalAction);
        }
        if self.phase != Phase::Active {
            return Err(Error::Finished);
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
        self.finish_mechanics_turn()?;
        if self.phase != Phase::Active {
            return Ok(());
        }
        self.turn = if self.turn == PLAYER { ENEMY } else { PLAYER };
        self.start_mechanics_turn()?;
        if self.phase != Phase::Active {
            return Ok(());
        }
        self.reset_turn_budget();
        Ok(())
    }
    pub fn limiter_status(&self, id: PieceId) -> Option<(bool, u8)> {
        self.limiter.get(&id).map(|s| (true, s.moves_left))
    }
    pub fn grant_limiter(&mut self, id: PieceId) -> Result<(), Error> {
        if self.phase != Phase::Active {
            return Err(Error::Finished);
        }
        if self.limiter.contains_key(&id) {
            return Err(Error::IllegalAction);
        }
        let piece = self
            .board
            .cells()
            .iter()
            .flatten()
            .find(|p| p.id == id)
            .ok_or(Error::IllegalAction)?;
        let owner = piece.owner;
        self.limiter.insert(
            id,
            LimiterState {
                owner,
                moves_left: 3,
            },
        );
        if owner == self.turn {
            self.limiter_turn = true;
            self.actions_left = self.actions_left.saturating_add(3);
        }
        Ok(())
    }
    pub fn forward_pawn_candidates(&self) -> Vec<PieceId> {
        self.board
            .cells()
            .iter()
            .enumerate()
            .filter_map(|(i, p)| {
                let piece = p.as_ref()?;
                if piece.owner != PLAYER || !piece_name(piece.kind).contains("폰") {
                    return None;
                }
                let from = Position::new(i as i32 % 8, i as i32 / 8);
                let to = Position::new(from.file, from.rank - 1);
                self.board
                    .index(to)
                    .filter(|_| self.board.piece_at(to).ok().flatten().is_none())
                    .map(|_| piece.id)
            })
            .collect()
    }
    pub fn advance_pawns(&mut self, ids: &[PieceId]) -> Result<(), Error> {
        if self.phase != Phase::Active || ids.is_empty() {
            return Err(Error::IllegalAction);
        }
        let eligible = self.forward_pawn_candidates();
        let mut seen = HashSet::new();
        if ids
            .iter()
            .any(|id| !seen.insert(*id) || !eligible.contains(id))
        {
            return Err(Error::IllegalAction);
        }
        let moves: Vec<_> = self
            .board
            .cells()
            .iter()
            .enumerate()
            .filter_map(|(i, p)| {
                let piece = p.as_ref()?;
                ids.contains(&piece.id).then(|| {
                    let from = Position::new(i as i32 % 8, i as i32 / 8);
                    (from, Position::new(from.file, from.rank - 1), *piece)
                })
            })
            .collect();
        let mut destinations = HashSet::new();
        if moves.len() != ids.len() || moves.iter().any(|(_, to, _)| !destinations.insert(*to)) {
            return Err(Error::IllegalAction);
        }
        let mut next = self.board.clone();
        for (from, _, _) in &moves {
            next.set_piece(*from, None)?;
        }
        for (_, to, piece) in &moves {
            next.set_piece(*to, Some(*piece))?;
        }
        self.board = next;
        self.sync_players();
        Ok(())
    }
    pub fn temporary_upgrades(&mut self, choices: &[(PieceId, PieceType)]) -> Result<(), Error> {
        if self.phase != Phase::Active || choices.is_empty() {
            return Err(Error::IllegalAction);
        }
        let mut seen = HashSet::new();
        let mut changes = vec![];
        for &(id, target) in choices {
            if !seen.insert(id) {
                return Err(Error::IllegalAction);
            }
            let (i, piece) = self
                .board
                .cells()
                .iter()
                .enumerate()
                .find_map(|(i, p)| p.filter(|p| p.id == id).map(|p| (i, p)))
                .ok_or(Error::IllegalAction)?;
            if piece.owner != PLAYER
                || !pieces::definition(piece.kind)
                    .is_some_and(|d| d.upgrades.iter().any(|u| u.target == target))
                || !pieces::required_owned(target)
                    .iter()
                    .all(|&kind| self.players.iter().any(|owned| owned.kind == kind))
            {
                return Err(Error::IllegalAction);
            }
            changes.push((
                Position::new(i as i32 % 8, i as i32 / 8),
                Piece {
                    kind: target,
                    ..piece
                },
            ));
        }
        let mut next = self.board.clone();
        for (at, piece) in changes {
            next.set_piece(at, Some(piece))?;
        }
        self.board = next;
        self.temporary_upgrades
            .extend(choices.iter().map(|(id, _)| *id));
        Ok(())
    }
    /// Applies a resolved card action atomically, preserving both royal identities and the turn.
    pub fn apply_resolved_card_effect(&mut self, action: &DefaultAction) -> Result<(), Error> {
        if self.phase != Phase::Active {
            return Err(Error::Finished);
        }
        let mut next = self.clone();
        next.apply_effect(action, ActionSource::ResolvedCardEffect)?;
        *self = next;
        Ok(())
    }
    pub fn has_ward(&self, id: PieceId) -> bool {
        self.wards.contains(&id)
    }
    pub fn has_self_defense(&self, id: PieceId) -> bool {
        self.self_defense_target == Some(id)
    }
    pub fn initiative_turns_left(&self) -> u8 {
        self.initiative_turns_left
    }
    pub fn enemy_initiative_turns_left(&self) -> u8 {
        self.enemy_initiative_turns_left
    }
    pub fn is_enemy_royal(&self, id: PieceId) -> bool {
        id == self.royal_id || self.additional_royal_ids.contains(&id)
    }
    pub fn grant_ward(&mut self, id: PieceId) -> Result<(), Error> {
        if self.phase != Phase::Active {
            return Err(Error::Finished);
        }
        if !self.board.cells().iter().flatten().any(|p| p.id == id) {
            return Err(Error::IllegalAction);
        }
        self.wards.insert(id);
        Ok(())
    }
    /// Non-capture removal always protects both royals. Counterattacks only occur in setup.
    pub fn remove_nonroyals(&mut self, ids: &[PieceId]) -> Result<usize, Error> {
        self.remove_pieces(ids, false)
    }
    fn remove_for_limiter(&mut self, ids: &[PieceId]) -> Result<(), Error> {
        self.remove_pieces(ids, true)?;
        self.update_result(ResultReason::RoyalCaptured);
        Ok(())
    }
    fn remove_pieces(&mut self, ids: &[PieceId], include_royals: bool) -> Result<usize, Error> {
        if self.phase != Phase::Active {
            return Err(Error::Finished);
        }
        let mut removed = 0;
        for rank in 0..8 {
            for file in 0..8 {
                let pos = Position::new(file, rank);
                if let Some(piece) = self.board.piece_at(pos)? {
                    if ids.contains(&piece.id)
                        && (include_royals
                            || (piece.id != self.king_id && !self.is_enemy_royal(piece.id))
                            || (self.boss.is_some() && piece.id == self.royal_id))
                    {
                        if !self.boss_hit(piece, None) {
                            continue;
                        }
                        self.board.set_piece(pos, None)?;
                        self.boss_on_death(piece, pos);
                        self.wards.remove(&piece.id);
                        self.limiter.remove(&piece.id);
                        removed += 1;
                    }
                }
            }
        }
        self.sync_players();
        if self.boss.is_some() {
            self.update_result(ResultReason::RoyalCaptured);
        }
        Ok(removed)
    }
    fn apply_effect(&mut self, action: &DefaultAction, source: ActionSource) -> Result<(), Error> {
        if source == ActionSource::PieceMove
            && action
                .steps()
                .last()
                .is_some_and(|s| s.move_to != action.first.from)
        {
            if let Some(p) = self.board.piece_at(action.first.from)? {
                self.mechanics.turn_moved.insert(p.id);
            }
        }
        if source == ActionSource::PieceMove && self.apply_mechanics_action(action)? {
            return Ok(());
        }
        // Card effects need royal-protection rollback; compound moves need a staging board
        // while rule effects inspect each intermediate position. A single piece step can
        // use chessembly's changed-square rollback directly on the live board.
        let mut staged = (source == ActionSource::ResolvedCardEffect
            || !action.continuation.is_empty()
            || self.boss.is_some())
        .then(|| self.board.clone());
        let next = staged.as_mut().unwrap_or(&mut self.board);
        let mut retaliation = None;
        let mut player_captured = false;
        let mut enemy_captured = false;
        let mut absorbed = vec![];
        let mut captures = vec![];
        let self_defense_target = self.self_defense_target;
        if source == ActionSource::ResolvedCardEffect {
            for step in action.steps() {
                let victim = next.piece_at(step.take)?;
                let attacker = next
                    .piece_at(step.from)?
                    .filter(|p| p.owner == PLAYER)
                    .map(|p| p.id);
                if !matches!(step.effect, crate::Effect::Move | crate::Effect::Shift)
                    && victim.is_some_and(|p| p.kind == pieces::TRIBAL_DICTATOR)
                    && !boss::hit_boss(
                        &mut self.boss,
                        &mut self.events,
                        next,
                        victim.unwrap(),
                        attacker,
                    )
                {
                    continue;
                }
                apply_default_action(next, &DefaultAction::one(step.clone()))?;
                if let Some(victim) = victim {
                    if !matches!(step.effect, crate::Effect::Move | crate::Effect::Shift)
                        && next.cells().iter().flatten().all(|p| p.id != victim.id)
                    {
                        boss::death_near_boss(
                            &mut self.boss,
                            &mut self.absorbed,
                            &mut self.events,
                            self.royal_id,
                            next,
                            victim,
                            step.take,
                        );
                    }
                }
            }
            if !self.sandbox {
                for id in std::iter::once(self.king_id)
                    .chain(std::iter::once(self.royal_id))
                    .chain(self.additional_royal_ids.iter().copied())
                {
                    if !next.cells().iter().flatten().any(|p| p.id == id)
                        && !(id == self.royal_id && self.boss.as_ref().is_some_and(|b| b.hp <= 0))
                    {
                        return Err(Error::ProtectedRoyal);
                    }
                }
            }
        } else {
            for step in action.steps() {
                // Resolve the actual captor and victim at this step, including multi-step moves.
                let captor = next.piece_at(step.from)?;
                let victim = next.piece_at(step.take)?;
                if captor.is_some_and(|p| p.owner == ENEMY && p.kind == pieces::EMPEROR)
                    && victim.is_some_and(|p| p.owner == PLAYER)
                    && !matches!(step.effect, crate::Effect::Move | crate::Effect::Shift)
                {
                    absorbed.push((captor.unwrap().id, victim.unwrap().kind));
                }
                let retaliates = !matches!(step.effect, crate::Effect::Move | crate::Effect::Shift)
                    && captor.is_some_and(|p| p.owner == ENEMY)
                    && victim
                        .is_some_and(|p| p.owner == PLAYER && self_defense_target == Some(p.id));
                if !matches!(step.effect, crate::Effect::Move | crate::Effect::Shift)
                    && victim.is_some_and(|p| p.kind == pieces::TRIBAL_DICTATOR)
                    && !boss::hit_boss(
                        &mut self.boss,
                        &mut self.events,
                        next,
                        victim.unwrap(),
                        captor.map(|p| p.id),
                    )
                {
                    continue;
                }
                apply_default_action(next, &DefaultAction::one(step.clone()))?;
                if let Some(victim) = victim {
                    if !matches!(step.effect, crate::Effect::Move | crate::Effect::Shift)
                        && next.cells().iter().flatten().all(|p| p.id != victim.id)
                    {
                        boss::death_near_boss(
                            &mut self.boss,
                            &mut self.absorbed,
                            &mut self.events,
                            self.royal_id,
                            next,
                            victim,
                            step.take,
                        );
                    }
                }
                player_captured |=
                    !matches!(step.effect, crate::Effect::Move | crate::Effect::Shift)
                        && captor.is_some_and(|p| p.owner == PLAYER)
                        && victim.is_some_and(|p| p.owner == ENEMY);
                enemy_captured |=
                    !matches!(step.effect, crate::Effect::Move | crate::Effect::Shift)
                        && captor.is_some_and(|p| p.owner == ENEMY)
                        && victim.is_some_and(|p| p.owner == PLAYER);
                if !matches!(step.effect, crate::Effect::Move | crate::Effect::Shift) {
                    if let (Some(captor), Some(victim)) = (captor, victim) {
                        if captor.owner < 2 && victim.owner < 2 && captor.owner != victim.owner {
                            captures.push((captor, victim));
                        }
                    }
                }
                if retaliates {
                    let captor = captor.unwrap();
                    if let Some(index) = next
                        .cells()
                        .iter()
                        .position(|p| p.is_some_and(|p| p.id == captor.id))
                    {
                        let width = next.width();
                        next.set_piece(
                            Position::new((index % width) as i32, (index / width) as i32),
                            None,
                        )
                        .expect("existing board square is always valid");
                    }
                    retaliation = Some(captor);
                    // A dead captor cannot perform any remaining steps of its move.
                    break;
                }
            }
        }
        if let Some(next) = staged {
            self.board = next;
        }
        for (id, kind) in absorbed {
            if self.board.cells().iter().flatten().any(|p| p.id == id) {
                self.absorbed.entry(id).or_default().insert(kind);
            }
        }
        if player_captured {
            self.mechanics.killed_this_turn = true;
        }
        if player_captured && self.initiative_turns_left > 0 {
            self.initiative_turns_left = 0;
            self.events
                .push("선공권: 아군의 포획이 성공해 보호가 끝났습니다.".into());
        }
        if enemy_captured && self.enemy_initiative_turns_left > 0 {
            self.enemy_initiative_turns_left = 0;
            self.events
                .push("선공권: 적의 포획이 성공해 보호가 끝났습니다.".into());
        }
        for (captor, victim) in captures {
            self.record_kill(captor, victim);
        }
        self.wards
            .retain(|id| self.board.cells().iter().flatten().any(|p| p.id == *id));
        self.limiter
            .retain(|id, _| self.board.cells().iter().flatten().any(|p| p.id == *id));
        self.sync_players();
        if let Some(captor) = retaliation {
            self.events.push(format!(
                "호신용품: 보호 대상을 포획한 적 {}도 사망했습니다.",
                piece_name(captor.kind)
            ));
        }
        if source == ActionSource::PieceMove || self.boss.is_some() {
            self.update_result(if retaliation.is_some() {
                ResultReason::SelfDefense
            } else {
                ResultReason::RoyalCaptured
            });
        }
        Ok(())
    }
    fn sync_players(&mut self) {
        let previous: Vec<_> = self.players.iter().map(|p| p.position).collect();
        for p in &mut self.players {
            p.position = None;
        }
        for (index, piece) in self.board.cells().iter().enumerate() {
            if let Some(piece) = piece {
                if piece.owner == PLAYER && piece.kind != pieces::CORPSE {
                    if let Some(p) = self.players.iter_mut().find(|p| p.id == piece.id) {
                        p.position = Some(Position::new(index as i32 % 8, index as i32 / 8));
                    }
                }
            }
        }
        for (piece, before) in self.players.iter_mut().zip(previous) {
            if piece.position != before && piece.position.is_some() {
                piece.moved = true;
                self.moved_ids.insert(piece.id);
                if before.is_some() {
                    self.mechanics.turn_moved.insert(piece.id);
                }
            }
        }
    }
    fn update_result(&mut self, reason: ResultReason) {
        if self.sandbox {
            return;
        }
        let king_alive = if self.mechanics.player_artillery {
            self.board
                .cells()
                .iter()
                .flatten()
                .any(|p| p.owner == PLAYER && p.kind == pieces::WAREHOUSE)
        } else {
            self.players
                .iter()
                .any(|p| p.id == self.king_id && p.position.is_some())
        };
        let royal_alive = self
            .board
            .cells()
            .iter()
            .flatten()
            .any(|p| p.owner == ENEMY && self.is_enemy_royal(p.id));
        self.result = if !king_alive {
            GameResult::Win {
                winner: ENEMY,
                reason,
            }
        } else if !royal_alive {
            GameResult::Win {
                winner: PLAYER,
                reason,
            }
        } else {
            GameResult::Ongoing
        };
        self.phase = if self.result == GameResult::Ongoing {
            Phase::Active
        } else {
            Phase::Finished
        };
        if self.phase == Phase::Finished {
            self.finish_persistent_statuses();
            self.self_defense_target = None;
            self.initiative_turns_left = 0;
            self.enemy_initiative_turns_left = 0;
            self.limiter.clear();
        }
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
