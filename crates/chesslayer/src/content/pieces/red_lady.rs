use super::PieceDefinition;
use crate::PieceType;

pub const RED_LADY: PieceType = 48;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 100,
    description:
        "퀸 행마. 전투당 1회, 인접한 아군 비로얄을 제자리에서 시체로 만듭니다. 행동 1회 소모.",
    upgrades: &[
        super::UpgradeDefinition {
            target: super::BANQUET_LADY,
            key: "banquet_lady",
            effect: "만찬의 귀부인(으)로 영구 강화합니다.",
            requirement: "살아 있는 기물 · 쉼터 행동 1회",
        },
        super::UpgradeDefinition {
            target: super::INVITING_LADY,
            key: "inviting_lady",
            effect: "초대의 귀부인(으)로 영구 강화합니다.",
            requirement: "살아 있는 기물 · 쉼터 행동 1회",
        },
    ],
    ..PieceDefinition::new(RED_LADY, "red_lady", "붉은 귀부인", movement)
};

// Each piece keeps its complete movement script here. Coordinates are owner-relative.
fn movement() -> String {
    "take-move(1,0) repeat(1);\ntake-move(-1,0) repeat(1);\ntake-move(0,-1) repeat(1);\ntake-move(0,1) repeat(1);\ntake-move(1,1) repeat(1);\ntake-move(1,-1) repeat(1);\ntake-move(-1,1) repeat(1);\ntake-move(-1,-1) repeat(1)".into()
}
