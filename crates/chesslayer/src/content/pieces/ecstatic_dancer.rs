//! v0.3 prototype; movement belongs to this piece independently.
use super::PieceDefinition;
use crate::PieceType;
pub const ECSTATIC_DANCER: PieceType = 67;
pub const DEFINITION: PieceDefinition = PieceDefinition {
    description:
        "8방향 시체 포획. 첫 시체 포획 시 민첩 1 추가. 별도 표기가 없는 발동 능력은 전투당 1회.",
    upgrades: &[],
    initial_double_step: false,
    ..PieceDefinition::new(ECSTATIC_DANCER, "ecstatic_dancer", "환희의 무희", movement)
};
fn movement() -> String {
    r#"take-move(1,2);take-move(2,1);take-move(-1,2);take-move(-2,1);take-move(1,-2);take-move(2,-1);take-move(-1,-2);take-move(-2,-1);"#.into()
}
