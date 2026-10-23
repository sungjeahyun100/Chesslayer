//! Chesslayer battle rules and persistent player piece state.
use crate::{
    apply_default_action, ActionError, Board, BoardError, DefaultAction, GameResult, Piece,
    PieceId, PieceType, PlayerId, Position, ResultReason,
};
use chessembly::{Error as DslError, Program};
use std::collections::{BTreeMap, HashSet};
pub const PLAYER: PlayerId = 0;
pub const ENEMY: PlayerId = 1;
// Keep the existing crate-root API through battle's re-exports.
use crate::content::pieces;
pub use crate::content::pieces::{
    piece_name, AMAZON, BISHOP, BOUNCING_BISHOP, BOUNCING_PAWN, BOUNCING_QUEEN, BOUNCING_ROOK,
    KING, KNIGHT, PAWN, QUEEN, ROOK, TEMPEST_BISHOP, TEMPEST_KNIGHT, TEMPEST_PAWN, TEMPEST_QUEEN,
    TEMPEST_ROOK,
};

#[derive(Debug, Clone, Copy)]
pub struct BattleOptions {
    pub actions_per_turn: u8,
    pub self_defense: bool,
}
impl Default for BattleOptions {
    fn default() -> Self {
        Self {
            actions_per_turn: 1,
            self_defense: false,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerPiece {
    pub id: PieceId,
    pub kind: PieceType,
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
    pub king_id: PieceId,
    pub actions_left: u8,
    pub events: Vec<String>,
    actions_per_turn: u8,
    wards: HashSet<PieceId>,
    players: Vec<PlayerPiece>,
    programs: BTreeMap<PieceType, Program>,
    initial_double_step: Program,
    moved_ids: HashSet<PieceId>,
}
impl Battle {
    pub fn standard_roster() -> Vec<PlayerPiece> {
        let back = [ROOK, KNIGHT, BISHOP, QUEEN, KING, BISHOP, KNIGHT, ROOK];
        (0..16)
            .map(|i| PlayerPiece {
                id: i + 1,
                kind: if i < 8 { back[i as usize] } else { PAWN },
                position: Some(Position::new((i % 8) as i32, if i < 8 { 7 } else { 6 })),
                moved: false,
                upgrades: vec![],
                persistent: BTreeMap::new(),
            })
            .collect()
    }
    pub fn new(start: BattleStart) -> Result<Self, Error> {
        Self::with_options(start, BattleOptions::default())
    }
    pub fn with_options(start: BattleStart, options: BattleOptions) -> Result<Self, Error> {
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
        let mut king_id = None;
        for p in &start.players {
            if !ids.insert(p.id) || !valid_kind(p.kind) {
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
        let start_positions: BTreeMap<_, _> = start
            .players
            .iter()
            .filter_map(|p| p.position.map(|pos| (p.id, pos)))
            .collect();
        let mut players = start.players;
        let mut found_royal = false;
        let mut enemy_positions = HashSet::new();
        for e in &start.encounter.enemies {
            if !ids.insert(e.id) || !valid_kind(e.kind) || !enemy_positions.insert(e.position) {
                return Err(Error::InvalidSetup);
            }
            if e.id == start.encounter.royal_id {
                found_royal = true
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
            let collision = players
                .iter()
                .any(|p| start_positions.get(&p.id) == Some(&e.position));
            if collision && options.self_defense && e.id != start.encounter.royal_id {
                board.set_piece(e.position, None)?;
                events.push(format!(
                    "호신용품: {}의 적도 제거했습니다.",
                    square(e.position)
                ));
                continue;
            }
            if collision && options.self_defense && e.id == start.encounter.royal_id {
                events.push("호신용품: 적 로얄은 직접 포획으로만 제거할 수 있습니다.".into());
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
        if !found_royal {
            return Err(Error::InvalidSetup);
        }
        let programs = pieces::ALL
            .iter()
            .map(|d| Program::parse(&(d.movement)()).map(|program| (d.kind, program)))
            .collect::<Result<_, _>>()?;
        let initial_double_step = Program::parse(pieces::movement::INITIAL_DOUBLE_STEP)?;
        let moved_ids = players.iter().filter(|p| p.moved).map(|p| p.id).collect();
        let wards = players
            .iter()
            .filter(|p| p.position.is_some() && p.persistent.get("ward").copied().unwrap_or(0) > 0)
            .map(|p| p.id)
            .collect();
        let mut battle = Self {
            events,
            wards,
            actions_left: options.actions_per_turn,
            actions_per_turn: options.actions_per_turn,
            board,
            turn: PLAYER,
            result: GameResult::Ongoing,
            phase: Phase::Active,
            royal_id: start.encounter.royal_id,
            king_id,
            players,
            programs,
            initial_double_step,
            moved_ids,
        };
        battle.update_result(ResultReason::Counterattack);
        Ok(battle)
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
        self.wards = roster
            .iter()
            .filter(|p| p.position.is_some() && p.persistent.get("ward").copied().unwrap_or(0) > 0)
            .map(|p| p.id)
            .collect();
        self.moved_ids = roster.iter().filter(|p| p.moved).map(|p| p.id).collect();
        self.board = next;
        self.players = roster;
        Ok(())
    }
    pub fn player_pieces(&self) -> &[PlayerPiece] {
        &self.players
    }
    pub fn outcome(&self) -> Vec<PlayerPiece> {
        self.players.clone()
    }
    pub fn legal_actions(&self, from: Position) -> Result<Vec<DefaultAction>, Error> {
        if self.phase != Phase::Active {
            return Err(Error::Finished);
        }
        let piece = self.board.piece_at(from)?.ok_or(Error::IllegalAction)?;
        if piece.owner != self.turn {
            return Err(Error::WrongTurn);
        }
        let mut actions = self
            .programs
            .get(&piece.kind)
            .ok_or(Error::InvalidSetup)?
            .execute(&self.board, from)?;
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
        for action in actions {
            let destination = action.steps().last().unwrap().move_to;
            if destination.rank == last_rank && !definition.promotions.is_empty() {
                for &kind in definition.promotions {
                    let mut promoted = action.clone();
                    let step = promoted
                        .continuation
                        .last_mut()
                        .unwrap_or(&mut promoted.first);
                    step.transition = Some(pieces::definition(kind).unwrap().key.into());
                    if !result.contains(&promoted) {
                        result.push(promoted);
                    }
                }
            } else if !result.contains(&action) {
                result.push(action);
            }
        }
        Ok(result)
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
        // A ward cancels the whole capture action; the attack still spends one action.
        let victim = action.steps().find_map(|step| {
            if matches!(step.effect, crate::Effect::Shift | crate::Effect::Move) {
                return None;
            }
            self.board
                .piece_at(step.take)
                .ok()
                .flatten()
                .filter(|p| p.owner != piece.owner && self.wards.contains(&p.id))
        });
        if let Some(victim) = victim {
            self.wards.remove(&victim.id);
            if let Some(p) = self.players.iter_mut().find(|p| p.id == victim.id) {
                p.persistent.remove("ward");
            }
            self.events.push(format!(
                "가호: {}의 포획을 한 번 막았습니다.",
                piece_name(victim.kind)
            ));
        } else {
            self.apply_effect(action, ActionSource::PieceMove)?;
        }
        self.moved_ids.insert(piece.id);
        if let Some(p) = self.players.iter_mut().find(|p| p.id == piece.id) {
            p.moved = true;
        }
        self.actions_left -= 1;
        if self.phase == Phase::Active && self.actions_left == 0 {
            self.end_turn()?;
        }
        Ok(())
    }
    pub fn end_turn(&mut self) -> Result<(), Error> {
        if self.phase != Phase::Active {
            return Err(Error::Finished);
        }
        self.turn = if self.turn == PLAYER { ENEMY } else { PLAYER };
        self.actions_left = self.actions_per_turn;
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
    pub fn grant_ward(&mut self, id: PieceId) -> Result<(), Error> {
        if self.phase != Phase::Active {
            return Err(Error::Finished);
        }
        if !self.board.cells().iter().flatten().any(|p| p.id == id) {
            return Err(Error::IllegalAction);
        }
        self.wards.insert(id);
        if let Some(p) = self.players.iter_mut().find(|p| p.id == id) {
            p.persistent.insert("ward".into(), 1);
        }
        Ok(())
    }
    /// Non-capture removal always protects both royals. Counterattacks only occur in setup.
    pub fn remove_nonroyals(&mut self, ids: &[PieceId]) -> Result<usize, Error> {
        if self.phase != Phase::Active {
            return Err(Error::Finished);
        }
        let mut removed = 0;
        for rank in 0..8 {
            for file in 0..8 {
                let pos = Position::new(file, rank);
                if let Some(piece) = self.board.piece_at(pos)? {
                    if ids.contains(&piece.id)
                        && piece.id != self.king_id
                        && piece.id != self.royal_id
                    {
                        self.board.set_piece(pos, None)?;
                        self.wards.remove(&piece.id);
                        removed += 1;
                    }
                }
            }
        }
        self.sync_players();
        Ok(removed)
    }
    fn apply_effect(&mut self, action: &DefaultAction, source: ActionSource) -> Result<(), Error> {
        if source == ActionSource::ResolvedCardEffect {
            let mut next = self.board.clone();
            apply_default_action(&mut next, action)?;
            for id in [self.king_id, self.royal_id] {
                if !next.cells().iter().flatten().any(|p| p.id == id) {
                    return Err(Error::ProtectedRoyal);
                }
            }
            self.board = next;
        } else {
            apply_default_action(&mut self.board, action)?;
        }
        self.sync_players();
        if source == ActionSource::PieceMove {
            self.update_result(ResultReason::RoyalCaptured);
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
                        p.kind = piece.kind;
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
            .any(|p| p.owner == ENEMY && p.id == self.royal_id);
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
        }
    }
}
fn valid_kind(k: PieceType) -> bool {
    pieces::definition(k).is_some()
}
pub fn square(p: Position) -> String {
    format!("{}{}", (b'a' + p.file as u8) as char, 8 - p.rank)
}
