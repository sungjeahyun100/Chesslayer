use super::PieceDefinition;
use crate::PieceType;

pub const FUNERAL_CART: PieceType = 52;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 60,
    description: "도살 수레 능력 유지. 직선 이동 경로의 시체를 통과할 수 있습니다. 살아 있는 기물은 통과하지 못합니다.",
    upgrades: &[
        super::UpgradeDefinition { target: super::SLAUGHTER_CART, key: "slaughter_cart", effect: "살육의 수레(으)로 영구 강화합니다.", requirement: "살아 있는 기물 · 쉼터 행동 1회" },
        super::UpgradeDefinition { target: super::FEAST_CART, key: "feast_cart", effect: "연회의 수레(으)로 영구 강화합니다.", requirement: "살아 있는 기물 · 쉼터 행동 1회" }
    ],
    ..PieceDefinition::new(FUNERAL_CART, "funeral_cart", "운구 수레", movement)
};

// Each piece keeps its complete movement script here. Coordinates are owner-relative.
fn movement() -> String {
    "label(1) piece-on(corpse,1,0) jne(2) {take(1,0)} anchor(1,0) jmp(1) label(2) take-move(1,0) jmp(1);\nlabel(1) piece-on(corpse,-1,0) jne(2) {take(-1,0)} anchor(-1,0) jmp(1) label(2) take-move(-1,0) jmp(1);\nlabel(1) piece-on(corpse,0,-1) jne(2) {take(0,-1)} anchor(0,-1) jmp(1) label(2) take-move(0,-1) jmp(1);\nlabel(1) piece-on(corpse,0,1) jne(2) {take(0,1)} anchor(0,1) jmp(1) label(2) take-move(0,1) jmp(1)".into()
}
