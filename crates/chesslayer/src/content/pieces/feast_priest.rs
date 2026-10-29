//! v0.3 prototype; movement belongs to this piece independently.
use super::PieceDefinition;
use crate::PieceType;
pub const FEAST_PRIEST: PieceType = 62;
pub const DEFINITION: PieceDefinition = PieceDefinition {
    description: "시체 포획 시 2칸 내 다른 아군 민첩 1. 별도 표기가 없는 발동 능력은 전투당 1회.",
    upgrades: &[super::UpgradeDefinition {target: super::LOCUST_PRIEST,key:"locust_priest",effect:"시체 포획 시 2칸 내 다른 아군 민첩 2. 별도 표기가 없는 발동 능력은 전투당 1회.",requirement:"생존 기물 · 강화 기회 1회"},super::UpgradeDefinition {target: super::HUNGER_PRIEST,key:"hunger_priest",effect:"향연의 사제 효과와 2칸 내 적 비로얄 허기 악화. 별도 표기가 없는 발동 능력은 전투당 1회.",requirement:"생존 기물 · 강화 기회 1회"}],
    initial_double_step: false,
    ..PieceDefinition::new(FEAST_PRIEST, "feast_priest", "향연의 사제", movement)
};
fn movement() -> String {
    r#"take-move(1,1) repeat(1);take-move(-1,1) repeat(1);take-move(1,-1) repeat(1);take-move(-1,-1) repeat(1);"#.into()
}
