use super::{CardDefinition, CardKind, CardSelection};
use crate::{Battle, PieceId};

pub fn definition() -> CardDefinition {
    let mut definition = CardDefinition::new(
        CardKind::GeneralMobilization,
        "총동원령",
        1,
        "다음 자신의 턴 전까지 아군 기물이 적에게 포획될 때마다 마나 1을 회복합니다. 최대 3회.",
        effect,
    );
    definition.exhaust = false;
    definition
}

fn effect(battle: &mut Battle, target: Option<PieceId>) -> Result<Option<usize>, String> {
    battle.v030_card_effect(
        CardKind::GeneralMobilization,
        &CardSelection::single(target),
    )
}
