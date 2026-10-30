use super::PieceDefinition;
use crate::PieceType;

pub const HUNGER_PRIEST: PieceType = 58;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 55,
    description: "향연의 사제 능력 유지. 전투당 1회, 시체 포획 시 두 칸 이내 적 비로얄 하나의 허기를 한 단계 악화합니다.",
    family: super::PieceFamily::Bishop,
    ..PieceDefinition::new(HUNGER_PRIEST, "hunger_priest", "허기의 사제", movement)
};

// Each piece keeps its complete movement script here. Coordinates are owner-relative.
fn movement() -> String {
    "take-move(1,1) repeat(1);\ntake-move(1,-1) repeat(1);\ntake-move(-1,1) repeat(1);\ntake-move(-1,-1) repeat(1)".into()
}
