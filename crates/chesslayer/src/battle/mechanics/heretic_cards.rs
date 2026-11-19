//! heretic cards effects. Selection and payment stay in the shared card pipeline.
use super::*;
impl Battle {
    pub(in crate::battle) fn execute_heretic_cards(
        &mut self,
        kind: CardKind,
        ids: &[PieceId],
        one: &impl Fn() -> Result<PieceId, String>,
        err: fn(Error) -> String,
        removed: &mut Option<usize>,
    ) -> Result<(), String> {
        use CardKind::*;
        match kind {
            Stimpack => {
                let id = one()?;
                self.grant_agility(id, 4);
                self.mechanics.statuses.entry(id).or_default().weakness += 1;
            }
            ExcitedSteps => {
                let id = one()?;
                self.grant_agility(id, 1);
            }
            DiningEtiquette => {
                let id = one()?;
                self.mechanics.statuses.entry(id).or_default().dining = true;
            }
            BloodyInvitation | MealTime => {
                let id = one()?;
                let hungry = self.status(id).hunger >= 2;
                self.worsen_hunger(id);
                if kind == MealTime && hungry {
                    self.mechanics.pending_draw += 1;
                }
            }
            LateDinner => {
                let id = one()?;
                let s = self.mechanics.statuses.entry(id).or_default();
                if s.hunger == 0 {
                    return Err("허기 상태의 아군을 선택하세요.".into());
                }
                s.hunger -= 1;
            }
            TornMask => {
                let id = one()?;
                self.mechanics.statuses.entry(id).or_default().blessing = false;
            }
            CrackedConscience => {
                let id = one()?;
                let s = self.mechanics.statuses.entry(id).or_default();
                if s.madness > 0 {
                    s.madness += 1;
                } else if s.guilt > 0 {
                    s.guilt += 1;
                } else {
                    return Err("광기 또는 죄책감이 있는 적을 선택하세요.".into());
                }
                self.resolve_conscience(id).map_err(err)?;
            }
            Sacrifice => {
                let id = one()?;
                let (pos, _) = self.piece_position(id).ok_or("대상 없음")?;
                self.kill_at(pos, true).map_err(err)?;
                self.mechanics.pending_mana += 3;
                *removed = Some(1);
            }
            BackstageSacrifice => {
                let id = one()?;
                if !self.mechanics.used.insert("backstage".into()) {
                    return Err("무대 뒤의 희생은 전투당 한 번만 사용할 수 있습니다.".into());
                }
                let (pos, _) = self.piece_position(id).ok_or("대상 없음")?;
                self.make_corpse(pos).map_err(err)?;
                self.mechanics.pending_draw += 2;
                *removed = Some(1);
            }
            InfantrySupport => {
                let id = one()?;
                let at = Position::new(((id - 10000) % 8) as i32, ((id - 10000) / 8) as i32);
                let next_id = self
                    .players
                    .iter()
                    .map(|p| p.id)
                    .chain(self.board.cells().iter().flatten().map(|p| p.id))
                    .max()
                    .unwrap_or(100)
                    + 1;
                let p = Piece {
                    id: next_id,
                    kind: pieces::AXE_INFANTRY,
                    owner: PLAYER,
                };
                self.board
                    .set_piece(at, Some(p))
                    .map_err(|e| format!("{e:?}"))?;
                self.mechanics.temporary_pieces.insert(next_id);
                if self.mechanics.relics.contains(&Relic::CarnivalFlare)
                    || self
                        .living(ENEMY)
                        .iter()
                        .any(|(_, p)| key(p.kind) == "eternal_guest")
                {
                    self.mechanics
                        .statuses
                        .entry(next_id)
                        .or_default()
                        .ingredient = true;
                }
            }
            BodySlam => {
                let allies: Vec<_> = ids
                    .iter()
                    .filter_map(|id| self.piece_position(*id).filter(|(_, p)| p.owner == PLAYER))
                    .collect();
                let enemies: Vec<_> = ids
                    .iter()
                    .filter_map(|id| self.piece_position(*id).filter(|(_, p)| p.owner == ENEMY))
                    .collect();
                if allies.is_empty() || allies.len() != enemies.len() {
                    return Err(
                        "희생할 아군과 제거할 적을 같은 수만큼 선택하세요. 로얄은 제외합니다."
                            .into(),
                    );
                }
                for (pos, _) in allies.iter().chain(enemies.iter()) {
                    self.kill_at(*pos, true).map_err(err)?;
                }
                self.mechanics.killed_this_turn = true;
                *removed = Some(enemies.len());
            }
            LastStand => {
                let alive = self.living(PLAYER);
                if alive.len() * 2 > self.mechanics.initial_allies {
                    return Err("아군이 전투 시작 수의 절반 이하일 때 사용할 수 있습니다.".into());
                }
                for (pos, p) in alive {
                    self.grant_agility(p.id, 3);
                    if let Some(target) = pieces::definition(p.kind)
                        .and_then(|d| d.upgrades.first())
                        .map(|u| u.target)
                    {
                        self.board
                            .set_piece(pos, Some(Piece { kind: target, ..p }))
                            .map_err(|e| format!("{e:?}"))?;
                        if let Some(owned) = self.players.iter_mut().find(|q| q.id == p.id) {
                            owned.kind = target;
                            owned.upgrades.push(key(target).into());
                        }
                    }
                }
                self.mechanics.last_stand = true;
            }
            Masquerade | IntermissionWaltz => {
                if ids.len() != 2 {
                    return Err("아군 비로얄 기물 두 개를 선택하세요.".into());
                }
                let (a, pa) = self.piece_position(ids[0]).ok_or("대상 없음")?;
                let (b, pb) = self.piece_position(ids[1]).ok_or("대상 없음")?;
                if kind == IntermissionWaltz && distance(a, b) != 1 {
                    return Err("서로 인접한 아군 두 개를 선택하세요.".into());
                }
                self.board
                    .set_piece(a, Some(pb))
                    .map_err(|e| format!("{e:?}"))?;
                self.board
                    .set_piece(b, Some(pa))
                    .map_err(|e| format!("{e:?}"))?;
            }
            ClearTablecloth | Frontline | Infiltration | Extraction => {
                if ids.len() != 2 {
                    return Err("기물 하나와 빈 칸 하나를 선택하세요.".into());
                }
                let piece_id = *ids
                    .iter()
                    .find(|id| **id < 10000)
                    .ok_or("이동할 기물을 선택하세요.")?;
                let cell = *ids
                    .iter()
                    .find(|id| **id >= 10000)
                    .ok_or("빈 칸을 선택하세요.")?
                    - 10000;
                let to = Position::new((cell % 8) as i32, (cell / 8) as i32);
                let (from, p) = self.piece_position(piece_id).ok_or("대상 없음")?;
                if kind == ClearTablecloth {
                    if (from.file - to.file).abs() + (from.rank - to.rank).abs() != 1 {
                        return Err("시체와 상하좌우로 인접한 빈 칸을 선택하세요.".into());
                    }
                    if self.mechanics.turn_used.contains("tablecloth") {
                        return Err("식탁보 걷기는 턴당 한 번입니다.".into());
                    }
                    self.mechanics.turn_used.insert("tablecloth".into());
                }
                if kind == Infiltration && distance(from, to) > 2 {
                    return Err("2칸 이내의 빈 칸을 선택하세요.".into());
                }
                self.board
                    .set_piece(from, None)
                    .map_err(|e| format!("{e:?}"))?;
                self.board
                    .set_piece(to, Some(p))
                    .map_err(|e| format!("{e:?}"))?;
                if kind == Frontline {
                    self.queue_formation();
                }
            }
            DeliveredSupper => {
                if ids.len() != 2 {
                    return Err("아군과 2칸 이내의 시체를 하나씩 선택하세요.".into());
                }
                let ally = ids
                    .iter()
                    .filter_map(|id| self.piece_position(*id))
                    .find(|(_, p)| p.owner == PLAYER)
                    .ok_or("아군을 선택하세요.")?;
                let corpse = ids
                    .iter()
                    .filter_map(|id| self.piece_position(*id))
                    .find(|(_, p)| p.kind == pieces::CORPSE)
                    .ok_or("시체를 선택하세요.")?;
                if distance(ally.0, corpse.0) > 2 {
                    return Err("시체가 아군으로부터 2칸을 초과했습니다.".into());
                }
                self.kill_at(corpse.0, false).map_err(err)?;
                self.eat_corpse(ally.1.id).map_err(err)?;
            }
            LastSupper => {
                let corpses: Vec<_> = ids
                    .iter()
                    .filter_map(|id| self.piece_position(*id))
                    .filter(|(_, p)| p.kind == pieces::CORPSE)
                    .collect();
                let allies: Vec<_> = ids
                    .iter()
                    .filter_map(|id| self.piece_position(*id))
                    .filter(|(_, p)| p.owner == PLAYER)
                    .collect();
                if corpses.is_empty() || corpses.len() > 3 {
                    return Err("소비할 시체 1~3개와 민첩을 받을 아군을 선택하세요.".into());
                }
                let recipients = if allies.is_empty() {
                    self.living(PLAYER)
                } else {
                    allies
                };
                if recipients.is_empty() {
                    return Err("민첩을 받을 아군이 없습니다.".into());
                }
                for (i, (pos, _)) in corpses.iter().enumerate() {
                    self.kill_at(*pos, false).map_err(err)?;
                    self.grant_agility(recipients[i % recipients.len()].1.id, 1);
                }
            }
            EndlessEcstasy => {
                if !self.mechanics.used.insert("endless".into()) {
                    return Err("환희는 끝나지 않는다: 전투당 한 번입니다.".into());
                }
                self.mechanics.endless = true;
            }
            _ => unreachable!("card dispatched to wrong effect family"),
        }
        Ok(())
    }
}
