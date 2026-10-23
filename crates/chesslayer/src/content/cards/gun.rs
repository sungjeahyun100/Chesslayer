use super::{CardDefinition, CardKind};
use crate::{game::engine_error, Battle, PieceId, Position, ENEMY, PLAYER, QUEEN, ROOK};

pub const GUN_ADJACENT_FORMATION: bool = true;
pub const GUN_HALF_WIDTH: i32 = 1;
pub const GUN_HITS_ALLIES: bool = true;

pub fn definition() -> CardDefinition {
    CardDefinition {
        retain: true,
        ..CardDefinition::new(
            CardKind::Gun,
            "알레킨의 총",
            3,
            "같은 파일에 전방부터 룩·룩·퀸을 연속 배치하세요. 맨 앞 룩을 선택하면 그 앞의 3개 파일에서 양측 기물을 제거합니다. 로얄 제외.",
            effect,
        ).with_targets(targets)
    }
}

fn targets(battle: &Battle) -> Vec<PieceId> {
    battle
        .board
        .cells()
        .iter()
        .flatten()
        .filter(|p| front(battle, p.id).is_some())
        .map(|p| p.id)
        .collect()
}

fn effect(battle: &mut Battle, target: Option<PieceId>) -> Result<Option<usize>, String> {
    let target = target.ok_or("진형 맨 앞의 아군 룩을 선택하세요.")?;
    let front =
        front(battle, target).ok_or("전방부터 룩·룩·퀸을 같은 파일의 연속 세 칸에 배치하세요.")?;
    let ids: Vec<_> = battle
        .board
        .cells()
        .iter()
        .enumerate()
        .filter_map(|(i, p)| {
            p.filter(|p| {
                (GUN_HITS_ALLIES || p.owner == ENEMY)
                    && (i as i32 % 8 - front.file).abs() <= GUN_HALF_WIDTH
                    && (i as i32 / 8) < front.rank
            })
            .map(|p| p.id)
        })
        .collect();
    Ok(Some(battle.remove_nonroyals(&ids).map_err(engine_error)?))
}

fn front(battle: &Battle, id: PieceId) -> Option<Position> {
    let (i, _) = battle
        .board
        .cells()
        .iter()
        .enumerate()
        .find(|(_, p)| p.is_some_and(|p| p.id == id && p.owner == PLAYER && p.kind == ROOK))?;
    let front = Position::new((i % 8) as i32, (i / 8) as i32);
    let behind: Vec<_> = ((front.rank + 1)..8)
        .filter_map(|rank| {
            battle
                .board
                .piece_at(Position::new(front.file, rank))
                .ok()
                .flatten()
                .map(|p| (rank, p))
        })
        .collect();
    if behind.len() < 2 {
        return None;
    }
    if behind[0].1.owner != PLAYER
        || behind[0].1.kind != ROOK
        || behind[1].1.owner != PLAYER
        || behind[1].1.kind != QUEEN
    {
        return None;
    }
    if GUN_ADJACENT_FORMATION && (behind[0].0 != front.rank + 1 || behind[1].0 != front.rank + 2) {
        return None;
    }
    Some(front)
}
