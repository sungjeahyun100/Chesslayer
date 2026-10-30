use super::{CardDefinition, CardKind, CardSelection, SelectionMode};
use crate::{Battle, PieceId};

pub fn definition() -> CardDefinition {
    let mut definition = CardDefinition::new(
        CardKind::ExcitedSteps,
        "들뜬 발걸음",
        1,
        "아군 하나에 이번 턴 민첩 1을 부여합니다.",
        effect,
    )
    .with_selection(SelectionMode::Single, targets, selected_effect);
    definition.exhaust = false;
    definition
}

fn effect(battle: &mut Battle, target: Option<PieceId>) -> Result<Option<usize>, String> {
    battle.v030_card_effect(CardKind::ExcitedSteps, &CardSelection::single(target))
}

fn targets(battle: &Battle) -> Vec<PieceId> {
    battle.v030_card_targets(CardKind::ExcitedSteps)
}

fn selected_effect(
    battle: &mut Battle,
    selection: &CardSelection,
) -> Result<Option<usize>, String> {
    battle.v030_card_effect(CardKind::ExcitedSteps, selection)
}
