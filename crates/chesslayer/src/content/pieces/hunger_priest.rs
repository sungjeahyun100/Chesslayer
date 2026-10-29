//! v0.3 prototype; movement belongs to this piece independently.
use super::PieceDefinition;
use crate::PieceType;
pub const HUNGER_PRIEST: PieceType = 64;
pub const DEFINITION: PieceDefinition = PieceDefinition {
    description:
        "향연의 사제 효과와 2칸 내 적 비로얄 허기 악화. 별도 표기가 없는 발동 능력은 전투당 1회.",
    upgrades: &[],
    initial_double_step: false,
    ..PieceDefinition::new(HUNGER_PRIEST, "hunger_priest", "허기의 사제", movement)
};
fn movement() -> String {
    r#"take-move(1,1) repeat(1);take-move(-1,1) repeat(1);take-move(1,-1) repeat(1);take-move(-1,-1) repeat(1);"#.into()
}
