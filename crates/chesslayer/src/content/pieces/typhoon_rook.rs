use super::PieceDefinition;
use crate::PieceType;

pub const TYPHOON_ROOK: PieceType = 28;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 90,
    description: "템페스트 룩의 대각 경유·분기 행마와 일반 룩의 직교 직선 이동을 모두 사용합니다.",
    ..PieceDefinition::new(TYPHOON_ROOK, "typhoon_rook", "태풍 룩", movement)
};

pub(super) fn movement() -> String {
    let mut chains: Vec<_> = super::movement::DIAGONAL.iter().map(|&(x, y)| {
        format!("take-move({x},{y}) {{ take-move({x},0) repeat(1) }} {{ take-move(0,{y}) repeat(1) }}")
    }).collect();
    chains.extend(
        super::movement::ORTHOGONAL
            .iter()
            .map(|&(x, y)| format!("take-move({x},{y}) repeat(1)")),
    );
    chains.join("; ")
}
