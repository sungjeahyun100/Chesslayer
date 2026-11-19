//! relics rules. The call sites retain their original ordering.
use super::*;

impl Battle {
    pub fn configure_v030(&mut self, relics: &[Relic]) {
        self.mechanics.relics = relics.to_vec();
        if relics.contains(&Relic::RecruitmentOffice) {
            if let Some(cell) = self.friendly_empty_cells(true).first().copied() {
                let cell = cell - 10000;
                let _ =
                    self.spawn_reserve_pawn(Position::new((cell % 8) as i32, (cell / 8) as i32));
            }
        }
        let boss = self
            .board
            .cells()
            .iter()
            .flatten()
            .any(|p| key(p.kind) == "eternal_guest");
        let enemy_carnival = boss
            || self.board.cells().iter().flatten().any(|p| {
                p.owner == ENEMY
                    && matches!(key(p.kind), "smiling_king" | "toast_lord" | "ecstatic_lord")
            });
        let carnival = relics.contains(&Relic::CarnivalFlare);
        let enemy_vanguard = self.board.cells().iter().flatten().any(|p| {
            p.owner == ENEMY && matches!(key(p.kind), "vanguard_commander" | "mounted_commander")
        });
        self.mechanics.enemy_actions = if enemy_vanguard { 2 } else { 1 };
        for p in self.board.cells().iter().flatten().copied() {
            if p.kind == pieces::CORPSE {
                continue;
            }
            let st = self.mechanics.statuses.entry(p.id).or_default();
            if carnival || enemy_carnival {
                st.ingredient = true;
            }
            if (carnival && p.owner == PLAYER) || (enemy_carnival && p.owner == ENEMY) {
                st.blessing = true;
            }
        }
        let pieces: Vec<_> = self.board.cells().iter().flatten().copied().collect();
        for piece in pieces {
            self.apply_kind_passive(piece);
        }
        if relics.contains(&Relic::TornMenu) {
            if let Some(id) = self
                .living(ENEMY)
                .into_iter()
                .find(|(_, p)| !self.is_royal_piece(p.id))
                .map(|(_, p)| p.id)
            {
                self.worsen_hunger(id);
            }
        }
    }
    pub fn sandbox_set_relic(&mut self, relic: Relic, enabled: bool) -> Result<(), String> {
        if !self.sandbox {
            return Err("테스트장에서만 유물을 설정할 수 있습니다.".into());
        }
        let owned = self.mechanics.relics.contains(&relic);
        if enabled && !owned {
            self.mechanics.relics.push(relic);
            if relic == Relic::RecruitmentOffice {
                if let Some(cell) = self.friendly_empty_cells(true).first().copied() {
                    let cell = cell - 10000;
                    self.spawn_reserve_pawn(Position::new((cell % 8) as i32, (cell / 8) as i32))?;
                }
            }
        } else if !enabled {
            self.mechanics.relics.retain(|r| *r != relic);
        }
        Ok(())
    }
    pub fn sandbox_relics(&self) -> &[Relic] {
        &self.mechanics.relics
    }
    pub fn relic_status(&self, relic: Relic) -> Option<String> {
        match relic {
            Relic::RecruitmentOffice => Some(format!(
                "임시 폰 {}기 생존",
                self.mechanics
                    .temporary_pieces
                    .iter()
                    .filter(|id| self.piece_position(**id).is_some())
                    .count()
            )),
            Relic::WarBonds => Some(format!(
                "정규 아군 포획 누적 {}회",
                self.mechanics.regular_losses
            )),
            Relic::PopulationMobilization => Some(
                if self.mechanics.revived_once {
                    "이번 전투 발동 완료"
                } else {
                    "첫 부활 대기 중"
                }
                .into(),
            ),
            Relic::BallisticComputer => Some(format!(
                "사격한 기물 {}기",
                self.mechanics.shot_counts.len()
            )),
            Relic::AmmunitionLine => Some(format!(
                "이번 턴 명중 {}회",
                self.mechanics.shot_hits_this_turn
            )),
            Relic::GunMount => Some(format!(
                "이번 턴 이동한 아군 {}기",
                self.mechanics.turn_moved.len()
            )),
            _ => None,
        }
    }
}
