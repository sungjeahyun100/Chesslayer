use super::PieceDefinition;
use crate::PieceType;

pub const BUTCHER_CART: PieceType = 51;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 50,
    description: "룩 행마. 전투당 1회, 자신이 시체를 포획하면 카드 1장을 뽑습니다.",
    upgrades: &[super::UpgradeDefinition {
        target: super::FUNERAL_CART,
        key: "funeral_cart",
        effect: "운구 수레(으)로 영구 강화합니다.",
        requirement: "살아 있는 기물 · 쉼터 행동 1회",
    }],
    ..PieceDefinition::new(BUTCHER_CART, "butcher_cart", "도살 수레", movement)
};

// Each piece keeps its complete movement script here. Coordinates are owner-relative.
fn movement() -> String {
    "take-move(1,0) repeat(1);\ntake-move(-1,0) repeat(1);\ntake-move(0,-1) repeat(1);\ntake-move(0,1) repeat(1)".into()
}
