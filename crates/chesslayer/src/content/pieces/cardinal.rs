use super::{movement, PieceDefinition};
use crate::PieceType;

pub const CARDINAL: PieceType = 22;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    family: super::PieceFamily::Bishop,
    value: 100,
    description: "대각선으로 이동하며 보드 경계와 바운싱 폰에서 가로·세로 성분 중 하나를 반전해 횟수 제한 없이 반사합니다. 적을 포획하면 멈춥니다.",
    ..PieceDefinition::new(CARDINAL, "cardinal", "추기경", movement)
};

fn movement() -> String {
    movement::DIAGONAL
        .iter()
        .map(|&(x, y)| format!("wall-piece(bouncing_pawn) reflect-ray({x},{y})"))
        .collect::<Vec<_>>()
        .join(";")
}
