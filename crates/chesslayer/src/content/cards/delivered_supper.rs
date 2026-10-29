use super::{CardDefinition, CardKind, CardSelection, SelectionMode};
use crate::{Battle, PieceId};

pub fn definition() -> CardDefinition {
    let mut definition = CardDefinition::new(
        CardKind::DeliveredSupper, "배달된 만찬", 2, "아군 하나와 그 기물에서 두 칸 이내 시체를 선택합니다. 이동 없이 시체를 포획하고 시체 포획 효과를 받습니다.", effect,
    ).with_selection(SelectionMode::Multiple, targets, selected_effect);
    definition.exhaust = false;
    definition
}

fn effect(battle: &mut Battle, target: Option<PieceId>) -> Result<Option<usize>, String> {
    battle.v030_card_effect(CardKind::DeliveredSupper, &CardSelection::single(target))
}

fn targets(battle: &Battle) -> Vec<PieceId> {
    battle.v030_card_targets(CardKind::DeliveredSupper)
}

fn selected_effect(
    battle: &mut Battle,
    selection: &CardSelection,
) -> Result<Option<usize>, String> {
    battle.v030_card_effect(CardKind::DeliveredSupper, selection)
}
