//! v0.3 prototype; movement belongs to this piece independently.
use super::PieceDefinition;
use crate::PieceType;
pub const MASKED_LADY: PieceType = 53;
pub const DEFINITION: PieceDefinition = PieceDefinition {
    description: "인접 아군 신도를 제물로 시체화. 별도 표기가 없는 발동 능력은 전투당 1회.",
    upgrades: &[super::UpgradeDefinition {
        target: super::RED_LADY,
        key: "red_lady",
        effect: "인접 아군 비로얄을 제물로 시체화. 별도 표기가 없는 발동 능력은 전투당 1회.",
        requirement: "생존 기물 · 강화 기회 1회",
    }],
    initial_double_step: false,
    ..PieceDefinition::new(MASKED_LADY, "masked_lady", "가면의 귀부인", movement)
};
fn movement() -> String {
    r#"take-move(1,0) repeat(1);take-move(-1,0) repeat(1);take-move(0,1) repeat(1);take-move(0,-1) repeat(1);take-move(1,1) repeat(1);take-move(-1,1) repeat(1);take-move(1,-1) repeat(1);take-move(-1,-1) repeat(1);"#.into()
}
