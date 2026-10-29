//! v0.3 prototype; movement belongs to this piece independently.
use super::PieceDefinition;
use crate::PieceType;
pub const MASKED_DANCER: PieceType = 66;
pub const DEFINITION: PieceDefinition = PieceDefinition {
    description: "나이트 행마 + 8방향 인접 시체 포획. 별도 표기가 없는 발동 능력은 전투당 1회.",
    upgrades: &[super::UpgradeDefinition {target: super::ECSTATIC_DANCER,key:"ecstatic_dancer",effect:"8방향 시체 포획. 첫 시체 포획 시 민첩 1 추가. 별도 표기가 없는 발동 능력은 전투당 1회.",requirement:"생존 기물 · 강화 기회 1회"},super::UpgradeDefinition {target: super::TEMPTING_DANCER,key:"tempting_dancer",effect:"8방향 시체 포획. 시체 포획 시 인접 적 비로얄 허기 악화. 별도 표기가 없는 발동 능력은 전투당 1회.",requirement:"생존 기물 · 강화 기회 1회"}],
    initial_double_step: false,
    ..PieceDefinition::new(MASKED_DANCER, "masked_dancer", "가면의 무희", movement)
};
fn movement() -> String {
    r#"take-move(1,2);take-move(2,1);take-move(-1,2);take-move(-2,1);take-move(1,-2);take-move(2,-1);take-move(-1,-2);take-move(-2,-1);"#.into()
}
