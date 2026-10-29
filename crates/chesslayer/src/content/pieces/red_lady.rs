//! v0.3 prototype; movement belongs to this piece independently.
use super::PieceDefinition;
use crate::PieceType;
pub const RED_LADY: PieceType = 54;
pub const DEFINITION: PieceDefinition = PieceDefinition {
    description: "인접 아군 비로얄을 제물로 시체화. 별도 표기가 없는 발동 능력은 전투당 1회.",
    upgrades: &[super::UpgradeDefinition {target: super::BANQUET_LADY,key:"banquet_lady",effect:"제물 능력에 드로우 1 추가. 별도 표기가 없는 발동 능력은 전투당 1회.",requirement:"생존 기물 · 강화 기회 1회"},super::UpgradeDefinition {target: super::INVITING_LADY,key:"inviting_lady",effect:"제물 능력에 2칸 이내 적 비로얄 허기 악화 추가. 별도 표기가 없는 발동 능력은 전투당 1회.",requirement:"생존 기물 · 강화 기회 1회"}],
    initial_double_step: false,
    ..PieceDefinition::new(RED_LADY, "red_lady", "붉은 귀부인", movement)
};
fn movement() -> String {
    r#"take-move(1,0) repeat(1);take-move(-1,0) repeat(1);take-move(0,1) repeat(1);take-move(0,-1) repeat(1);take-move(1,1) repeat(1);take-move(-1,1) repeat(1);take-move(1,-1) repeat(1);take-move(-1,-1) repeat(1);"#.into()
}
