use super::PieceDefinition;
use crate::PieceType;

pub const GLUTTON_FOLLOWER: PieceType = 65;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 30,
    description: "굶주린 신도 능력 계승. 시체를 포획할 때마다 행마권 1을 얻습니다.",
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
    "move(0,1); enemy(-1,1) take(-1,1); enemy(1,1) take(1,1); piece-on(corpse,0,1) take(0,1)".into()
}
