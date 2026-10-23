use super::{CardDefinition, CardKind, CardSelection, SelectionMode};
use crate::{game::engine_error, Battle, PieceId};

pub fn definition() -> CardDefinition {
    CardDefinition::new(CardKind::PawnStorm, "폰 스톰", 1,
        "전진 가능한 아군 폰 계열을 한 기 이상 골라 모두 한 칸 전진시킵니다. 포획과 이동 행동 소모는 없습니다.",
        |_, _| Err("복수 선택 명령이 필요합니다.".into()))
        .with_selection(SelectionMode::Multiple, targets, effect)
}
fn targets(battle: &Battle) -> Vec<PieceId> {
    battle.forward_pawn_candidates()
}
fn effect(battle: &mut Battle, selection: &CardSelection) -> Result<Option<usize>, String> {
    if selection.target.is_some() || !selection.upgrades.is_empty() || selection.targets.is_empty()
    {
        return Err("전진할 폰을 한 기 이상 선택하세요.".into());
    }
    battle
        .advance_pawns(&selection.targets)
        .map_err(engine_error)?;
    Ok(None)
}
