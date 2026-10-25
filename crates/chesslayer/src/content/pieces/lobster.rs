use super::{movement, PieceDefinition};
use crate::PieceType;

pub const LOBSTER: PieceType = 33;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 25,
    description: "상하좌우로 한 칸 이동·포획합니다.",
    ..PieceDefinition::new(LOBSTER, "lobster", "랍스타", movement)
};

fn movement() -> String {
    movement::steps(movement::ORTHOGONAL)
}
