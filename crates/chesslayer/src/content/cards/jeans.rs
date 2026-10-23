use super::{CardDefinition, CardKind};
use crate::{game::engine_error, Battle, PieceId, QUEEN, ROOK};

pub fn definition() -> CardDefinition {
    CardDefinition {
        exhaust: true,
        innate: true,
        ..CardDefinition::new(
            CardKind::Jeans,
            "청바지",
            0,
            "양 진영의 모든 룩과 퀸을 추방합니다. 양측 로얄은 보호됩니다.",
            effect,
        )
    }
}

fn effect(battle: &mut Battle, target: Option<PieceId>) -> Result<Option<usize>, String> {
    if target.is_some() {
        return Err("청바지는 대상을 선택하지 않습니다.".into());
    }
    let ids: Vec<_> = battle
        .board
        .cells()
        .iter()
        .flatten()
        .filter(|p| matches!(p.kind, ROOK | QUEEN))
        .map(|p| p.id)
        .collect();
    Ok(Some(battle.remove_nonroyals(&ids).map_err(engine_error)?))
}
