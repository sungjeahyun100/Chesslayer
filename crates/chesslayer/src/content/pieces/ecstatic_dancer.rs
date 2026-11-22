use super::PieceDefinition;
use crate::PieceType;

pub const ECSTATIC_DANCER: PieceType = 61;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 55,
    description: "나이트 행마와 주변 8칸 시체 포획. 시체 섭취의 부정적 효과를 받지 않고 포획 시 행마권 2를 얻습니다.",
    family: super::PieceFamily::Knight,
    ..PieceDefinition::new(ECSTATIC_DANCER, "ecstatic_dancer", "환희의 무희", movement)
};

// Each piece keeps its complete movement script here. Coordinates are owner-relative.
fn movement() -> String {
    "take-move(1,2);\ntake-move(2,1);\ntake-move(2,-1);\ntake-move(1,-2);\ntake-move(-1,-2);\ntake-move(-2,-1);\ntake-move(-2,1);\ntake-move(-1,2);\npiece-on(corpse,1,0) take(1,0);\npiece-on(corpse,-1,0) take(-1,0);\npiece-on(corpse,0,-1) take(0,-1);\npiece-on(corpse,0,1) take(0,1);\npiece-on(corpse,1,1) take(1,1);\npiece-on(corpse,1,-1) take(1,-1);\npiece-on(corpse,-1,1) take(-1,1);\npiece-on(corpse,-1,-1) take(-1,-1)".into()
}
