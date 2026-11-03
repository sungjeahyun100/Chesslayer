use super::{movement, PieceDefinition};
use crate::PieceType;

pub const JOCKEY: PieceType = 93;
pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 10,
    description: "킹의 기본 행마와 도박꾼의 주사위 행마를 함께 사용합니다. 매 전투 시작과 자신의 턴 시작에 1 폰, 2 나이트, 3 비숍, 4 룩, 5 퀸, 6 킹 행마를 굴립니다. 상대 진영 끝 랭크에 도달하면 승리합니다.",
    ..PieceDefinition::new(JOCKEY, "jockey", "기수가 된 경마사", movement)
};

fn movement() -> String {
    movement::steps(&movement::all_directions())
}
