use super::{movement, PieceDefinition};
use crate::PieceType;

pub const TEMPEST_BISHOP: PieceType = 10;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 50,
    description: "직교 한 칸 이동·포획 후 전진 방향의 양쪽 대각선으로 계속 이동합니다. 중간 아군은 차단하고 적을 포획하면 멈춥니다.",
    ..PieceDefinition::new(TEMPEST_BISHOP, "tempest_bishop", "템페스트 비숍", movement)
};

pub(super) fn movement() -> String {
    movement::ORTHOGONAL.iter().map(|&(x,y)| {
        let (a,b) = if x == 0 { ((-1,y),(1,y)) } else { ((x,-1),(x,1)) };
        format!("take-move({x},{y}) {{ take-move({},{}) repeat(1) }} {{ take-move({},{}) repeat(1) }}", a.0,a.1,b.0,b.1)
    }).collect::<Vec<_>>().join("; ")
}
