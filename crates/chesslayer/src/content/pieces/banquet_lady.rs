use super::PieceDefinition;
use crate::PieceType;

pub const BANQUET_LADY: PieceType = 49;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 110,
    description: "붉은 귀부인 능력에 제물 능력 사용 시 카드 1장 드로우를 더합니다.",
    ..PieceDefinition::new(BANQUET_LADY, "banquet_lady", "만찬의 귀부인", movement)
};

// Each piece keeps its complete movement script here. Coordinates are owner-relative.
fn movement() -> String {
    "take-move(1,0) repeat(1);\ntake-move(-1,0) repeat(1);\ntake-move(0,-1) repeat(1);\ntake-move(0,1) repeat(1);\ntake-move(1,1) repeat(1);\ntake-move(1,-1) repeat(1);\ntake-move(-1,1) repeat(1);\ntake-move(-1,-1) repeat(1)".into()
}
