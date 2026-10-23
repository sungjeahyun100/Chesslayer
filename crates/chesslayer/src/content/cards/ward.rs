use super::{CardDefinition, CardKind};
use crate::{game::engine_error, Battle, PieceId};

pub fn definition() -> CardDefinition {
    CardDefinition::new(
        CardKind::Ward,
        "가호",
        1,
        "기물 하나가 다음 일반 포획을 한 번 막습니다. 공격자는 제자리에 남습니다.",
        effect,
    )
    .with_targets(targets)
}

/// Both sides, including royals, may receive a ward.
pub fn targets(battle: &Battle) -> Vec<PieceId> {
    battle
        .board
        .cells()
        .iter()
        .flatten()
        .map(|p| p.id)
        .collect()
}

/// Shared by Ward and FleetingWard; persistence and capture rules stay in Battle.
pub fn effect(battle: &mut Battle, target: Option<PieceId>) -> Result<Option<usize>, String> {
    battle
        .grant_ward(target.ok_or("가호를 받을 기물을 선택하세요.")?)
        .map_err(engine_error)?;
    Ok(None)
}
