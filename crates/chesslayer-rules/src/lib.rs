//! Chesslayer battle rules and persistent player piece state.
use chessembly::{Error as DslError, Program};
use chesslayer_core::{
    apply_default_action, ActionError, Board, BoardError, DefaultAction, GameResult, Piece,
    PieceId, PieceType, PlayerId, Position, ResultReason,
};
use std::collections::{BTreeMap, HashSet};
pub const PLAYER: PlayerId = 0;
pub const ENEMY: PlayerId = 1;
pub const PAWN: PieceType = 1;
pub const KNIGHT: PieceType = 2;
pub const BISHOP: PieceType = 3;
pub const ROOK: PieceType = 4;
pub const QUEEN: PieceType = 5;
pub const KING: PieceType = 6;
const NAMES: [&str; 7] = ["", "pawn", "knight", "bishop", "rook", "queen", "king"];
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
pub struct Battle {
    pub board: Board,
    pub turn: PlayerId,
    pub result: GameResult,
    pub phase: Phase,
    pub royal_id: PieceId,
    king_id: PieceId,
    players: Vec<PlayerPiece>,
    programs: Vec<Program>,
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
        let mut board = Board::new(8, 8)?;
        board.set_catalog(
            NAMES.iter().map(|s| s.to_string()).collect(),
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
        let mut players = start.players;
        let mut found_royal = false;
        for e in &start.encounter.enemies {
            if !ids.insert(e.id) || !valid_kind(e.kind) {
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
                    p.position = None
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
        if !found_royal {
            return Err(Error::InvalidSetup);
        }
        let programs = movement_programs()?;
        let moved_ids = players.iter().filter(|p| p.moved).map(|p| p.id).collect();
        let mut battle = Self {
            board,
            turn: PLAYER,
            result: GameResult::Ongoing,
            phase: Phase::Active,
            royal_id: start.encounter.royal_id,
            king_id,
            players,
            programs,
            moved_ids,
        };
        battle.update_result();
        Ok(battle)
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
        let mut actions = self.programs[usize::from(piece.kind)].execute(&self.board, from)?;
        if piece.kind == PAWN
            && !self.moved_ids.contains(&piece.id)
            && from.rank == if piece.owner == PLAYER { 6 } else { 1 }
        {
            let double = self.programs[0].execute(&self.board, from)?;
            if let Some(second) = double.get(1) {
                actions.push(second.clone())
            }
        }
        Ok(actions)
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
        self.apply_effect(action, ActionSource::PieceMove)?;
        self.moved_ids.insert(piece.id);
        if self.phase == Phase::Active {
            self.turn = if self.turn == PLAYER { ENEMY } else { PLAYER }
        }
        Ok(())
    }
    /// Receives an already validated and resolved card effect. Card costs and targeting belong to a future card module.
    pub fn apply_resolved_card_effect(&mut self, action: &DefaultAction) -> Result<(), Error> {
        if self.phase != Phase::Active {
            return Err(Error::Finished);
        }
        self.apply_effect(action, ActionSource::ResolvedCardEffect)
    }
    fn apply_effect(&mut self, action: &DefaultAction, _source: ActionSource) -> Result<(), Error> {
        let previous: Vec<_> = self.players.iter().map(|p| p.position).collect();
        apply_default_action(&mut self.board, action)?;
        for p in &mut self.players {
            p.position = None
        }
        for rank in 0..8 {
            for file in 0..8 {
                let pos = Position::new(file, rank);
                if let Some(piece) = self.board.piece_at(pos)? {
                    if piece.owner == PLAYER {
                        if let Some(p) = self.players.iter_mut().find(|p| p.id == piece.id) {
                            p.position = Some(pos);
                            p.kind = piece.kind
                        }
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
        self.update_result();
        Ok(())
    }
    fn update_result(&mut self) {
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
                reason: ResultReason::RoyalCaptured,
            }
        } else if !royal_alive {
            GameResult::Win {
                winner: PLAYER,
                reason: ResultReason::RoyalCaptured,
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
    (PAWN..=KING).contains(&k)
}
fn movement_programs() -> Result<Vec<Program>, Error> {
    let mut programs = vec![Program::parse("move(0,1) move(0,1)")?];
    programs.push(Program::parse(
        "move(0,1); enemy(-1,1) take(-1,1); enemy(1,1) take(1,1)",
    )?);
    let knight = [
        (-1, 2),
        (1, 2),
        (-2, 1),
        (2, 1),
        (-2, -1),
        (2, -1),
        (-1, -2),
        (1, -2),
    ];
    programs.push(Program::parse(
        &knight
            .iter()
            .map(|(x, y)| format!("take-move({x},{y})"))
            .collect::<Vec<_>>()
            .join(";"),
    )?);
    let bishop = [(-1, -1), (-1, 1), (1, -1), (1, 1)];
    let rook = [(-1, 0), (1, 0), (0, -1), (0, 1)];
    let all = [bishop.as_slice(), rook.as_slice()].concat();
    for dirs in [bishop.as_slice(), rook.as_slice(), all.as_slice()] {
        programs.push(Program::parse(
            &dirs
                .iter()
                .map(|(x, y)| format!("take-move({x},{y}) repeat(1)"))
                .collect::<Vec<_>>()
                .join(";"),
        )?)
    }
    programs.push(Program::parse(
        &all.iter()
            .map(|(x, y)| format!("take-move({x},{y})"))
            .collect::<Vec<_>>()
            .join(";"),
    )?);
    Ok(programs)
}

/// Core-facing entry point for the battle module.
pub struct ChesslayerRules;
impl chesslayer_core::RuleModule for ChesslayerRules {
    type Setup = BattleStart;
    type State = Battle;
    type Action = DefaultAction;
    type Outcome = Vec<PlayerPiece>;
    type Error = Error;
    fn start(&self, setup: BattleStart) -> Result<Battle, Error> {
        Battle::new(setup)
    }
    fn legal_actions(&self, state: &Battle, from: Position) -> Result<Vec<DefaultAction>, Error> {
        state.legal_actions(from)
    }
    fn apply_action(&self, state: &mut Battle, action: &DefaultAction) -> Result<(), Error> {
        state.apply_piece_action(action)
    }
    fn result(&self, state: &Battle) -> GameResult {
        state.result
    }
    fn outcome(&self, state: &Battle) -> Vec<PlayerPiece> {
        state.outcome()
    }
}
