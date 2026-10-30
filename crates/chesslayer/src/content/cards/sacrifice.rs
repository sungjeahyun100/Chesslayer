use super::{CardDefinition, CardKind, CardSelection, SelectionMode};
use crate::{Battle, PieceId};

pub fn definition() -> CardDefinition {
    let mut definition = CardDefinition::new(
        CardKind::Sacrifice,
        "희생",
        0,
        "아군 비로얄 하나를 희생하고 마나 3을 얻습니다.",
        effect,
    )
    .with_selection(SelectionMode::Single, targets, selected_effect);
    definition.exhaust = false;
    definition
}

fn effect(battle: &mut Battle, target: Option<PieceId>) -> Result<Option<usize>, String> {
    battle.v030_card_effect(CardKind::Sacrifice, &CardSelection::single(target))
}

fn targets(battle: &Battle) -> Vec<PieceId> {
    battle.v030_card_targets(CardKind::Sacrifice)
}

fn selected_effect(
    battle: &mut Battle,
    selection: &CardSelection,
) -> Result<Option<usize>, String> {
    battle.v030_card_effect(CardKind::Sacrifice, selection)
}
