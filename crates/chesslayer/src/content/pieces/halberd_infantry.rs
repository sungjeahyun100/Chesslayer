//! v0.3 prototype; movement belongs to this piece independently.
use super::PieceDefinition;
use crate::PieceType;
pub const HALBERD_INFANTRY: PieceType = 42;
pub const DEFINITION: PieceDefinition = PieceDefinition {
    description: "상하좌우 1칸, 전방 나이트 방향 4곳, 전방 대각선 1~2칸으로 이동·포획합니다. 대각선 경로의 기물을 통과하지 못합니다.",
    upgrades: &[super::UpgradeDefinition {
        target: super::PISTOL_INFANTRY,
        key: "pistol_infantry",
        effect: "검 보병과 같은 행마에 권총 사격이 추가됩니다. 상하좌우 1칸, 전방 나이트 방향 4곳, 전방 대각선 1칸 이동·포획. 전방 3방향 사격은 거리 1에서 100%, 추가 칸마다 −5%p.",
        requirement: "생존 기물 · 강화 기회 1회",
    }],
    initial_double_step: false,
    ..PieceDefinition::new(
        HALBERD_INFANTRY,
        "halberd_infantry",
        "장창과 도끼로 무장된 보병",
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
take-move(-2, 1);
take-move(1, 1) take-move(1, 1);
take-move(-1, 1) take-move(-1, 1);"#
        .into()
}
