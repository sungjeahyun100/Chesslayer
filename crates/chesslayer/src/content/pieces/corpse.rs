use super::PieceDefinition;
use crate::PieceType;

pub const CORPSE: PieceType = 67;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 1,
    description: "이동 불가. 양 진영 모두 포획할 수 있습니다. 축복받은 포획자는 민첩 3, 나머지는 광기 또는 죄책감 1을 얻으며 허기를 해소합니다.",
    ..PieceDefinition::new(CORPSE, "corpse", "시체", movement)
};

// Each piece keeps its complete movement script here. Coordinates are owner-relative.
fn movement() -> String {
    "false".into()
}
