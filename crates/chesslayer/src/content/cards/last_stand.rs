use super::{CardDefinition, CardKind, CardSelection};
use crate::{Battle, PieceId};

pub fn definition() -> CardDefinition {
    let mut definition = CardDefinition::new(
        CardKind::LastStand, "마지막 저항", 3, "아군이 시작 수의 절반 이하일 때만 사용. 아군 전체 민첩 3, 영구 강화 한 단계. 이번 턴 적을 죽이지 못하고 턴을 넘기면 패배합니다.", effect,
    );
    definition.exhaust = false;
    definition
}

fn effect(battle: &mut Battle, target: Option<PieceId>) -> Result<Option<usize>, String> {
    battle.v030_card_effect(CardKind::LastStand, &CardSelection::single(target))
}
