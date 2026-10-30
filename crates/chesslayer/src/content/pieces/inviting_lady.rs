use super::PieceDefinition;
use crate::PieceType;

pub const INVITING_LADY: PieceType = 50;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 110,
    description: "붉은 귀부인 능력에 제물 능력 사용 시 두 칸 이내 적 비로얄 하나의 허기 한 단계 악화를 더합니다.",
    ..PieceDefinition::new(INVITING_LADY, "inviting_lady", "초대의 귀부인", movement)
};

// Each piece keeps its complete movement script here. Coordinates are owner-relative.
fn movement() -> String {
    "take-move(1,0) repeat(1);\ntake-move(-1,0) repeat(1);\ntake-move(0,-1) repeat(1);\ntake-move(0,1) repeat(1);\ntake-move(1,1) repeat(1);\ntake-move(1,-1) repeat(1);\ntake-move(-1,1) repeat(1);\ntake-move(-1,-1) repeat(1)".into()
}
