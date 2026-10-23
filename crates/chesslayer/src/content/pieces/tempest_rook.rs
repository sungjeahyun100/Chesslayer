use super::{movement, PieceDefinition};
use crate::PieceType;

pub const TEMPEST_ROOK: PieceType = 9;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 80,
    description: "대각선 한 칸 이동·포획 후 그 방향의 가로 또는 세로로 계속 이동합니다. 중간 아군은 차단하고 적을 포획하면 멈춥니다.",
    ..PieceDefinition::new(TEMPEST_ROOK, "tempest_rook", "템페스트 룩", movement)
};

pub(super) fn movement() -> String {
    movement::DIAGONAL
        .iter()
        .map(|&(x, y)| {
            format!(
        "take-move({x},{y}) {{ take-move({x},0) repeat(1) }} {{ take-move(0,{y}) repeat(1) }}"
    )
        })
        .collect::<Vec<_>>()
        .join("; ")
}
