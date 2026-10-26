use super::{movement, PieceDefinition};
use crate::PieceType;

pub const BOUNCING_ROOK: PieceType = 14;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 60,
    description: "직교로 이동합니다. 첫 보드 경계·아군 바운싱 폰 직전에서 양쪽 직교 방향으로 꺾어 직진합니다. 반사 뒤 다음 장애물에서 멈추며, 일반 적은 포획할 수 있습니다.",
    upgrades: &[super::UpgradeDefinition {
        target: super::HOOK,
        key: "hook",
        effect: super::hook::DEFINITION.description,
        requirement: "살아 있는 바운싱 룩 · 쉼터 행동 1회 · 추가 비용 없음",
    }],
    ..PieceDefinition::new(BOUNCING_ROOK, "bouncing_rook", "바운싱 룩", movement)
};

pub(super) fn movement() -> String {
    movement::ORTHOGONAL.iter().map(|&(x, y)| {
        let (a, b) = if x == 0 { ((1, 0), (-1, 0)) }
            else if y == 0 { ((0, 1), (0, -1)) }
            else { ((-x, y), (x, -y)) };
        format!("wall-piece(bouncing_pawn) do take-move({x},{y}) while enemy(0,0) not barrier({x},{y}) {{ take-move({},{}) repeat(1) }} {{ take-move({},{}) repeat(1) }}", a.0, a.1, b.0, b.1)
    }).collect::<Vec<_>>().join("; ")
}
