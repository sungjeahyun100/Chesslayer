//! vanguard cards effects. Selection and payment stay in the shared card pipeline.
use super::*;
impl Battle {
    pub(in crate::battle) fn execute_vanguard_cards(
        &mut self,
        kind: CardKind,
        ids: &[PieceId],
        one: &impl Fn() -> Result<PieceId, String>,
        err: fn(Error) -> String,
        removed: &mut Option<usize>,
    ) -> Result<(), String> {
        use CardKind::*;
        match kind {
            EmergencyConscription => {
                if ids.len() != 2 {
                    return Err("사망한 폰과 아군 진영 빈칸을 선택하세요.".into());
                }
                let dead_id = *ids
                    .iter()
                    .find(|id| **id < 10000)
                    .ok_or("사망한 폰을 선택하세요.")?;
                let cell = *ids
                    .iter()
                    .find(|id| **id >= 10000)
                    .ok_or("빈칸을 선택하세요.")?
                    - 10000;
                let at = Position::new((cell % 8) as i32, (cell / 8) as i32);
                let pawn = self
                    .players
                    .iter()
                    .find(|p| p.id == dead_id && p.position.is_none() && pieces::is_pawn(p.kind))
                    .ok_or("이번 전투에서 사망한 폰을 선택하세요.")?
                    .kind;
                self.board
                    .set_piece(
                        at,
                        Some(Piece {
                            id: dead_id,
                            kind: pawn,
                            owner: PLAYER,
                        }),
                    )
                    .map_err(|e| format!("{e:?}"))?;
                self.sync_players();
                if !self.mechanics.revived_once {
                    self.mechanics.revived_once = true;
                    if self
                        .mechanics
                        .relics
                        .contains(&Relic::PopulationMobilization)
                    {
                        let cells = self.friendly_empty_cells(false);
                        self.enqueue(
                            "인구 동원령: 예비 폰을 배치할 칸 선택",
                            cells,
                            ChoiceEffect::ReservePawn,
                            false,
                        );
                    }
                }
            }
            GeneralMobilization => {
                if self.mechanics.mobilization_left > 0 {
                    return Err("총동원령이 이미 유지되고 있습니다.".into());
                }
                self.mechanics.mobilization_left = 3;
            }
            ComradeRevenge => {
                if self.mechanics.revenge_ready {
                    return Err("전우의 복수가 이미 대기 중입니다.".into());
                }
                self.mechanics.revenge_ready = true;
            }
            ReliefForce => {
                if self.mechanics.relief_ready {
                    return Err("교대 병력이 이미 대기 중입니다.".into());
                }
                self.mechanics.relief_ready = true;
            }
            HumanWave => {
                if self.mechanics.human_wave_left > 0 {
                    return Err("인해전술이 이미 유지되고 있습니다.".into());
                }
                self.mechanics.human_wave_left = 2;
                self.mechanics.wave_movers.clear();
            }
            LastDefense => {
                if self.mechanics.last_defense_ready {
                    return Err("최후의 방어선이 이미 대기 중입니다.".into());
                }
                self.mechanics.last_defense_ready = true;
            }
            FireDataSharing => {
                if self.mechanics.fire_data_sharing {
                    return Err("사격 제원 공유가 이미 유지되고 있습니다.".into());
                }
                self.mechanics.fire_data_sharing = true;
                self.mechanics.shared_target = None;
            }
            AssaultOrder => {
                if ids.is_empty() || ids.len() > 3 {
                    return Err("이동할 아군 1~3기를 선택하세요.".into());
                }
                for &id in ids {
                    let (from, p) = self.piece_position(id).ok_or("기물이 없습니다.")?;
                    let to = Position::new(from.file, from.rank - 1);
                    if !self.board.contains(to)
                        || self
                            .board
                            .piece_at(to)
                            .map_err(|e| format!("{e:?}"))?
                            .is_some()
                    {
                        return Err("전방 빈칸으로 이동할 수 없습니다.".into());
                    }
                    self.board
                        .set_piece(from, None)
                        .map_err(|e| format!("{e:?}"))?;
                    self.board
                        .set_piece(to, Some(p))
                        .map_err(|e| format!("{e:?}"))?;
                    self.mechanics.turn_moved.insert(id);
                    self.moved_ids.insert(id);
                }
            }
            Expendables => {
                let id = one()?;
                let (at, _) = self.piece_position(id).ok_or("기물이 없습니다.")?;
                self.kill_at(at, false).map_err(err)?;
                self.mechanics.pending_draw += 3;
                *removed = Some(1);
            }
            RecklessCharge => {
                if !self.mechanics.reckless.insert(one()?) {
                    return Err("이미 무모한 돌격이 적용된 기물입니다.".into());
                }
            }
            AimCorrection => {
                let id = one()?;
                let value = self.mechanics.aimed.entry(id).or_default();
                *value = value.saturating_add(25).min(100);
            }
            ExtraPropellant => {
                let id = one()?;
                let value = self.mechanics.propellant.entry(id).or_default();
                *value = value.saturating_add(20).min(100);
            }
            EntrenchedDefense => {
                let id = one()?;
                if self.mechanics.turn_moved.contains(&id) {
                    return Err("이번 턴 이동하지 않은 사수를 선택하세요.".into());
                }
                if !self.mechanics.entrenched_pending.insert(id) {
                    return Err("이미 진지를 유지 중인 사수입니다.".into());
                }
            }
            SuppressionFire => {
                if !self.mechanics.suppressed.insert(one()?) {
                    return Err("이미 이동이 제압된 적입니다.".into());
                }
            }
            RapidReload => {
                if ids.len() != 2 {
                    return Err("사수와 적을 한 기씩 선택하세요.".into());
                }
                let shooter = *ids
                    .iter()
                    .find(|id| {
                        self.piece_position(**id)
                            .is_some_and(|(_, p)| p.owner == PLAYER)
                    })
                    .ok_or("사수를 선택하세요.")?;
                let victim = *ids
                    .iter()
                    .find(|id| {
                        self.piece_position(**id)
                            .is_some_and(|(_, p)| p.owner == ENEMY)
                    })
                    .ok_or("적을 선택하세요.")?;
                self.free_counterattack(shooter, victim, true)?;
            }
            FocusFire => {
                if ids.len() != 3 {
                    return Err("사수 두 기와 적 하나를 선택하세요.".into());
                }
                let shooters: Vec<_> = ids
                    .iter()
                    .copied()
                    .filter(|id| {
                        self.piece_position(*id)
                            .is_some_and(|(_, p)| p.owner == PLAYER)
                    })
                    .collect();
                let victim = *ids
                    .iter()
                    .find(|id| {
                        self.piece_position(**id)
                            .is_some_and(|(_, p)| p.owner == ENEMY)
                    })
                    .ok_or("적을 선택하세요.")?;
                if shooters.len() != 2
                    || shooters
                        .iter()
                        .any(|id| !self.counterattack_candidates(victim, true).contains(id))
                {
                    return Err("같은 적을 사격할 수 있는 사수 두 기를 선택하세요.".into());
                }
                self.free_counterattack(shooters[0], victim, true)?;
                if self
                    .piece_position(victim)
                    .is_some_and(|(_, p)| p.owner == ENEMY)
                    && self.piece_position(shooters[1]).is_some()
                {
                    self.free_counterattack(shooters[1], victim, true)?;
                }
            }
            _ => unreachable!("card dispatched to wrong effect family"),
        }
        Ok(())
    }
}
