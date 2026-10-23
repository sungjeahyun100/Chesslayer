use super::{movement, PieceDefinition};
use crate::PieceType;

pub const BISHOP: PieceType = 3;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 30,
    description: "대각선으로 원하는 만큼 이동·포획합니다. 다른 기물을 통과할 수 없습니다.",
    ..PieceDefinition::new(BISHOP, "bishop", "비숍", movement)
};

fn movement() -> String {
    movement::slides(movement::DIAGONAL)
}
