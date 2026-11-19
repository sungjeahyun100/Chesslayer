//! cards rules. The call sites retain their original ordering.
use super::*;

impl Battle {
    pub fn v030_card_targets(&self, kind: CardKind) -> Vec<PieceId> {
        use CardKind::*;
        let mut out = vec![];
        for p in self.board.cells().iter().flatten() {
            let corpse = p.kind == pieces::CORPSE;
            let ally = p.owner == PLAYER && !corpse;
            let enemy = p.owner == ENEMY && !corpse;
            let nonroyal = !self.is_royal_piece(p.id);
            let yes = match kind {
                Stimpack | ExcitedSteps | DiningEtiquette | LateDinner | SmokeScreen => ally,
                BloodyInvitation | MealTime | TornMask => enemy,
                CrackedConscience => {
                    enemy && (self.status(p.id).madness > 0 || self.status(p.id).guilt > 0)
                }
                Sacrifice => ally && nonroyal && !self.mechanics.temporary_pieces.contains(&p.id),
                BackstageSacrifice => {
                    ally && nonroyal
                        && pieces::is_pawn(p.kind)
                        && !self.mechanics.temporary_pieces.contains(&p.id)
                }
                BodySlam => (ally || enemy) && nonroyal,
                Masquerade | IntermissionWaltz | Infiltration | Extraction => ally && nonroyal,
                Frontline => ally,
                DeliveredSupper | LastSupper => ally || corpse,
                ClearTablecloth => corpse,
                AimedShot => {
                    (enemy && nonroyal)
                        || (ally && self.can_act(*p) && shot_profile(key(p.kind)).is_some())
                }
                EmergencyConscription => false,
                AssaultOrder => {
                    ally && self.board.cells().iter().enumerate().any(|(i, q)| {
                        q.is_none()
                            && i / 8 + 1
                                == self
                                    .piece_position(p.id)
                                    .map_or(0, |(at, _)| at.rank as usize)
                            && i % 8
                                == self
                                    .piece_position(p.id)
                                    .map_or(9, |(at, _)| at.file as usize)
                    })
                }
                Expendables => ally && nonroyal && !self.mechanics.temporary_pieces.contains(&p.id),
                RecklessCharge => {
                    ally && self.piece_position(p.id).is_some_and(|(from, _)| {
                        self.piece_actions(p.kind, from)
                            .ok()
                            .is_some_and(|actions| {
                                actions.iter().any(|a| {
                                    if !a.continuation.is_empty() || a.first.effect != Effect::Move
                                    {
                                        return false;
                                    }
                                    let one = a.first.move_to;
                                    let dx = one.file - from.file;
                                    let dy = one.rank - from.rank;
                                    let to = Position::new(one.file + dx, one.rank + dy);
                                    dx.abs().max(dy.abs()) == 1
                                        && self.board.contains(to)
                                        && self.board.piece_at(one).ok().flatten().is_none()
                                        && self.board.piece_at(to).ok().flatten().is_none()
                                })
                            })
                    })
                }
                AimCorrection | RapidReload => ally && shot_profile(key(p.kind)).is_some(),
                EntrenchedDefense => {
                    ally && shot_profile(key(p.kind)).is_some_and(|(_, _, range)| range < 7)
                }
                ExtraPropellant => {
                    ally && matches!(p.kind, pieces::AP_TANK | pieces::FRAGMENT_TANK)
                        && self.piece_position(p.id).is_some_and(|(from, _)| {
                            self.living(ENEMY).into_iter().any(|(to, _)| {
                                self.hit_chance(from, to).is_some()
                                    && self
                                        .shot_effect_preview(p.kind, from, to)
                                        .1
                                        .iter()
                                        .any(|path| !path.is_empty())
                            })
                        })
                }
                SuppressionFire => enemy,
                FocusFire => (ally && shot_profile(key(p.kind)).is_some()) || enemy,
                _ => false,
            };
            if yes {
                out.push(p.id);
            }
        }
        if kind == EmergencyConscription {
            let dead: Vec<_> = self
                .players
                .iter()
                .filter(|p| {
                    p.position.is_none()
                        && !self.dead_at_start.contains(&p.id)
                        && pieces::is_pawn(p.kind)
                })
                .map(|p| p.id)
                .collect();
            if !self.friendly_empty_cells(false).is_empty() {
                out.extend(dead);
            }
        }
        if kind == RapidReload {
            out.extend(self.living(ENEMY).into_iter().map(|(_, p)| p.id));
        }
        if matches!(
            kind,
            InfantrySupport
                | ClearTablecloth
                | Frontline
                | Infiltration
                | Extraction
                | EmergencyConscription
        ) {
            for (i, p) in self.board.cells().iter().enumerate() {
                if p.is_none()
                    && (!matches!(kind, InfantrySupport | Extraction) || i / 8 >= 6)
                    && (kind != EmergencyConscription || (i / 8 >= 4 && !out.is_empty()))
                {
                    out.push(10000 + i as u32);
                }
            }
        }
        out
    }
    pub fn v030_card_effect(
        &mut self,
        kind: CardKind,
        selection: &CardSelection,
    ) -> Result<Option<usize>, String> {
        use CardKind::*;
        if self.pending_choice().is_some() {
            return Err("먼저 발동한 능력의 대상을 선택하세요.".into());
        }
        if self.phase != Phase::Active || self.turn != PLAYER {
            return Err("플레이어 전투 턴에만 사용할 수 있습니다.".into());
        }
        let ids = if selection.targets.is_empty() {
            selection.target.into_iter().collect::<Vec<_>>()
        } else {
            selection.targets.clone()
        };
        let candidates = self.v030_card_targets(kind);
        if ids.iter().any(|id| !candidates.contains(id)) {
            return Err("카드 효과에 맞는 대상을 선택하세요.".into());
        }
        if ids.iter().copied().collect::<HashSet<_>>().len() != ids.len() {
            return Err("같은 대상을 중복 선택할 수 없습니다.".into());
        }
        let one = || -> Result<PieceId, String> {
            if ids.len() != 1 {
                Err("대상을 하나 선택하세요.".into())
            } else {
                Ok(ids[0])
            }
        };
        let err = |e: Error| format!("전투 효과 오류: {e:?}");
        let mut removed = None;
        match kind {
            EmergencyConscription
            | GeneralMobilization
            | ComradeRevenge
            | ReliefForce
            | HumanWave
            | LastDefense
            | FireDataSharing
            | AssaultOrder
            | Expendables
            | RecklessCharge
            | AimCorrection
            | ExtraPropellant
            | EntrenchedDefense
            | SuppressionFire
            | RapidReload
            | FocusFire => self.execute_vanguard_cards(kind, &ids, &one, err, &mut removed)?,
            Stimpack | ExcitedSteps | DiningEtiquette | BloodyInvitation | MealTime
            | LateDinner | TornMask | CrackedConscience | Sacrifice | BackstageSacrifice
            | InfantrySupport | BodySlam | LastStand | Masquerade | IntermissionWaltz
            | ClearTablecloth | Frontline | Infiltration | Extraction | DeliveredSupper
            | LastSupper | EndlessEcstasy => {
                self.execute_heretic_cards(kind, &ids, &one, err, &mut removed)?
            }
            Volley => self.execute_artillery_cards(kind, &ids, &one, err, &mut removed)?,
            SmokeScreen | AimedShot => {
                self.execute_special_cards(kind, &ids, &one, err, &mut removed)?
            }
            _ => return Err("이 카드는 기존 카드 효과를 사용합니다.".into()),
        }
        self.sync_players();
        self.update_result(ResultReason::RoyalCaptured);
        Ok(removed)
    }
}
