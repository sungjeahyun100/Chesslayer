use super::PieceDefinition;
use crate::PieceType;

pub const FEAST_CART: PieceType = 54;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 70,
    description: "운구 수레 능력 계승. 시체 포획 시 시체자루 1을 얻습니다. 고기다!: 시체자루 1을 소모해 주변 8칸의 황충의 축복 아군 모두에 행마권 1을 부여합니다.",
    ..PieceDefinition::new(FEAST_CART, "feast_cart", "연회의 수레", movement)
};

// Each piece keeps its complete movement script here. Coordinates are owner-relative.
fn movement() -> String {
    "label(1) piece-on(corpse,1,0) jne(2) {take(1,0)} anchor(1,0) jmp(1) label(2) take-move(1,0) jmp(1);\nlabel(1) piece-on(corpse,-1,0) jne(2) {take(-1,0)} anchor(-1,0) jmp(1) label(2) take-move(-1,0) jmp(1);\nlabel(1) piece-on(corpse,0,-1) jne(2) {take(0,-1)} anchor(0,-1) jmp(1) label(2) take-move(0,-1) jmp(1);\nlabel(1) piece-on(corpse,0,1) jne(2) {take(0,1)} anchor(0,1) jmp(1) label(2) take-move(0,1) jmp(1)".into()
}
