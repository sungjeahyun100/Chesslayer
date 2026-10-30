use super::{CardDefinition, CardKind, CardSelection};
use crate::{Battle, PieceId};

pub fn definition() -> CardDefinition {
    let mut definition = CardDefinition::new(
        CardKind::EndlessEcstasy, "환희는 끝나지 않는다", 3, "이번 턴 아군 비로얄이 시체를 포획할 때마다 일반 이동·포획을 한 번 더 할 수 있습니다. 전투당 한 번.", effect,
    );
    definition.exhaust = false;
    definition
}

fn effect(battle: &mut Battle, target: Option<PieceId>) -> Result<Option<usize>, String> {
    battle.v030_card_effect(CardKind::EndlessEcstasy, &CardSelection::single(target))
}
