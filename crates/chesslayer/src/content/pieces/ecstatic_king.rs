//! v0.3 prototype; movement belongs to this piece independently.
use super::PieceDefinition;
use crate::PieceType;
pub const ECSTATIC_KING: PieceType = 52;
pub const DEFINITION: PieceDefinition = PieceDefinition {
    description: "아군 시체 포획 시 드로우 1, 포획자 민첩 1, 마나 1. 별도 표기가 없는 발동 능력은 전투당 1회.",
    upgrades: &[],
    initial_double_step: false,
    ..PieceDefinition::new(ECSTATIC_KING, "ecstatic_king", "환희의 군주", movement)
};
fn movement() -> String {
    r#"take-move(1,0);take-move(-1,0);take-move(0,1);take-move(0,-1);take-move(1,1);take-move(-1,1);take-move(1,-1);take-move(-1,-1);"#.into()
}
