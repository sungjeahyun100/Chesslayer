use super::{movement, PieceDefinition};
use crate::PieceType;

pub const AMAZON: PieceType = 7;

pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 90,
    description: "퀸의 직선·대각선 이동과 나이트의 도약을 모두 사용합니다. 도약할 때는 중간 기물을 뛰어넘습니다.",
    ..PieceDefinition::new(AMAZON, "amazon", "아마존", movement)
};

fn movement() -> String {
    format!(
        "{};{}",
        movement::slides(&movement::all_directions()),
        movement::steps(movement::KNIGHT)
    )
}
