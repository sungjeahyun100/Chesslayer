use super::{movement, PieceDefinition, UpgradeDefinition};
use crate::PieceType;

pub const GAMBLER: PieceType = 92;
pub const DEFINITION: PieceDefinition = PieceDefinition {
    value: 10,
    description: "매 전투 시작과 자신의 턴 시작에 주사위를 굴립니다. 1 폰, 2 나이트, 3 비숍, 4 룩, 5 퀸, 6 킹 행마를 그 턴에 사용합니다.",
    upgrades: &[UpgradeDefinition {
        target: super::JOCKEY,
        key: "jockey",
        effect: super::jockey::DEFINITION.description,
        requirement: "살아 있는 도박꾼 · 쉼터 행동 1회",
    }],
    ..PieceDefinition::new(GAMBLER, "gambler", "도박꾼", movement)
};

fn movement() -> String {
    movement::steps(&movement::all_directions())
}
