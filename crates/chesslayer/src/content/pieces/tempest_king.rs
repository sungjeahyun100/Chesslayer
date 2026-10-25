use super::{movement, PieceDefinition};
use crate::PieceType;

pub const TEMPEST_KING: PieceType = 34;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 10,
    description: "템페스트 킹. 기본 킹처럼 모든 방향으로 한 칸 이동·포획합니다.",
    ..PieceDefinition::new(TEMPEST_KING, "tempest_king", "템페스트 킹", movement)
};

fn movement() -> String {
    movement::steps(&movement::all_directions())
}
