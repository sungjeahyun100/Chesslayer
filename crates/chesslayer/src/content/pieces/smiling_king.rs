//! v0.3 prototype; movement belongs to this piece independently.
use super::PieceDefinition;
use crate::PieceType;
pub const SMILING_KING: PieceType = 50;
pub const DEFINITION: PieceDefinition = PieceDefinition {
    description: "2칸 이내 아군 시체 포획 시 카드 1장. 별도 표기가 없는 발동 능력은 전투당 1회.",
    upgrades: &[super::UpgradeDefinition {target: super::TOAST_KING,key:"toast_king",effect:"3칸 이내 아군 시체 포획 시 드로우 1, 포획자 민첩 1. 별도 표기가 없는 발동 능력은 전투당 1회.",requirement:"생존 기물 · 강화 기회 1회"}],
    initial_double_step: false,
    ..PieceDefinition::new(SMILING_KING, "smiling_king", "웃는 왕", movement)
};
fn movement() -> String {
    r#"take-move(1,0);take-move(-1,0);take-move(0,1);take-move(0,-1);take-move(1,1);take-move(-1,1);take-move(1,-1);take-move(-1,-1);"#.into()
}
