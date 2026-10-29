//! v0.3 prototype; movement belongs to this piece independently.
use super::PieceDefinition;
use crate::PieceType;
pub const PIKE_INFANTRY: PieceType = 41;
pub const DEFINITION: PieceDefinition = PieceDefinition {
    description: "상하좌우 1칸 또는 전방의 나이트 방향 4곳으로 이동·포획합니다. 나이트 이동은 중간 기물을 뛰어넘습니다.",
    upgrades: &[super::UpgradeDefinition {
        target: super::SWORD_INFANTRY,
        key: "sword_infantry",
        effect: "상하좌우 1칸, 전방 나이트 방향 4곳, 전방 대각선 1칸으로 이동·포획합니다.",
        requirement: "생존 기물 · 강화 기회 1회",
    }],
    initial_double_step: false,
    ..PieceDefinition::new(
        PIKE_INFANTRY,
        "pike_infantry",
        "장창으로 무장된 보병",
        movement,
    )
};
// The game engine already rotates relative coordinates for each side.
// Keep each piece independent; the specification's white/black branches use fixed board directions.
fn movement() -> String {
    r#"take-move(1, 0);
take-move(-1, 0);
take-move(0, -1);
take-move(0, 1);

take-move(1, 2);
take-move(2, 1);
take-move(-1, 2);
take-move(-2, 1);"#
        .into()
}
