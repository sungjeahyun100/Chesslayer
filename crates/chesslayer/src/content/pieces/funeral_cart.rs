//! v0.3 prototype; movement belongs to this piece independently.
use super::PieceDefinition;
use crate::PieceType;
pub const FUNERAL_CART: PieceType = 58;
pub const DEFINITION: PieceDefinition = PieceDefinition {
    description: "직선 이동 중 시체를 통과. 시체 포획 시 드로우 1. 별도 표기가 없는 발동 능력은 전투당 1회.",
    upgrades: &[super::UpgradeDefinition {target: super::SLAUGHTER_CART,key:"slaughter_cart",effect:"시체 통과, 시체 포획 시 드로우 1 및 민첩 1. 별도 표기가 없는 발동 능력은 전투당 1회.",requirement:"생존 기물 · 강화 기회 1회"},super::UpgradeDefinition {target: super::FEAST_CART,key:"feast_cart",effect:"시체 통과, 시체 포획 시 드로우 1 및 2칸 내 다른 시체 1칸 이동. 별도 표기가 없는 발동 능력은 전투당 1회.",requirement:"생존 기물 · 강화 기회 1회"}],
    initial_double_step: false,
    ..PieceDefinition::new(FUNERAL_CART, "funeral_cart", "운구 수레", movement)
};
fn movement() -> String {
    r#"take-move(1,0) repeat(1);take-move(-1,0) repeat(1);take-move(0,1) repeat(1);take-move(0,-1) repeat(1);"#.into()
}
