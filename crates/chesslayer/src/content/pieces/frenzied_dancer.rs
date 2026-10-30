use super::PieceDefinition;
use crate::PieceType;

pub const FRENZIED_DANCER: PieceType = 59;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 35,
    description: "나이트 행마. 추가로 상하좌우 인접한 시체만 포획할 수 있습니다.",
    upgrades: &[super::UpgradeDefinition {
        target: super::MASKED_DANCER,
        key: "masked_dancer",
        effect: "가면의 무희(으)로 영구 강화합니다.",
        requirement: "살아 있는 기물 · 쉼터 행동 1회",
    }],
    family: super::PieceFamily::Knight,
    ..PieceDefinition::new(FRENZIED_DANCER, "frenzied_dancer", "광란의 무희", movement)
};

// Each piece keeps its complete movement script here. Coordinates are owner-relative.
fn movement() -> String {
    "take-move(1,2);\ntake-move(2,1);\ntake-move(2,-1);\ntake-move(1,-2);\ntake-move(-1,-2);\ntake-move(-2,-1);\ntake-move(-2,1);\ntake-move(-1,2);\npiece-on(corpse,1,0) take(1,0);\npiece-on(corpse,-1,0) take(-1,0);\npiece-on(corpse,0,-1) take(0,-1);\npiece-on(corpse,0,1) take(0,1)".into()
}
