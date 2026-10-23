use super::{CardDefinition, CardKind};
use crate::{
    content::pieces::{self, PieceFamily},
    game::engine_error,
    Battle, PieceId,
};

pub fn definition() -> CardDefinition {
    CardDefinition::new(
        CardKind::ReverseJeans,
        "거꾸로 입은 청바지",
        3,
        "양측 나이트·비숍 계열을 모두 제거합니다. 로얄은 보호됩니다.",
        effect,
    )
}
fn effect(battle: &mut Battle, target: Option<PieceId>) -> Result<Option<usize>, String> {
    if target.is_some() {
        return Err("이 카드는 대상을 선택하지 않습니다.".into());
    }
    let ids: Vec<_> = battle
        .board
        .cells()
        .iter()
        .flatten()
        .filter(|p| {
            pieces::definition(p.kind)
                .is_some_and(|d| matches!(d.family, PieceFamily::Knight | PieceFamily::Bishop))
        })
        .map(|p| p.id)
        .collect();
    Ok(Some(battle.remove_nonroyals(&ids).map_err(engine_error)?))
}
