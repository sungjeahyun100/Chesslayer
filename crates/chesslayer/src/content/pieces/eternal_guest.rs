use super::PieceDefinition;
use crate::PieceType;

pub const ETERNAL_GUEST: PieceType = 68;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 110,
    description: "보스 로얄. 양 진영에 식재료, 아군에 황충의 축복을 부여합니다. 아군의 첫 세 시체 포획에 추가 민첩 1. 3수·6수에 표적의 허기를 악화합니다.",
    ..PieceDefinition::new(ETERNAL_GUEST, "eternal_guest", "영원한 주빈", movement)
};

// Each piece keeps its complete movement script here. Coordinates are owner-relative.
fn movement() -> String {
    "take-move(1,0);\ntake-move(-1,0);\ntake-move(0,-1);\ntake-move(0,1);\ntake-move(1,1);\ntake-move(1,-1);\ntake-move(-1,1);\ntake-move(-1,-1)".into()
}
