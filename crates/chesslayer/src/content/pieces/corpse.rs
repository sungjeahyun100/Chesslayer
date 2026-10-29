//! v0.3 prototype; movement belongs to this piece independently.
use super::PieceDefinition;
use crate::PieceType;
pub const CORPSE: PieceType = 75;
pub const DEFINITION: PieceDefinition = PieceDefinition {
    description: "움직일 수 없으며 양 진영 모두 포획 가능.",
    upgrades: &[],
    initial_double_step: false,
    ..PieceDefinition::new(CORPSE, "corpse", "시체", movement)
};
fn movement() -> String {
    r#""#.into()
}
