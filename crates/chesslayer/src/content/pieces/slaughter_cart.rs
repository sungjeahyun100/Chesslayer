//! v0.3 prototype; movement belongs to this piece independently.
use super::PieceDefinition;
use crate::PieceType;
pub const SLAUGHTER_CART: PieceType = 59;
pub const DEFINITION: PieceDefinition = PieceDefinition {
    description:
        "시체 통과, 시체 포획 시 드로우 1 및 민첩 1. 별도 표기가 없는 발동 능력은 전투당 1회.",
    upgrades: &[],
    initial_double_step: false,
    ..PieceDefinition::new(SLAUGHTER_CART, "slaughter_cart", "살육의 수레", movement)
};
fn movement() -> String {
    r#"take-move(1,0) repeat(1);take-move(-1,0) repeat(1);take-move(0,1) repeat(1);take-move(0,-1) repeat(1);"#.into()
}
