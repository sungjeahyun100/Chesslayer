use super::{CardDefinition, CardKind, CardSelection, SelectionMode};
use crate::{Battle, PieceId};

pub fn definition() -> CardDefinition {
    let mut definition = CardDefinition::new(
        CardKind::Expendables,
        "소모품",
        0,
        "아군 비로얄 기물 하나를 희생하고 카드 3장을 뽑습니다.",
        effect,
    )
    .with_selection(SelectionMode::Single, targets, selected_effect);
    definition.exhaust = false;
    definition
}

fn effect(battle: &mut Battle, target: Option<PieceId>) -> Result<Option<usize>, String> {
    battle.v030_card_effect(CardKind::Expendables, &CardSelection::single(target))
}

fn targets(battle: &Battle) -> Vec<PieceId> {
    battle.v030_card_targets(CardKind::Expendables)
}
fn selected_effect(
    battle: &mut Battle,
    selection: &CardSelection,
) -> Result<Option<usize>, String> {
    battle.v030_card_effect(CardKind::Expendables, selection)
}
