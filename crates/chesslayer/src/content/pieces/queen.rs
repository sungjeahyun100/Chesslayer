use super::{movement, PieceDefinition};
use crate::PieceType;

pub const QUEEN: PieceType = 5;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 90,
    description:
        "가로·세로·대각선으로 원하는 만큼 이동·포획합니다. 다른 기물을 통과할 수 없습니다.",
    ..PieceDefinition::new(QUEEN, "queen", "퀸", movement)
};

fn movement() -> String {
    movement::slides(&movement::all_directions())
}
