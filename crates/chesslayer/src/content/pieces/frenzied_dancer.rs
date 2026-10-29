//! v0.3 prototype; movement belongs to this piece independently.
use super::PieceDefinition;
use crate::PieceType;
pub const FRENZIED_DANCER: PieceType = 65;
pub const DEFINITION: PieceDefinition = PieceDefinition {
    description: "나이트 행마 + 상하좌우 인접 시체 포획. 별도 표기가 없는 발동 능력은 전투당 1회.",
    upgrades: &[super::UpgradeDefinition {
        target: super::MASKED_DANCER,
        key: "masked_dancer",
        effect: "나이트 행마 + 8방향 인접 시체 포획. 별도 표기가 없는 발동 능력은 전투당 1회.",
        requirement: "생존 기물 · 강화 기회 1회",
    }],
    initial_double_step: false,
    ..PieceDefinition::new(FRENZIED_DANCER, "frenzied_dancer", "광란의 무희", movement)
};
fn movement() -> String {
    r#"take-move(1,2);take-move(2,1);take-move(-1,2);take-move(-2,1);take-move(1,-2);take-move(2,-1);take-move(-1,-2);take-move(-2,-1);"#.into()
}
