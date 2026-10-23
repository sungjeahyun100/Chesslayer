use super::{movement, PieceDefinition};
use crate::PieceType;

pub const TEMPEST_KNIGHT: PieceType = 11;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 50,
    description: "대각선 한 칸에서 멈추거나 같은 방향으로 추가 도약해 (3,2)·(2,3)에 도달합니다. 대각 경유 칸은 차단·포획을 판정합니다. 별도로 직교 세 칸을 뜁니다.",
    ..PieceDefinition::new(TEMPEST_KNIGHT, "tempest_knight", "템페스트 나이트", movement)
};

pub(super) fn movement() -> String {
    let mut chains: Vec<_> = movement::DIAGONAL
        .iter()
        .map(|&(x, y)| {
            format!(
                "take-move({x},{y}) {{ take-move({},{y}) }} {{ take-move({x},{}) }}",
                2 * x,
                2 * y
            )
        })
        .collect();
    chains.extend(
        movement::ORTHOGONAL
            .iter()
            .map(|&(x, y)| format!("take-move({},{})", 3 * x, 3 * y)),
    );
    chains.join("; ")
}
