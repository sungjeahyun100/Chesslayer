//! artillery cards effects. Selection and payment stay in the shared card pipeline.
use super::*;
impl Battle {
    pub(in crate::battle) fn execute_artillery_cards(
        &mut self,
        kind: CardKind,
        _ids: &[PieceId],
        _one: &impl Fn() -> Result<PieceId, String>,
        err: fn(Error) -> String,
        removed: &mut Option<usize>,
    ) -> Result<(), String> {
        use CardKind::*;
        match kind {
            Volley => {
                let shots = self.volley_shots();
                if shots.is_empty() {
                    return Err("사격 범위 안에 적이 있는 아군 보병이 없습니다.".into());
                }
                let count = shots.len();
                for (from, shooter, to, target, planned_chance) in shots {
                    let target_alive = self
                        .board
                        .piece_at(to)
                        .map_err(|e| format!("{e:?}"))?
                        .is_some_and(|p| p.id == target.id);
                    let chance = if target_alive {
                        self.hit_chance(from, to).unwrap_or(planned_chance)
                    } else {
                        planned_chance
                    };
                    *self.mechanics.shot_counts.entry(shooter.id).or_default() += 1;
                    self.mechanics.aimed.remove(&shooter.id);
                    if self.mechanics.fire_data_sharing
                        && self.mechanics.shared_target == Some(target.id)
                    {
                        self.mechanics.shared_target = None;
                    }
                    let hit = self.roll_shot(chance);
                    self.events.push(format!(
                        "{} 일제사격 {} (명중률 {}%).",
                        piece_name(shooter.kind),
                        if hit { "명중" } else { "빗나감" },
                        chance
                    ));
                    if !hit || !target_alive {
                        continue;
                    }
                    if self.mechanics.fire_data_sharing {
                        self.mechanics.shared_target = Some(target.id);
                    }
                    self.mechanics.shot_hits_this_turn =
                        self.mechanics.shot_hits_this_turn.saturating_add(1);
                    if self.wards.remove(&target.id) {
                        self.combat_events.push(CombatEvent::StatusTriggered {
                            piece_id: target.id,
                            at: to,
                            status: "ward",
                        });
                        self.events.push(format!(
                            "가호: {}에게 명중한 사격을 한 번 막았습니다.",
                            piece_name(target.kind)
                        ));
                    } else if self.kill_at_by(to, true, Some(shooter.id)).map_err(err)? {
                        self.record_kill(shooter, target);
                    }
                }
                *removed = Some(count);
            }
            _ => unreachable!("card dispatched to wrong effect family"),
        }
        Ok(())
    }
}
