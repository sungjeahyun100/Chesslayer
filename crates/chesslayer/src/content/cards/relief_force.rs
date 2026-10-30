use super::{CardDefinition, CardKind, CardSelection};
use crate::{Battle, PieceId};

pub fn definition() -> CardDefinition {
    let mut definition = CardDefinition::new(
        CardKind::ReliefForce,
        "교대 병력",
        2,
        "다음 자신의 턴 전까지 첫 아군 포획 시 아군 진영의 빈칸에 임시 폰을 배치합니다.",
        effect,
    );
    definition.exhaust = false;
    definition
}

fn effect(battle: &mut Battle, target: Option<PieceId>) -> Result<Option<usize>, String> {
    battle.v030_card_effect(CardKind::ReliefForce, &CardSelection::single(target))
}
