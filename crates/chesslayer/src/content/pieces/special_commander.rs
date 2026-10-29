//! v0.3 prototype; movement belongs to this piece independently.
use super::PieceDefinition;
use crate::PieceType;
pub const SPECIAL_COMMANDER: PieceType = 81;
pub const DEFINITION: PieceDefinition = PieceDefinition {
    description: "임시 로얄. 킹 행마.",
    upgrades: &[],
    initial_double_step: false,
    ..PieceDefinition::new(
        SPECIAL_COMMANDER,
        "special_commander",
        "특수부대 지휘관",
        movement,
    )
};
fn movement() -> String {
    r#"take-move(1,0);take-move(-1,0);take-move(0,1);take-move(0,-1);take-move(1,1);take-move(-1,1);take-move(1,-1);take-move(-1,-1);"#.into()
}
