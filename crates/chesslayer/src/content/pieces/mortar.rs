//! v0.3 prototype; movement belongs to this piece independently.
use super::PieceDefinition;
use crate::PieceType;
pub const MORTAR: PieceType = 78;
pub const DEFINITION: PieceDefinition = PieceDefinition {
    description: "임시 비숍 행마 및 직선 사격. 탄약 1.",
    upgrades: &[],
    initial_double_step: false,
    ..PieceDefinition::new(MORTAR, "mortar", "박격포병", movement)
};
fn movement() -> String {
    r#"take-move(1,1) repeat(1);take-move(-1,1) repeat(1);take-move(1,-1) repeat(1);take-move(-1,-1) repeat(1);"#.into()
}
