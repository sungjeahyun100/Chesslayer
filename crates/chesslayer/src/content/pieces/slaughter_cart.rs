use super::PieceDefinition;
use crate::PieceType;

pub const SLAUGHTER_CART: PieceType = 53;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 70,
    description: "운구 수레 능력 유지. 전투당 1회, 자신의 시체 포획 시 민첩 1을 추가합니다.",
    ..PieceDefinition::new(SLAUGHTER_CART, "slaughter_cart", "살육의 수레", movement)
};

// Each piece keeps its complete movement script here. Coordinates are owner-relative.
fn movement() -> String {
    "label(1) piece-on(corpse,1,0) jne(2) {take(1,0)} anchor(1,0) jmp(1) label(2) take-move(1,0) jmp(1);\nlabel(1) piece-on(corpse,-1,0) jne(2) {take(-1,0)} anchor(-1,0) jmp(1) label(2) take-move(-1,0) jmp(1);\nlabel(1) piece-on(corpse,0,-1) jne(2) {take(0,-1)} anchor(0,-1) jmp(1) label(2) take-move(0,-1) jmp(1);\nlabel(1) piece-on(corpse,0,1) jne(2) {take(0,1)} anchor(0,1) jmp(1) label(2) take-move(0,1) jmp(1)".into()
}
