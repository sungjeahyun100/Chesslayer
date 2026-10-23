use super::{CardDefinition, CardKind, CardSelection, SelectionMode};
use crate::{content::pieces, game::engine_error, Battle, PieceId, PLAYER};

pub fn definition() -> CardDefinition {
    CardDefinition::new(CardKind::TemporaryUpgrade, "일시 강화", 0,
        "아군 기물 N기를 각각 한 단계 강화합니다. 비용은 선택한 기물 수 N이며 현재 전투에서만 유지됩니다.",
        |_, _| Err("강화 선택 명령이 필요합니다.".into()))
        .with_selection(SelectionMode::Upgrade, targets, effect)
}
fn targets(battle: &Battle) -> Vec<PieceId> {
    battle
        .board
        .cells()
        .iter()
        .flatten()
        .filter(|p| {
            p.owner == PLAYER && pieces::definition(p.kind).is_some_and(|d| !d.upgrades.is_empty())
        })
        .map(|p| p.id)
        .collect()
}
fn effect(battle: &mut Battle, selection: &CardSelection) -> Result<Option<usize>, String> {
    if selection.target.is_some() || !selection.targets.is_empty() || selection.upgrades.is_empty()
    {
        return Err("강화할 기물과 경로를 한 개 이상 선택하세요.".into());
    }
    battle
        .temporary_upgrades(&selection.upgrades)
        .map_err(engine_error)?;
    Ok(None)
}
