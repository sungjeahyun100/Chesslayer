//! v0.3 prototype; movement belongs to this piece independently.
use super::PieceDefinition;
use crate::PieceType;
pub const PISTOL_INFANTRY: PieceType = 48;
pub const DEFINITION: PieceDefinition = PieceDefinition {
    description: "검 보병과 같은 행마에 권총 사격이 추가됩니다. 상하좌우 1칸, 전방 나이트 방향 4곳, 전방 대각선 1칸 이동·포획. 전방 3방향 사격은 거리 1에서 100%, 추가 칸마다 −5%p.",
    upgrades: &[],
    initial_double_step: false,
    ..PieceDefinition::new(
        PISTOL_INFANTRY,
        "pistol_infantry",
        "권총과 냉병기로 무장된 보병",
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
take-move(1, 1);
take-move(-1, 1);"#
        .into()
}
