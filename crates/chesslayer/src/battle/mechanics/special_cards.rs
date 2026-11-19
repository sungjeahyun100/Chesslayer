//! special cards effects. Selection and payment stay in the shared card pipeline.
use super::*;
impl Battle {
    pub(in crate::battle) fn execute_special_cards(
        &mut self,
        kind: CardKind,
        ids: &[PieceId],
        one: &impl Fn() -> Result<PieceId, String>,
        err: fn(Error) -> String,
        removed: &mut Option<usize>,
    ) -> Result<(), String> {
        use CardKind::*;
        match kind {
            SmokeScreen => {
                let id = one()?;
                self.grant_ward(id).map_err(err)?;
                self.grant_agility(id, 1);
            }
            AimedShot => {
                if ids.len() != 2 {
                    return Err("사격할 아군과 사선 안의 적을 선택하세요.".into());
                }
                let (from, p) = ids
                    .iter()
                    .filter_map(|id| self.piece_position(*id))
                    .find(|(_, p)| p.owner == PLAYER)
                    .ok_or("아군 사수를 선택하세요.")?;
                let (to, q) = ids
                    .iter()
                    .filter_map(|id| self.piece_position(*id))
                    .find(|(_, p)| p.owner == ENEMY)
                    .ok_or("적을 선택하세요.")?;
                if self.hit_chance(from, to).is_none() {
                    return Err("사거리와 사선을 확인하세요.".into());
                }
                let killed = self.kill_at_by(to, true, Some(p.id)).map_err(err)?;
                self.forced_turn_end = true;
                if killed {
                    self.record_kill(p, q);
                }
                *removed = Some(usize::from(killed));
            }
            _ => unreachable!("card dispatched to wrong effect family"),
        }
        Ok(())
    }
}
