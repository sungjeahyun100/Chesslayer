//! v0.3 prototype; movement belongs to this piece independently.
use super::PieceDefinition;
use crate::PieceType;
pub const DRONE: PieceType = 79;
pub const DEFINITION: PieceDefinition = PieceDefinition {
    description: "임시 나이트 행마 및 직선 사격. 탄약 1.",
    upgrades: &[],
    initial_double_step: false,
    ..PieceDefinition::new(DRONE, "drone", "드론", movement)
};
fn movement() -> String {
    r#"take-move(1,2);take-move(2,1);take-move(-1,2);take-move(-2,1);take-move(1,-2);take-move(2,-1);take-move(-1,-2);take-move(-2,-1);"#.into()
}
