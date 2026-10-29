//! v0.3 prototype; movement belongs to this piece independently.
use super::PieceDefinition;
use crate::PieceType;
pub const BUTCHER_CART: PieceType = 57;
pub const DEFINITION: PieceDefinition = PieceDefinition {
    description: "시체 포획 시 드로우 1. 별도 표기가 없는 발동 능력은 전투당 1회.",
    upgrades: &[super::UpgradeDefinition {target: super::FUNERAL_CART,key:"funeral_cart",effect:"직선 이동 중 시체를 통과. 시체 포획 시 드로우 1. 별도 표기가 없는 발동 능력은 전투당 1회.",requirement:"생존 기물 · 강화 기회 1회"}],
    initial_double_step: false,
    ..PieceDefinition::new(BUTCHER_CART, "butcher_cart", "도살 수레", movement)
};
fn movement() -> String {
    r#"take-move(1,0) repeat(1);take-move(-1,0) repeat(1);take-move(0,1) repeat(1);take-move(0,-1) repeat(1);"#.into()
}
