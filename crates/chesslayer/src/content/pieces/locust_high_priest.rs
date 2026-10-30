use super::PieceDefinition;
use crate::PieceType;

pub const LOCUST_HIGH_PRIEST: PieceType = 57;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 55,
    description: "향연의 사제 능력의 민첩 부여량이 2가 됩니다.",
    family: super::PieceFamily::Bishop,
    ..PieceDefinition::new(
        LOCUST_HIGH_PRIEST,
        "locust_high_priest",
        "황충의 대사제",
        movement,
    )
};

// Each piece keeps its complete movement script here. Coordinates are owner-relative.
fn movement() -> String {
    "take-move(1,1) repeat(1);\ntake-move(1,-1) repeat(1);\ntake-move(-1,1) repeat(1);\ntake-move(-1,-1) repeat(1)".into()
}
