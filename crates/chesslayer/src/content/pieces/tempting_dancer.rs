use super::PieceDefinition;
use crate::PieceType;

pub const TEMPTING_DANCER: PieceType = 62;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 55,
    description: "가면의 무희 행마 유지. 전투당 1회, 시체 포획 후 인접 적 비로얄 하나의 허기를 한 단계 악화합니다.",
    family: super::PieceFamily::Knight,
    ..PieceDefinition::new(TEMPTING_DANCER, "tempting_dancer", "유혹의 무희", movement)
};

// Each piece keeps its complete movement script here. Coordinates are owner-relative.
fn movement() -> String {
    "take-move(1,2);\ntake-move(2,1);\ntake-move(2,-1);\ntake-move(1,-2);\ntake-move(-1,-2);\ntake-move(-2,-1);\ntake-move(-2,1);\ntake-move(-1,2);\npiece-on(corpse,1,0) take(1,0);\npiece-on(corpse,-1,0) take(-1,0);\npiece-on(corpse,0,-1) take(0,-1);\npiece-on(corpse,0,1) take(0,1);\npiece-on(corpse,1,1) take(1,1);\npiece-on(corpse,1,-1) take(1,-1);\npiece-on(corpse,-1,1) take(-1,1);\npiece-on(corpse,-1,-1) take(-1,-1)".into()
}
