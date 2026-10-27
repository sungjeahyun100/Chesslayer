use super::{CardDefinition, CardKind};
use crate::{game::engine_error, Battle, PieceId};

pub fn definition() -> CardDefinition {
    CardDefinition::new(CardKind::LimiterRelease, "리미터 해제", 6,
        "선택한 양측 기물 하나에 즉시 효과를 적용합니다. 대상은 총 3번 움직일 수 있으며, 세 번째 이동 직후 사망하고 턴이 넘어갑니다. 로얄도 사망합니다.", effect)
        .with_targets(targets)
}
fn targets(battle: &Battle) -> Vec<PieceId> {
    battle
        .board
        .cells()
        .iter()
        .flatten()
        .filter(|p| battle.limiter_status(p.id).is_none())
        .map(|p| p.id)
        .collect()
}
fn effect(battle: &mut Battle, target: Option<PieceId>) -> Result<Option<usize>, String> {
    battle
        .grant_limiter(target.ok_or("리미터를 해제할 기물을 선택하세요.")?)
        .map_err(engine_error)?;
    Ok(None)
}
