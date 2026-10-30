use super::{CardDefinition, CardKind, CardSelection};
use crate::{Battle, PieceId};

pub fn definition() -> CardDefinition {
    let mut definition = CardDefinition::new(
        CardKind::HumanWave,
        "인해전술",
        2,
        "이번 턴 아군이 적을 포획하면 다른 아군 하나를 추가 이동시킬 수 있습니다. 최대 2회.",
        effect,
    );
    definition.exhaust = false;
    definition
}

fn effect(battle: &mut Battle, target: Option<PieceId>) -> Result<Option<usize>, String> {
    battle.v030_card_effect(CardKind::HumanWave, &CardSelection::single(target))
}
