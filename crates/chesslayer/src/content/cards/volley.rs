use super::{CardDefinition, CardKind, CardSelection};
use crate::{Battle, PieceId};

pub fn definition() -> CardDefinition {
    let mut definition = CardDefinition::new(
        CardKind::Volley, "전원 일제사격", 0, "사격 가능한 아군 보병 계열이 각자 사거리 안의 모든 적에게 명중 판정 사격을 합니다. 총 사격 횟수만큼 탄약을 소비합니다.", effect,
    );
    definition.exhaust = false;
    definition
}

fn effect(battle: &mut Battle, target: Option<PieceId>) -> Result<Option<usize>, String> {
    battle.v030_card_effect(CardKind::Volley, &CardSelection::single(target))
}
