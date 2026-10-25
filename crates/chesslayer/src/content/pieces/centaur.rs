use super::{movement, PieceDefinition};
use crate::PieceType;

pub const CENTAUR: PieceType = 32;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 65,
    description: "킹의 한 칸 이동과 나이트의 도약을 모두 사용합니다.",
    ..PieceDefinition::new(CENTAUR, "centaur", "켄타우로스", movement)
};

fn movement() -> String {
    format!(
        "{};{}",
        movement::steps(&movement::all_directions()),
        movement::steps(movement::KNIGHT)
    )
}
