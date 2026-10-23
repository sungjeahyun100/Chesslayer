use super::{movement, PieceDefinition};
use crate::PieceType;

pub const HOOK: PieceType = 17;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 90,
    description: "상하좌우로 직선 이동하며, 경로의 빈 칸에서 한 번 직각으로 꺾을 수 있습니다. 아군에 막히고 적을 포획하면 멈춥니다.",
    ..PieceDefinition::new(HOOK, "hook", "구행", movement)
};

pub(super) fn movement() -> String {
    movement::ORTHOGONAL
        .iter()
        .map(|&(x, y)| {
            // Each empty step offers two perpendicular branches. Child blocks restore
            // the parent's position; a capture stops the parent before either branch.
            format!(
                "do take-move({x},{y}) {{ take-move({},{}) repeat(1) }} {{ take-move({},{}) repeat(1) }} while",
                -y, x, y, -x
            )
        })
        .collect::<Vec<_>>()
        .join(";")
}
