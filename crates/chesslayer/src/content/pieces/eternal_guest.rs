//! v0.3 prototype; movement belongs to this piece independently.
use super::PieceDefinition;
use crate::PieceType;
pub const ETERNAL_GUEST: PieceType = 74;
pub const DEFINITION: PieceDefinition = PieceDefinition {
    description: "보스 전용. 첫 시체 포획 3회 민첩 +1. 3·6수에 예고한 대상 허기 악화.",
    upgrades: &[],
    initial_double_step: false,
    ..PieceDefinition::new(ETERNAL_GUEST, "eternal_guest", "영원한 주빈", movement)
};
fn movement() -> String {
    r#"take-move(1,0);take-move(-1,0);take-move(0,1);take-move(0,-1);take-move(1,1);take-move(-1,1);take-move(1,-1);take-move(-1,-1);"#.into()
}
