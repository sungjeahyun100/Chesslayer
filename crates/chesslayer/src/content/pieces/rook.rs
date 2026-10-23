use super::{movement, PieceDefinition};
use crate::PieceType;

pub const ROOK: PieceType = 4;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 50,
    description: "가로·세로로 원하는 만큼 이동·포획합니다. 다른 기물을 통과할 수 없습니다.",
    ..PieceDefinition::new(ROOK, "rook", "룩", movement)
};

fn movement() -> String {
    movement::slides(movement::ORTHOGONAL)
}
