use super::PieceDefinition;
use crate::PieceType;

pub const TYPHOON_BISHOP: PieceType = 29;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    family: super::PieceFamily::Bishop,
    value: 65,
    description:
        "템페스트 비숍의 직교 경유·대각 분기 행마와 일반 비숍의 대각 직선 이동을 모두 사용합니다.",
    ..PieceDefinition::new(TYPHOON_BISHOP, "typhoon_bishop", "태풍 비숍", movement)
};

pub(super) fn movement() -> String {
    let mut chains: Vec<_> = super::movement::ORTHOGONAL.iter().map(|&(x, y)| {
        let (a, b) = if x == 0 { ((-1, y), (1, y)) } else { ((x, -1), (x, 1)) };
        format!("take-move({x},{y}) {{ take-move({},{}) repeat(1) }} {{ take-move({},{}) repeat(1) }}", a.0, a.1, b.0, b.1)
    }).collect();
    chains.extend(
        super::movement::DIAGONAL
            .iter()
            .map(|&(x, y)| format!("take-move({x},{y}) repeat(1)")),
    );
    chains.join("; ")
}
