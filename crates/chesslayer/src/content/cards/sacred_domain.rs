use super::{CardDefinition, CardKind};
use crate::{
    content::pieces::{self, PieceFamily},
    game::engine_error,
    Battle, PieceId, PLAYER,
};

pub fn definition() -> CardDefinition {
    CardDefinition::new(
        CardKind::SacredDomain,
        "신성 영역 전개",
        0,
        "아군 비숍 계열 하나를 희생하고 남은 모든 아군에게 전투 한정 가호를 부여합니다.",
        effect,
    )
    .with_targets(targets)
}
fn targets(battle: &Battle) -> Vec<PieceId> {
    battle
        .board
        .cells()
        .iter()
        .flatten()
        .filter(|p| {
            p.owner == PLAYER
                && pieces::definition(p.kind).is_some_and(|d| d.family == PieceFamily::Bishop)
        })
        .map(|p| p.id)
        .collect()
}
fn effect(battle: &mut Battle, target: Option<PieceId>) -> Result<Option<usize>, String> {
    let id = target.ok_or("희생할 아군 비숍을 선택하세요.")?;
    if !targets(battle).contains(&id) {
        return Err("살아 있는 아군 비숍 계열만 희생할 수 있습니다.".into());
    }
    battle.remove_nonroyals(&[id]).map_err(engine_error)?;
    let allies: Vec<_> = battle
        .board
        .cells()
        .iter()
        .flatten()
        .filter(|p| p.owner == PLAYER)
        .map(|p| p.id)
        .collect();
    for id in allies {
        battle.grant_ward(id).map_err(engine_error)?;
    }
    Ok(None)
}
