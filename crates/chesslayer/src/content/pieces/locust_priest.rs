//! v0.3 prototype; movement belongs to this piece independently.
use super::PieceDefinition;
use crate::PieceType;
pub const LOCUST_PRIEST: PieceType = 63;
pub const DEFINITION: PieceDefinition = PieceDefinition {
    description: "시체 포획 시 2칸 내 다른 아군 민첩 2. 별도 표기가 없는 발동 능력은 전투당 1회.",
    upgrades: &[],
    initial_double_step: false,
    ..PieceDefinition::new(LOCUST_PRIEST, "locust_priest", "황충의 대사제", movement)
};
fn movement() -> String {
    r#"take-move(1,1) repeat(1);take-move(-1,1) repeat(1);take-move(1,-1) repeat(1);take-move(-1,-1) repeat(1);"#.into()
}
