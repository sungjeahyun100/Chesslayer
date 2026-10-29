//! v0.3 prototype; movement belongs to this piece independently.
use super::PieceDefinition;
use crate::PieceType;
pub const OPERATIVE: PieceType = 82;
pub const DEFINITION: PieceDefinition = PieceDefinition {
    description: "임시 나이트 행마. 향후 교체할 기본 기물.",
    upgrades: &[],
    initial_double_step: false,
    ..PieceDefinition::new(OPERATIVE, "operative", "특수요원", movement)
};
fn movement() -> String {
    r#"take-move(1,2);take-move(2,1);take-move(-1,2);take-move(-2,1);take-move(1,-2);take-move(2,-1);take-move(-1,-2);take-move(-2,-1);"#.into()
}
