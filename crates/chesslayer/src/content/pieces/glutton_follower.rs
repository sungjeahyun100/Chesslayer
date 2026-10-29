//! v0.3 prototype; movement belongs to this piece independently.
use super::PieceDefinition;
use crate::PieceType;
pub const GLUTTON_FOLLOWER: PieceType = 71;
pub const DEFINITION: PieceDefinition = PieceDefinition {
    description:
        "전방 시체 포획. 첫 시체 포획 시 민첩 1 추가. 별도 표기가 없는 발동 능력은 전투당 1회.",
    upgrades: &[],
    initial_double_step: true,
    ..PieceDefinition::new(
        GLUTTON_FOLLOWER,
        "glutton_follower",
        "탐식의 신도",
        movement,
    )
};
fn movement() -> String {
    r#"move(0,1); enemy(-1,1) take(-1,1); enemy(1,1) take(1,1);"#.into()
}
