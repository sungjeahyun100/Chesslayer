//! v0.3 prototype; movement belongs to this piece independently.
use super::PieceDefinition;
use crate::PieceType;
pub const PROCESSION_FOLLOWER: PieceType = 69;
pub const DEFINITION: PieceDefinition = PieceDefinition {
    description: "폰 행마. 프로모션 미정. 별도 표기가 없는 발동 능력은 전투당 1회.",
    upgrades: &[super::UpgradeDefinition {
        target: super::HUNGRY_FOLLOWER,
        key: "hungry_follower",
        effect: "폰 행마 + 전방 1칸 시체 포획. 별도 표기가 없는 발동 능력은 전투당 1회.",
        requirement: "생존 기물 · 강화 기회 1회",
    }],
    initial_double_step: true,
    ..PieceDefinition::new(
        PROCESSION_FOLLOWER,
        "procession_follower",
        "행렬의 신도",
        movement,
    )
};
fn movement() -> String {
    r#"move(0,1); enemy(-1,1) take(-1,1); enemy(1,1) take(1,1);"#.into()
}
