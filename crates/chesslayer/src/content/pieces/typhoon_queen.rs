use super::PieceDefinition;
use crate::PieceType;

pub const TYPHOON_QUEEN: PieceType = 27;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 125,
    description: "템페스트 퀸의 경유·분기 행마에 일반 룩과 비숍의 직선 이동을 더합니다. 포획하면 해당 경로는 멈춥니다.",
    ..PieceDefinition::new(TYPHOON_QUEEN, "typhoon_queen", "태풍 퀸", movement)
};

pub(super) fn movement() -> String {
    let mut chains: Vec<_> = super::movement::DIAGONAL.iter().map(|&(x, y)| {
        format!("take-move({x},{y}) {{ take-move({x},0) repeat(1) }} {{ take-move(0,{y}) repeat(1) }}")
    }).collect();
    chains.extend(super::movement::ORTHOGONAL.iter().map(|&(x, y)| {
        let (a, b) = if x == 0 {
            ((-1, y), (1, y))
        } else {
            ((x, -1), (x, 1))
        };
        format!(
            "take-move({x},{y}) {{ take-move({},{}) repeat(1) }} {{ take-move({},{}) repeat(1) }}",
            a.0, a.1, b.0, b.1
        )
    }));
    chains.extend(
        super::movement::ORTHOGONAL
            .iter()
            .map(|&(x, y)| format!("take-move({x},{y}) repeat(1)")),
    );
    chains.extend(
        super::movement::DIAGONAL
            .iter()
            .map(|&(x, y)| format!("take-move({x},{y}) repeat(1)")),
    );
    chains.join("; ")
}
