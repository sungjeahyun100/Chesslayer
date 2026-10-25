use super::{movement, PieceDefinition};
use crate::PieceType;

pub const EMPEROR: PieceType = 31;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 100,
    description:
        "킹처럼 움직입니다. 직접 포획한 기물의 행마법을 전투 동안 흡수하며 선공권을 가집니다.",
    ..PieceDefinition::new(EMPEROR, "emperor", "제왕", movement)
};

fn movement() -> String {
    movement::steps(&movement::all_directions())
}
