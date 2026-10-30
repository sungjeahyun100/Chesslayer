use super::{CardDefinition, CardKind, CardSelection, SelectionMode};
use crate::{Battle, PieceId};

pub fn definition() -> CardDefinition {
    let mut definition = CardDefinition::new(
        CardKind::Masquerade,
        "가면무도회",
        2,
        "아군 비로얄 두 기물의 위치를 교환합니다. 소멸.",
        effect,
    )
    .with_selection(SelectionMode::Multiple, targets, selected_effect);
    definition.exhaust = true;
    definition
}

fn effect(battle: &mut Battle, target: Option<PieceId>) -> Result<Option<usize>, String> {
    battle.v030_card_effect(CardKind::Masquerade, &CardSelection::single(target))
}

fn targets(battle: &Battle) -> Vec<PieceId> {
    battle.v030_card_targets(CardKind::Masquerade)
}

fn selected_effect(
    battle: &mut Battle,
    selection: &CardSelection,
) -> Result<Option<usize>, String> {
    battle.v030_card_effect(CardKind::Masquerade, selection)
}
