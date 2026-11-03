use super::{movement, PieceDefinition};
use crate::PieceType;

pub const HILL_KING: PieceType = 94;
pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 10,
    description: "킹의 기본 행마를 사용합니다. 중앙 네 칸에 도달하면 승리합니다.",
    ..PieceDefinition::new(HILL_KING, "hill_king", "언덕의 왕", movement)
};

fn movement() -> String {
    movement::steps(&movement::all_directions())
}
