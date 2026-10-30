use super::{CardDefinition, CardKind, CardSelection, SelectionMode};
use crate::{Battle, PieceId};

pub fn definition() -> CardDefinition {
    let mut definition = CardDefinition::new(
        CardKind::RecklessCharge, "무모한 돌격", 2, "아군 하나가 이번 턴 기존 1칸 비포획 행마 방향으로 빈칸 1칸을 더 이동할 수 있습니다. 다음 적 턴에 포획되면 마나 1을 잃습니다.", effect,
    ).with_selection(SelectionMode::Single, targets, selected_effect);
    definition.exhaust = false;
    definition
}

fn effect(battle: &mut Battle, target: Option<PieceId>) -> Result<Option<usize>, String> {
    battle.v030_card_effect(CardKind::RecklessCharge, &CardSelection::single(target))
}

fn targets(battle: &Battle) -> Vec<PieceId> {
    battle.v030_card_targets(CardKind::RecklessCharge)
}
fn selected_effect(
    battle: &mut Battle,
    selection: &CardSelection,
) -> Result<Option<usize>, String> {
    battle.v030_card_effect(CardKind::RecklessCharge, selection)
}
