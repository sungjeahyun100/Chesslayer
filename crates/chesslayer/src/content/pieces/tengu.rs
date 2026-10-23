use super::{movement, PieceDefinition};
use crate::PieceType;

pub const TENGU: PieceType = 21;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    family: super::PieceFamily::Bishop,
    value: 90,
    description: "대각선으로 이동한 빈 경유지에서 다시 대각 방향을 골라 이동합니다. 경유지에 멈출 수 있고 포획 뒤에는 이어가지 않습니다.",
    ..PieceDefinition::new(TENGU, "tengu", "텐쿠(天狗)", movement)
};

pub(super) fn movement() -> String {
    movement::DIAGONAL
        .iter()
        .map(|&(x, y)| {
            let branches = movement::DIAGONAL
                .iter()
                .map(|&(bx, by)| format!("{{ take-move({bx},{by}) repeat(1) }}"))
                .collect::<Vec<_>>()
                .join(" ");
            format!("do take-move({x},{y}) {branches} while")
        })
        .collect::<Vec<_>>()
        .join(";")
}
