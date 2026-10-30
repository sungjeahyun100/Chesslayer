use super::{CardDefinition, CardKind, CardSelection};
use crate::{Battle, PieceId};

pub fn definition() -> CardDefinition {
    let mut definition = CardDefinition::new(
        CardKind::FireDataSharing,
        "사격 제원 공유",
        1,
        "이번 턴 명중한 적을 다시 사격할 때 다음 아군 사격 명중률 +15%p.",
        effect,
    );
    definition.exhaust = false;
    definition
}

fn effect(battle: &mut Battle, target: Option<PieceId>) -> Result<Option<usize>, String> {
    battle.v030_card_effect(CardKind::FireDataSharing, &CardSelection::single(target))
}
