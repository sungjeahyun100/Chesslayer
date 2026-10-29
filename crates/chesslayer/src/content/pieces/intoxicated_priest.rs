//! v0.3 prototype; movement belongs to this piece independently.
use super::PieceDefinition;
use crate::PieceType;
pub const INTOXICATED_PRIEST: PieceType = 61;
pub const DEFINITION: PieceDefinition = PieceDefinition {
    description: "시체 포획 시 인접 다른 아군 민첩 1. 별도 표기가 없는 발동 능력은 전투당 1회.",
    upgrades: &[super::UpgradeDefinition {
        target: super::FEAST_PRIEST,
        key: "feast_priest",
        effect: "시체 포획 시 2칸 내 다른 아군 민첩 1. 별도 표기가 없는 발동 능력은 전투당 1회.",
        requirement: "생존 기물 · 강화 기회 1회",
    }],
    initial_double_step: false,
    ..PieceDefinition::new(
        INTOXICATED_PRIEST,
        "intoxicated_priest",
        "도취의 사제",
        movement,
    )
};
fn movement() -> String {
    r#"take-move(1,1) repeat(1);take-move(-1,1) repeat(1);take-move(1,-1) repeat(1);take-move(-1,-1) repeat(1);"#.into()
}
