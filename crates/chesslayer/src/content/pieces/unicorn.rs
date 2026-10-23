use super::{movement, PieceDefinition};
use crate::PieceType;

pub const UNICORN: PieceType = 19;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    family: super::PieceFamily::Knight,
    value: 75,
    description: "일반 나이트처럼 이동·포획하거나, 보드의 임의 빈칸 하나로 포획 없이 이동합니다.",
    ..PieceDefinition::new(UNICORN, "unicorn", "유니콘", movement)
};

fn movement() -> String {
    let mut chains = vec![movement::steps(movement::KNIGHT)];
    // The battle board is 8x8. `move(0,0)` checks emptiness in Chesssembly,
    // and the common action applier checks it again at execution time.
    for y in 0..8 {
        for x in 0..8 {
            chains.push(format!("absolute({x},{y}) move(0,0)"));
        }
    }
    chains.join(";")
}
