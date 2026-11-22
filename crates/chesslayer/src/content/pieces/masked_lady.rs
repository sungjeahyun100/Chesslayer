use super::PieceDefinition;
use crate::PieceType;

pub const MASKED_LADY: PieceType = 47;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 90,
    description:
        "퀸 행마. 턴당 1회 약자 살해로 주변 8칸의 양측 폰 하나를 제자리에서 시체로 만듭니다.",
    upgrades: &[super::UpgradeDefinition {
        target: super::RED_LADY,
        key: "red_lady",
        effect: "붉은 귀부인(으)로 영구 강화합니다.",
        requirement: "살아 있는 기물 · 쉼터 행동 1회",
    }],
    ..PieceDefinition::new(MASKED_LADY, "masked_lady", "가면의 귀부인", movement)
};

// Each piece keeps its complete movement script here. Coordinates are owner-relative.
fn movement() -> String {
    "take-move(1,0) repeat(1);\ntake-move(-1,0) repeat(1);\ntake-move(0,-1) repeat(1);\ntake-move(0,1) repeat(1);\ntake-move(1,1) repeat(1);\ntake-move(1,-1) repeat(1);\ntake-move(-1,1) repeat(1);\ntake-move(-1,-1) repeat(1)".into()
}
