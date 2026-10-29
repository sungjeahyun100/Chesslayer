//! v0.3 prototype; movement belongs to this piece independently.
use super::PieceDefinition;
use crate::PieceType;
pub const RIFLEMAN: PieceType = 77;
pub const DEFINITION: PieceDefinition = PieceDefinition {
    description: "임시 폰 행마 및 전방 사격. 탄약 1.",
    upgrades: &[],
    initial_double_step: false,
    ..PieceDefinition::new(RIFLEMAN, "rifleman", "소총수", movement)
};
fn movement() -> String {
    r#"move(0,1); enemy(-1,1) take(-1,1); enemy(1,1) take(1,1);"#.into()
}
