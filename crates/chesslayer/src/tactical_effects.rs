//! Card resolution validates on the caller's cloned battle, then commits atomically.
use super::*;
use crate::content::cards::CardKind;
use crate::content::pieces::*;
impl Battle {
    pub fn tactical_card(
        &mut self,
        kind: CardKind,
        targets: &[PieceId],
        cells: &[usize],
    ) -> Result<u32, String> {
        use CardKind::*;
        let fail = || "카드 대상 또는 사용 조건이 올바르지 않습니다.".to_string();
        if targets.iter().collect::<HashSet<_>>().len() != targets.len() {
            return Err(fail());
        }
        let d = kind.definition();
        if d.selection_mode == crate::content::cards::SelectionMode::None && !targets.is_empty() {
            return Err(fail());
        }
        let candidates = (d.target_candidates)(self);
        if !targets.iter().all(|id| candidates.contains(id)) {
            return Err(fail());
        }
        let single = || {
            targets
                .first()
                .copied()
                .filter(|_| targets.len() == 1)
                .ok_or_else(fail)
        };
        let ally = |b: &Self, id| {
            b.located(id).is_some_and(|(_, p)| {
                p.owner == PLAYER && p.kind != CORPSE && !b.status(id).disabled
            })
        };
        let err = crate::game::engine_error;
        match kind {
            Stimpack => {
                let id = single()?;
                self.grant_agile(id, 4);
                self.status_mut(id).weak += 1;
            }
            ExcitedSteps | SpecialManeuver => {
                self.grant_agile(single()?, 1);
            }
            BloodyInvitation | MealTime => {
                let id = single()?;
                if kind == MealTime && self.status(id).hunger >= 2 {
                    self.mechanics.draws += 1;
                }
                self.worsen_hunger(id);
            }
            TableManners => {
                self.status_mut(single()?).etiquette = true;
            }
            LateDinner => {
                let id = single()?;
                let s = self.status_mut(id);
                s.hunger = s.hunger.saturating_sub(1);
            }
            TornMask => {
                self.status_mut(single()?).blessed = false;
            }
            CrackedConscience => {
                let id = single()?;
                let s = self.status_mut(id);
                if s.madness > 0 {
                    s.madness += 1;
                } else if s.guilt > 0 {
                    s.guilt += 1;
                } else {
                    return Err(fail());
                }
                self.resolve_mind(id).map_err(err)?;
            }
            Sacrifice => {
                self.kill_nonroyal(single()?, true).map_err(err)?;
                self.mechanics.mana += 3;
            }
            BackstageSacrifice => {
                if !self.mechanics.used.insert((0, "backstage".into())) {
                    return Err("전투당 한 번 사용할 수 있습니다.".into());
                }
                self.make_corpse(single()?).map_err(err)?;
                self.mechanics.draws += 2;
            }
            MassCrash => {
                let (friends, enemies): (Vec<_>, Vec<_>) =
                    targets.iter().copied().partition(|id| ally(self, *id));
                if friends.is_empty() || friends.len() != enemies.len() {
                    return Err("희생할 아군과 제거할 적을 같은 수로 선택하세요.".into());
                }
                for id in friends.into_iter().chain(enemies) {
                    self.kill_nonroyal(id, true).map_err(err)?;
                }
            }
            Masquerade | InterludeWaltz => {
                if targets.len() != 2 {
                    return Err("아군 둘을 선택하세요.".into());
                }
                let (a, p) = self.located(targets[0]).ok_or_else(fail)?;
                let (b, q) = self.located(targets[1]).ok_or_else(fail)?;
                if kind == InterludeWaltz
                    && (a.file - b.file).abs().max((a.rank - b.rank).abs()) != 1
                {
                    return Err("인접한 기물을 선택하세요.".into());
                }
                self.board.set_piece(a, Some(q)).map_err(|_| fail())?;
                self.board.set_piece(b, Some(p)).map_err(|_| fail())?;
                self.sync_players();
            }
            PullTablecloth => {
                if !self.mechanics.turn_used.insert("tablecloth".into()) {
                    return Err("턴당 한 번 사용할 수 있습니다.".into());
                }
                let id = single()?;
                self.queue_corpse_move(id).map_err(err)?;
                if self
                    .mechanics
                    .choices
                    .last()
                    .is_none_or(|c| c.kind != "move_corpse")
                {
                    return Err("시체 옆에 빈 칸이 없습니다.".into());
                }
            }
            DeliveredFeast => {
                if targets.len() != 2 {
                    return Err("아군과 시체를 하나씩 선택하세요.".into());
                }
                let id = *targets
                    .iter()
                    .find(|id| ally(self, **id))
                    .ok_or_else(fail)?;
                let corpse = *targets
                    .iter()
                    .find(|id| self.located(**id).is_some_and(|(_, p)| p.kind == CORPSE))
                    .ok_or_else(fail)?;
                let (a, _) = self.located(id).unwrap();
                let (b, _) = self.located(corpse).unwrap();
                if (a.file - b.file).abs().max((a.rank - b.rank).abs()) > 2 {
                    return Err("시체는 아군으로부터 2칸 이내여야 합니다.".into());
                }
                self.board.set_piece(b, None).map_err(|_| fail())?;
                self.eat(id).map_err(err)?;
            }
            LastSupper => {
                let friends = targets
                    .iter()
                    .copied()
                    .filter(|id| ally(self, *id))
                    .collect::<Vec<_>>();
                let corpses = targets
                    .iter()
                    .copied()
                    .filter(|id| self.located(*id).is_some_and(|(_, p)| p.kind == CORPSE))
                    .collect::<Vec<_>>();
                if corpses.is_empty() || corpses.len() > 3 || friends.len() != corpses.len() {
                    return Err("시체 1~3개와 같은 수의 아군을 선택하세요.".into());
                }
                for id in corpses {
                    let (at, _) = self.located(id).unwrap();
                    self.board.set_piece(at, None).map_err(|_| fail())?;
                }
                for id in friends {
                    self.grant_agile(id, 1);
                }
            }
            EndlessJoy => {
                if !self.mechanics.used.insert((0, "joy".into())) {
                    return Err("전투당 한 번 사용할 수 있습니다.".into());
                }
                self.mechanics.frenzy = true;
            }
            LastStand => {
                if self.living(PLAYER).len() > self.players.len() / 2 {
                    return Err("아군 수가 초기 편성의 절반 이하여야 합니다.".into());
                }
                for id in self.living(PLAYER) {
                    self.grant_agile(id, 3);
                    let (at, mut piece) = self.located(id).unwrap();
                    if let Some(upgrade) =
                        pieces::definition(piece.kind).and_then(|d| d.upgrades.first())
                    {
                        piece.kind = upgrade.target;
                        self.board.set_piece(at, Some(piece)).map_err(|_| fail())?;
                        if let Some(p) = self.players.iter_mut().find(|p| p.id == id) {
                            p.kind = upgrade.target;
                            p.upgrades.push(upgrade.key.into());
                        }
                    }
                }
                self.mechanics.last_stand = true;
            }
            InfantrySupport => {
                if !targets.is_empty() || cells.len() != 1 || !(48..64).contains(&cells[0]) {
                    return Err("1~2랭크의 빈 칸을 선택하세요.".into());
                }
                let at = Position::new((cells[0] % 8) as i32, (cells[0] / 8) as i32);
                if self.board.piece_at(at).map_err(|_| fail())?.is_some() {
                    return Err("빈 칸을 선택하세요.".into());
                }
                let old = self.players.iter().position(|p| p.origin == at);
                let id = if let Some(i) = old {
                    if self.players[i].position.is_some() {
                        return Err("이 칸을 시작 칸으로 쓰는 기물이 살아 있습니다.".into());
                    }
                    let p = &mut self.players[i];
                    p.kind = pieces::AXE_INFANTRY;
                    p.position = Some(at);
                    p.upgrades.clear();
                    p.id
                } else {
                    let id = self
                        .players
                        .iter()
                        .map(|p| p.id)
                        .chain(self.board.cells().iter().flatten().map(|p| p.id))
                        .max()
                        .unwrap_or(200)
                        + 1;
                    self.players.push(PlayerPiece {
                        id,
                        kind: pieces::AXE_INFANTRY,
                        origin: at,
                        position: Some(at),
                        moved: false,
                        upgrades: vec![],
                        persistent: BTreeMap::new(),
                    });
                    id
                };
                self.board
                    .set_piece(
                        at,
                        Some(Piece {
                            id,
                            kind: pieces::AXE_INFANTRY,
                            owner: PLAYER,
                        }),
                    )
                    .map_err(|_| fail())?;
                let ingredient = self
                    .mechanics
                    .relics
                    .contains(&crate::content::Relic::CarnivalFlare)
                    || self
                        .board
                        .cells()
                        .iter()
                        .flatten()
                        .any(|p| p.kind == pieces::ETERNAL_GUEST);
                self.status_mut(id).ingredient = ingredient;
                self.sync_players();
            }
            Volley => {
                let mut victims = HashSet::new();
                for id in self.living(PLAYER) {
                    let (from, _) = self.located(id).unwrap();
                    for a in self.preview_actions(from).map_err(err)? {
                        for at in a.captures {
                            if let Some(p) = self.board.piece_at(at).map_err(|_| fail())? {
                                if p.owner == ENEMY
                                    && p.kind != pieces::CORPSE
                                    && (!self.is_royal(p.id) || p.kind == WAREHOUSE)
                                {
                                    victims.insert(p.id);
                                }
                            }
                        }
                    }
                }
                let count = victims.len() as u32;
                if self.mechanics.ammo < count {
                    return Err("탄약이 부족합니다.".into());
                }
                let mut victims = victims.into_iter().collect::<Vec<_>>();
                victims.sort_unstable();
                for id in victims {
                    self.kill_nonroyal(id, true).map_err(err)?;
                }
                return Ok(count);
            }
            FormLine => {
                if targets.is_empty()
                    || targets.len() != cells.len()
                    || cells.iter().collect::<HashSet<_>>().len() != cells.len()
                    || cells.iter().any(|c| *c >= 64)
                {
                    return Err("각 아군의 새 배치 칸을 선택하세요.".into());
                }
                let moving = targets
                    .iter()
                    .map(|id| self.located(*id).ok_or_else(fail))
                    .collect::<Result<Vec<_>, _>>()?;
                for (at, _) in &moving {
                    self.board.set_piece(*at, None).map_err(|_| fail())?;
                }
                for ((_, p), c) in moving.into_iter().zip(cells) {
                    let at = Position::new((c % 8) as i32, (c / 8) as i32);
                    if self.board.piece_at(at).map_err(|_| fail())?.is_some() {
                        return Err("다른 기물이 있는 칸에는 배치할 수 없습니다.".into());
                    }
                    self.board.set_piece(at, Some(p)).map_err(|_| fail())?;
                }
                self.sync_players();
            }
            _ => return Err(fail()),
        }
        Ok(d.cost)
    }
}
