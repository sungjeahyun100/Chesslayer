use super::{movement, PieceDefinition};
use crate::PieceType;

pub const TYPHOON_KING: PieceType = 98;
pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 10,
    description:
        "킹의 기본 행마를 사용합니다. 특수 능력으로 적 기물 2~6기의 위치를 무작위로 뒤섞습니다.",
    ..PieceDefinition::new(TYPHOON_KING, "typhoon_king", "태풍 킹", movement)
};

fn movement() -> String {
    movement::steps(&movement::all_directions())
}
