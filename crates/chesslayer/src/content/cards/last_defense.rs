use super::{CardDefinition, CardKind, CardSelection};
use crate::{Battle, PieceId};

pub fn definition() -> CardDefinition {
    let mut definition = CardDefinition::new(
        CardKind::LastDefense,
        "최후의 방어선",
        2,
        "다음 자신의 턴 전까지 아군 포획 시 사격 가능한 다른 아군 하나가 즉시 반격 사격합니다.",
        effect,
    );
    definition.exhaust = false;
    definition
}

fn effect(battle: &mut Battle, target: Option<PieceId>) -> Result<Option<usize>, String> {
    battle.v030_card_effect(CardKind::LastDefense, &CardSelection::single(target))
}
