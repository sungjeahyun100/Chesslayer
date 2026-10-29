//! v0.3 prototype; movement belongs to this piece independently.
use super::PieceDefinition;
use crate::PieceType;
pub const TANK: PieceType = 80;
pub const DEFINITION: PieceDefinition = PieceDefinition {
    description: "임시 룩 행마 및 직선 사격. 탄약 1.",
    upgrades: &[],
    initial_double_step: false,
    ..PieceDefinition::new(TANK, "tank", "전차", movement)
};
fn movement() -> String {
    r#"take-move(1,0) repeat(1);take-move(-1,0) repeat(1);take-move(0,1) repeat(1);take-move(0,-1) repeat(1);"#.into()
}
