use super::PieceDefinition;
use crate::PieceType;

pub const ECSTATIC_LORD: PieceType = 46;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 100,
    description: "로얄. 양 진영의 모든 기물에 식재료를 부여합니다. 전투당 1회 아군의 시체 포획 시 카드 1장, 민첩 1, 마나 1을 얻습니다.",
    upgrades: &[super::UpgradeDefinition { target: super::ETERNAL_GUEST, key: "eternal_guest", effect: super::eternal_guest::DEFINITION.description, requirement: "살아 있는 환희의 군주 · 쉼터 행동 1회" }],
    ..PieceDefinition::new(ECSTATIC_LORD, "ecstatic_lord", "환희의 군주", movement)
};

// Each piece keeps its complete movement script here. Coordinates are owner-relative.
fn movement() -> String {
    "take-move(1,0);\ntake-move(-1,0);\ntake-move(0,-1);\ntake-move(0,1);\ntake-move(1,1);\ntake-move(1,-1);\ntake-move(-1,1);\ntake-move(-1,-1)".into()
}
