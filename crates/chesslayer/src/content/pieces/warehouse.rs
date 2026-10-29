//! v0.3 prototype; movement belongs to this piece independently.
use super::PieceDefinition;
use crate::PieceType;
pub const WAREHOUSE: PieceType = 76;
pub const DEFINITION: PieceDefinition = PieceDefinition {
    description: "탄약 한계 +200. 파괴 시 탄약 −200, 상하좌우 폭발. 모든 창고 소실 시 패배.",
    upgrades: &[],
    initial_double_step: false,
    ..PieceDefinition::new(WAREHOUSE, "warehouse", "창고", movement)
};
fn movement() -> String {
    r#""#.into()
}
