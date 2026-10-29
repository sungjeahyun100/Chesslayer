use super::PieceDefinition;
use crate::PieceType;

pub const GLUTTON_FOLLOWER: PieceType = 65;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 30,
    description:
        "굶주린 신도 행마 유지. 전투당 첫 시체 포획 시 황충의 축복에 더해 민첩 1을 추가합니다.",
    initial_double_step: true,
    ..PieceDefinition::new(
        GLUTTON_FOLLOWER,
        "glutton_follower",
        "탐식의 신도",
        movement,
    )
};

// Each piece keeps its complete movement script here. Coordinates are owner-relative.
fn movement() -> String {
    "move(0,1); enemy(-1,1) take(-1,1); enemy(1,1) take(1,1);\npiece-on(corpse,0,1) take(0,1)"
        .into()
}
