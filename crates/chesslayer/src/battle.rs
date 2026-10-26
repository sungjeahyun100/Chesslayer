//! Chesslayer battle rules and persistent player piece state.
use crate::{
    apply_default_action, ActionError, Board, BoardError, DefaultAction, GameResult, Piece,
    PieceId, PieceType, PlayerId, Position, ResultReason,
};
use chessembly::{Error as DslError, Program};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::hash::{Hash, Hasher};
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
    players: Vec<PlayerPiece>,
    programs: BTreeMap<PieceType, Program>,
    initial_double_step: Program,
    moved_ids: HashSet<PieceId>,
}
#[derive(Clone, Copy)]
struct LimiterState {
    owner: PlayerId,
    active: bool,
    moves_left: u8,
}
impl Battle {
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
        if options.actions_per_turn == 0 {
            return Err(Error::InvalidSetup);
        }
        let mut events = vec![];
        let mut board = Board::new(8, 8)?;
        board.set_catalog(
            pieces::catalog(),
            vec!["player".into(), "enemy".into()],
            vec![1, -1],
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
            if p.kind == KING && king_id.replace(p.id).is_some() {
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
        let king_id = king_id.ok_or(Error::InvalidSetup)?;
        let mut players = start.players;
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
        if found_royals.len() != required_royals.len() {
            return Err(Error::InvalidSetup);
        }
        let programs = pieces::ALL
            .iter()
            .map(|d| Program::parse(&(d.movement)()).map(|program| (d.kind, program)))
            .collect::<Result<_, _>>()?;
        let initial_double_step = Program::parse(pieces::movement::INITIAL_DOUBLE_STEP)?;
        let moved_ids = players.iter().filter(|p| p.moved).map(|p| p.id).collect();
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
            programs,
            initial_double_step,
            moved_ids,
        };
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
        self.players.clone()
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
        if self
            .limiter
            .get(&piece.id)
            .is_some_and(|s| s.active && s.moves_left == 0)
        {
            return Ok(vec![]);
        }
        if enforce_turn
            && self.limiter_turn
            && !self.limiter.get(&piece.id).is_some_and(|s| s.active)
            && self.normal_actions_used >= self.actions_per_turn
        {
            return Ok(vec![]);
        }
        let mut actions = self.piece_actions(piece.kind, from)?;
        if piece.kind == pieces::EMPEROR {
            if let Some(absorbed) = self.absorbed.get(&piece.id) {
                let mut kinds: Vec<_> = absorbed.iter().copied().collect();
                kinds.sort_unstable();
                for kind in kinds {
                    actions.extend(self.piece_actions(kind, from)?);
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
    fn apply_piece_action_unchecked(&mut self, action: &DefaultAction) -> Result<(), Error> {
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
        let victim = self.warded_victim(action, piece.owner)?;
        if let Some(victim) = victim {
            self.wards.remove(&victim.id);
            self.events.push(format!(
                "가호: {}의 포획을 한 번 막았습니다.",
                piece_name(victim.kind)
            ));
        } else {
            self.apply_effect(action, ActionSource::PieceMove)?;
        }
        self.moved_ids.insert(piece.id);
        if let Some(state) = self.limiter.get_mut(&piece.id) {
            if state.active {
                state.moves_left = state.moves_left.saturating_sub(1);
            }
        } else {
            self.normal_actions_used = self.normal_actions_used.saturating_add(1);
        }
        if let Some(p) = self.players.iter_mut().find(|p| p.id == piece.id) {
            p.moved = true;
        }
        self.actions_left -= 1;
        if self.phase == Phase::Active && self.actions_left == 0 {
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
        if self.phase != Phase::Active {
            return Err(Error::Finished);
        }
        let expired: Vec<_> = self
            .limiter
            .iter()
            .filter(|(_, s)| s.active && s.owner == self.turn)
            .map(|(&id, _)| id)
            .collect();
        if !expired.is_empty() {
            self.remove_for_limiter(&expired)?;
            self.events.push(format!(
                "리미터 해제: 효과 종료로 {}기 사망했습니다.",
                expired.len()
            ));
            for id in expired {
                self.limiter.remove(&id);
            }
            if self.phase == Phase::Finished {
                return Ok(());
            }
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
        self.turn = if self.turn == PLAYER { ENEMY } else { PLAYER };
        self.actions_left = self.actions_per_turn;
        self.normal_actions_used = 0;
        let mut starts = false;
        for s in self.limiter.values_mut() {
            if s.owner == self.turn && !s.active {
                s.active = true;
                s.moves_left = 3;
                starts = true;
            }
        }
        self.limiter_turn = starts;
        if starts {
            self.actions_left = self.actions_left.max(3);
        }
        Ok(())
    }
    pub fn limiter_status(&self, id: PieceId) -> Option<(bool, u8)> {
        self.limiter.get(&id).map(|s| (s.active, s.moves_left))
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
        self.limiter.insert(
            id,
            LimiterState {
                owner: piece.owner,
                active: false,
                moves_left: 3,
            },
        );
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
        self.apply_effect(action, ActionSource::ResolvedCardEffect)
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
                            || (piece.id != self.king_id && !self.is_enemy_royal(piece.id)))
                    {
                        self.board.set_piece(pos, None)?;
                        self.wards.remove(&piece.id);
                        self.limiter.remove(&piece.id);
                        removed += 1;
                    }
                }
            }
        }
        self.sync_players();
        Ok(removed)
    }
    fn apply_effect(&mut self, action: &DefaultAction, source: ActionSource) -> Result<(), Error> {
        // Card effects need royal-protection rollback; compound moves need a staging board
        // while rule effects inspect each intermediate position. A single piece step can
        // use chessembly's changed-square rollback directly on the live board.
        let mut staged = (source == ActionSource::ResolvedCardEffect
            || !action.continuation.is_empty())
        .then(|| self.board.clone());
        let next = staged.as_mut().unwrap_or(&mut self.board);
        let mut retaliation = None;
        let mut player_captured = false;
        let mut enemy_captured = false;
        let mut absorbed = vec![];
        let self_defense_target = self.self_defense_target;
        if source == ActionSource::ResolvedCardEffect {
            apply_default_action(next, action)?;
            for id in std::iter::once(self.king_id)
                .chain(std::iter::once(self.royal_id))
                .chain(self.additional_royal_ids.iter().copied())
            {
                if !next.cells().iter().flatten().any(|p| p.id == id) {
                    return Err(Error::ProtectedRoyal);
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
                apply_default_action(next, &DefaultAction::one(step.clone()))?;
                player_captured |=
                    !matches!(step.effect, crate::Effect::Move | crate::Effect::Shift)
                        && captor.is_some_and(|p| p.owner == PLAYER)
                        && victim.is_some_and(|p| p.owner == ENEMY);
                enemy_captured |=
                    !matches!(step.effect, crate::Effect::Move | crate::Effect::Shift)
                        && captor.is_some_and(|p| p.owner == ENEMY)
                        && victim.is_some_and(|p| p.owner == PLAYER);
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
        if source == ActionSource::PieceMove {
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
                if piece.owner == PLAYER {
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
            }
        }
    }
    fn update_result(&mut self, reason: ResultReason) {
        let king_alive = self
            .players
            .iter()
            .any(|p| p.id == self.king_id && p.position.is_some());
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
