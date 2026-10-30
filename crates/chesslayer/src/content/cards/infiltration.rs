use super::{CardDefinition, CardKind, CardSelection, SelectionMode};
use crate::{Battle, PieceId};

pub fn definition() -> CardDefinition {
    let mut definition = CardDefinition::new(
        CardKind::Infiltration,
        "침투",
        2,
        "아군 비로얄 하나와 두 칸 이내 빈 칸을 선택해 이동합니다. 중간 기물을 뛰어넘습니다.",
        effect,
    )
    .with_selection(SelectionMode::Multiple, targets, selected_effect);
    definition.exhaust = false;
    definition
}

fn effect(battle: &mut Battle, target: Option<PieceId>) -> Result<Option<usize>, String> {
    battle.v030_card_effect(CardKind::Infiltration, &CardSelection::single(target))
}

fn targets(battle: &Battle) -> Vec<PieceId> {
    battle.v030_card_targets(CardKind::Infiltration)
}

fn selected_effect(
    battle: &mut Battle,
    selection: &CardSelection,
) -> Result<Option<usize>, String> {
    battle.v030_card_effect(CardKind::Infiltration, selection)
}
