use super::{CardDefinition, CardKind, CardSelection, SelectionMode};
use crate::{Battle, PieceId};

pub fn definition() -> CardDefinition {
    let mut definition = CardDefinition::new(
        CardKind::Frontline, "전선잡기", 0, "아군 기물과 빈 칸을 선택해 재배치합니다. 이후 원하는 만큼 다른 기물도 순서대로 재배치하고 완료합니다. 창고도 재배치할 수 있으며 탄약 소모는 없습니다.", effect,
    ).with_selection(SelectionMode::Multiple, targets, selected_effect);
    definition.exhaust = false;
    definition
}

fn effect(battle: &mut Battle, target: Option<PieceId>) -> Result<Option<usize>, String> {
    battle.v030_card_effect(CardKind::Frontline, &CardSelection::single(target))
}

fn targets(battle: &Battle) -> Vec<PieceId> {
    battle.v030_card_targets(CardKind::Frontline)
}

fn selected_effect(
    battle: &mut Battle,
    selection: &CardSelection,
) -> Result<Option<usize>, String> {
    battle.v030_card_effect(CardKind::Frontline, selection)
}
