use std::fmt;

pub type PieceType = u16;
pub type PlayerId = u16;
pub type PieceId = u32;

/// Logical coordinates: (file, rank). Coordinates are zero based.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub file: i32,
    pub rank: i32,
}

impl Position {
    pub const fn new(file: i32, rank: i32) -> Self {
        Self { file, rank }
    }
}

/// The shared piece identity. Per-piece capabilities and metadata belong to the rule module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Piece {
    pub kind: PieceType,
    pub owner: PlayerId,
    pub id: PieceId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoardError {
    InvalidDimensions,
    InvalidMaskLength,
    OutsidePlayableArea(Position),
    UnknownPieceType(PieceType),
    UnknownPlayer(PlayerId),
    InvalidDirection(PlayerId),
}

impl fmt::Display for BoardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for BoardError {}

/// A board can be rectangular or have a module-defined playable-cell mask.
/// Empty cells are None; piece type 0 is reserved as the empty catalog entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    width: usize,
    height: usize,
    cells: Vec<Option<Piece>>,
    playable: Option<Vec<bool>>,
    piece_names: Vec<String>,
    color_names: Vec<String>,
    color_directions: Vec<i8>,
}

impl Board {
    pub fn new(width: usize, height: usize) -> Result<Self, BoardError> {
        let len = width
            .checked_mul(height)
            .ok_or(BoardError::InvalidDimensions)?;
        if width == 0
            || height == 0
            || width > i32::MAX as usize
            || height > i32::MAX as usize
            || len > i32::MAX as usize
        {
            return Err(BoardError::InvalidDimensions);
        }
        Ok(Self {
            width,
            height,
            cells: vec![None; len],
            playable: None,
            piece_names: vec![String::new()],
            color_names: Vec::new(),
            color_directions: Vec::new(),
        })
    }

    pub const fn width(&self) -> usize {
        self.width
    }
    pub const fn height(&self) -> usize {
        self.height
    }
    pub fn cells(&self) -> &[Option<Piece>] {
        &self.cells
    }
    pub fn piece_names(&self) -> &[String] {
        &self.piece_names
    }
    pub fn color_names(&self) -> &[String] {
        &self.color_names
    }

    pub fn set_catalog(
        &mut self,
        piece_names: Vec<String>,
        color_names: Vec<String>,
        directions: Vec<i8>,
    ) -> Result<(), BoardError> {
        if piece_names.first().is_none_or(|name| !name.is_empty())
            || (!directions.is_empty() && directions.len() != color_names.len())
        {
            return Err(BoardError::InvalidDimensions);
        }
        for (owner, direction) in directions.iter().enumerate() {
            if !matches!(*direction, -1 | 1) {
                return Err(BoardError::InvalidDirection(owner as PlayerId));
            }
        }
        self.piece_names = piece_names;
        self.color_names = color_names;
        self.color_directions = directions;
        Ok(())
    }

    pub fn set_playable_mask(&mut self, mask: Option<Vec<bool>>) -> Result<(), BoardError> {
        if mask.as_ref().is_some_and(|m| m.len() != self.cells.len()) {
            return Err(BoardError::InvalidMaskLength);
        }
        if let Some(mask) = &mask {
            if self
                .cells
                .iter()
                .zip(mask)
                .any(|(piece, can_play)| piece.is_some() && !can_play)
            {
                return Err(BoardError::InvalidMaskLength);
            }
        }
        self.playable = mask;
        Ok(())
    }

    pub fn index(&self, position: Position) -> Option<usize> {
        let file = usize::try_from(position.file).ok()?;
        let rank = usize::try_from(position.rank).ok()?;
        if file >= self.width || rank >= self.height {
            return None;
        }
        let index = rank * self.width + file;
        if self.playable.as_ref().is_some_and(|mask| !mask[index]) {
            return None;
        }
        Some(index)
    }

    pub fn contains(&self, position: Position) -> bool {
        self.index(position).is_some()
    }

    pub fn piece_at(&self, position: Position) -> Result<Option<Piece>, BoardError> {
        let index = self
            .index(position)
            .ok_or(BoardError::OutsidePlayableArea(position))?;
        Ok(self.cells[index])
    }

    pub fn set_piece(
        &mut self,
        position: Position,
        piece: Option<Piece>,
    ) -> Result<(), BoardError> {
        let index = self
            .index(position)
            .ok_or(BoardError::OutsidePlayableArea(position))?;
        if let Some(piece) = piece {
            self.validate_piece(piece)?;
        }
        self.cells[index] = piece;
        Ok(())
    }

    fn validate_piece(&self, piece: Piece) -> Result<(), BoardError> {
        if piece.kind == 0 || self.piece_names.get(usize::from(piece.kind)).is_none() {
            return Err(BoardError::UnknownPieceType(piece.kind));
        }
        if self.color_names.get(usize::from(piece.owner)).is_none() {
            return Err(BoardError::UnknownPlayer(piece.owner));
        }
        Ok(())
    }

    pub fn piece_name(&self, piece: Piece) -> Result<&str, BoardError> {
        self.validate_piece(piece)?;
        Ok(&self.piece_names[usize::from(piece.kind)])
    }

    pub fn color_name(&self, piece: Piece) -> Result<&str, BoardError> {
        self.validate_piece(piece)?;
        Ok(&self.color_names[usize::from(piece.owner)])
    }

    pub fn direction(&self, owner: PlayerId) -> Result<i8, BoardError> {
        if self.color_names.get(usize::from(owner)).is_none() {
            return Err(BoardError::UnknownPlayer(owner));
        }
        if self.color_directions.is_empty() {
            return Ok(1);
        }
        self.color_directions
            .get(usize::from(owner))
            .copied()
            .ok_or(BoardError::InvalidDirection(owner))
    }

    pub(crate) fn cells_mut(&mut self) -> &mut [Option<Piece>] {
        &mut self.cells
    }

    pub(crate) fn find_piece_type(&self, name: &str) -> Option<PieceType> {
        self.piece_names
            .iter()
            .position(|item| item == name)
            .filter(|&index| index != 0)
            .and_then(|index| PieceType::try_from(index).ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supports_nonstandard_boards_and_masks() {
        let mut board = Board::new(12, 10).unwrap();
        board
            .set_catalog(
                vec!["".into(), "hopper".into()],
                vec!["white".into()],
                vec![-1],
            )
            .unwrap();
        let origin = Position::new(10, 8);
        board
            .set_piece(
                origin,
                Some(Piece {
                    kind: 1,
                    owner: 0,
                    id: 42,
                }),
            )
            .unwrap();
        assert_eq!(board.direction(0).unwrap(), -1);
        let mut mask = vec![true; 120];
        mask[0] = false;
        board.set_playable_mask(Some(mask)).unwrap();
        assert!(!board.contains(Position::new(0, 0)));
        assert!(board.contains(origin));
        assert_eq!(board.piece_at(origin).unwrap().unwrap().id, 42);
    }

    #[test]
    fn invalid_masks_and_positions_do_not_modify_board() {
        let mut board = Board::new(8, 8).unwrap();
        assert_eq!(
            board.set_playable_mask(Some(vec![true; 20])),
            Err(BoardError::InvalidMaskLength)
        );
        assert!(board.piece_at(Position::new(-1, 1)).is_err());
        assert_eq!(
            board.cells().iter().filter(|piece| piece.is_some()).count(),
            0
        );
    }
}
