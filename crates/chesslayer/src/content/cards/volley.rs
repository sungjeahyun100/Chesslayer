use super::{CardDefinition, CardKind, CardSelection};
use crate::{Battle, PieceId};

pub fn definition() -> CardDefinition {
    let mut definition = CardDefinition::new(
        CardKind::Volley, "전원 일제사격", 0, "아군의 포획 범위 안에 있는 모든 적 비로얄을 제거합니다. 사망한 적 수만큼 탄약을 소비합니다.", effect,
    );
    definition.exhaust = false;
    definition
}

fn effect(battle: &mut Battle, target: Option<PieceId>) -> Result<Option<usize>, String> {
    battle.v030_card_effect(CardKind::Volley, &CardSelection::single(target))
}
