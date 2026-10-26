use super::{movement, PieceDefinition};
use crate::PieceType;

pub const BOUNCING_BISHOP: PieceType = 15;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    family: super::PieceFamily::Bishop,
    value: 70,
    description: "대각선으로 이동합니다. 첫 보드 경계·아군 바운싱 폰 직전에서 가로 또는 세로 방향 성분을 반전해 직진합니다. 반사 뒤 다음 장애물에서 멈춥니다.",
    upgrades: &[
        super::UpgradeDefinition {
            target: super::TENGU,
            key: "tengu",
            effect: super::tengu::DEFINITION.description,
            requirement: "살아 있는 바운싱 비숍 · 쉼터 행동 1회 · 추가 비용 없음",
        },
        super::UpgradeDefinition {
            target: super::CARDINAL,
            key: "cardinal",
            effect: super::cardinal::DEFINITION.description,
            requirement: "살아 있는 바운싱 비숍 · 쉼터 행동 1회 · 추가 비용 없음",
        },
    ],
    ..PieceDefinition::new(BOUNCING_BISHOP, "bouncing_bishop", "바운싱 비숍", movement)
};

pub(super) fn movement() -> String {
    movement::DIAGONAL.iter().map(|&(x, y)| {
        let (a, b) = if x == 0 { ((1, 0), (-1, 0)) }
            else if y == 0 { ((0, 1), (0, -1)) }
            else { ((-x, y), (x, -y)) };
        format!("wall-piece(bouncing_pawn) do take-move({x},{y}) while enemy(0,0) not barrier({x},{y}) {{ take-move({},{}) repeat(1) }} {{ take-move({},{}) repeat(1) }}", a.0, a.1, b.0, b.1)
    }).collect::<Vec<_>>().join("; ")
}
