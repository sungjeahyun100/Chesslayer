use super::PieceDefinition;
use crate::PieceType;

pub const MASKED_DANCER: PieceType = 60;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 45,
    description: "나이트 행마. 추가 시체 포획 범위가 대각선을 포함한 인접 여덟 칸으로 늘어납니다.",
    upgrades: &[
        super::UpgradeDefinition {
            target: super::ECSTATIC_DANCER,
            key: "ecstatic_dancer",
            effect: "환희의 무희(으)로 영구 강화합니다.",
            requirement: "살아 있는 기물 · 쉼터 행동 1회",
        },
        super::UpgradeDefinition {
            target: super::TEMPTING_DANCER,
            key: "tempting_dancer",
            effect: "유혹의 무희(으)로 영구 강화합니다.",
            requirement: "살아 있는 기물 · 쉼터 행동 1회",
        },
    ],
    family: super::PieceFamily::Knight,
    ..PieceDefinition::new(MASKED_DANCER, "masked_dancer", "가면의 무희", movement)
};

// Each piece keeps its complete movement script here. Coordinates are owner-relative.
fn movement() -> String {
    "take-move(1,2);\ntake-move(2,1);\ntake-move(2,-1);\ntake-move(1,-2);\ntake-move(-1,-2);\ntake-move(-2,-1);\ntake-move(-2,1);\ntake-move(-1,2);\npiece-on(corpse,1,0) take(1,0);\npiece-on(corpse,-1,0) take(-1,0);\npiece-on(corpse,0,-1) take(0,-1);\npiece-on(corpse,0,1) take(0,1);\npiece-on(corpse,1,1) take(1,1);\npiece-on(corpse,1,-1) take(1,-1);\npiece-on(corpse,-1,1) take(-1,1);\npiece-on(corpse,-1,-1) take(-1,-1)".into()
}
