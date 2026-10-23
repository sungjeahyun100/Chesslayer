use super::{movement, PieceDefinition};
use crate::PieceType;

pub const KING: PieceType = 6;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 10,
    description: "모든 방향으로 한 칸 이동·포획합니다. 아군 킹을 잃으면 원정이 끝납니다.",
    ..PieceDefinition::new(KING, "king", "킹", movement)
};

fn movement() -> String {
    movement::steps(&movement::all_directions())
}
