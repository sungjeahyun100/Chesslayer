//! movement rules. The call sites retain their original ordering.
use super::*;

impl Battle {
    /// Targets for the king board manipulation action.
    pub fn king_reposition_targets(&self, from: Position) -> Result<Vec<Position>, Error> {
        let actor = self.board.piece_at(from)?.ok_or(Error::IllegalAction)?;
        if !matches!(actor.kind, pieces::TEMPEST_KING | pieces::TYPHOON_KING)
            || self.phase != Phase::Active
            || actor.owner != self.turn
            || self.pending_choice().is_some()
            || !self.can_act(actor)
            || (self.normal_actions_used >= self.side_actions(actor.owner)
                && self.agility(actor.id) == 0
                && !self.limiter.contains_key(&actor.id))
            || self
                .limiter
                .get(&actor.id)
                .is_some_and(|state| state.moves_left == 0)
            || (self.limiter_turn
                && !self.limiter.contains_key(&actor.id)
                && self.agility(actor.id) == 0
                && self.normal_actions_used >= self.actions_per_turn)
        {
            return Ok(vec![]);
        }
        Ok(self
            .board
            .cells()
            .iter()
            .enumerate()
            .filter_map(|(index, piece)| {
                piece
                    .filter(|p| p.owner != actor.owner && p.owner != 2)
                    .map(|_| {
                        Position::new(
                            (index % self.board.width()) as i32,
                            (index / self.board.width()) as i32,
                        )
                    })
            })
            .collect())
    }
    /// Permute whole enemy identities while preserving status and royal IDs.
    pub fn reposition_enemies(
        &mut self,
        from: Position,
        targets: &[Position],
    ) -> Result<(), Error> {
        let actor = self.board.piece_at(from)?.ok_or(Error::IllegalAction)?;
        let required = if actor.kind == pieces::TEMPEST_KING {
            2
        } else if actor.kind == pieces::TYPHOON_KING && (2..=6).contains(&targets.len()) {
            targets.len()
        } else {
            return Err(Error::IllegalAction);
        };
        if targets.len() != required {
            return Err(Error::IllegalAction);
        }
        let available = self.king_reposition_targets(from)?;
        let mut unique = HashSet::new();
        if !targets
            .iter()
            .all(|p| unique.insert(*p) && available.contains(p))
        {
            return Err(Error::IllegalAction);
        }
        let pieces: Vec<_> = targets
            .iter()
            .map(|&p| {
                self.board
                    .piece_at(p)
                    .map(|piece| piece.expect("validated target"))
            })
            .collect::<Result<_, _>>()?;
        let mut shuffled = pieces.clone();
        if actor.kind == pieces::TEMPEST_KING {
            shuffled.swap(0, 1);
        } else {
            self.mechanics.rng.shuffle(&mut shuffled);
            if shuffled == pieces {
                shuffled.rotate_left(1);
            }
        }
        for (&position, piece) in targets.iter().zip(shuffled) {
            self.board.set_piece(position, Some(piece))?;
        }
        self.events.push(format!(
            "{}: 적 기물 {}기의 위치를 바꾸었습니다.",
            piece_name(actor.kind),
            targets.len()
        ));
        self.complete_nonmovement_action(actor)
    }
    pub(crate) fn generated_legal_actions(
        &self,
        from: Position,
    ) -> Result<Vec<GeneratedAction<'_>>, Error> {
        Ok(self
            .legal_actions(from)?
            .into_iter()
            .map(|action| GeneratedAction {
                battle: self,
                action,
            })
            .collect())
    }
    /// Generate and apply using one exclusive borrow.
    pub(crate) fn select_and_apply<F>(
        &mut self,
        select: F,
    ) -> Result<Option<(DefaultAction, Piece, Option<Piece>)>, Error>
    where
        F: for<'a> FnOnce(&'a Battle) -> Result<Option<GeneratedAction<'a>>, Error>,
    {
        let Some(selected) = select(self)? else {
            return Ok(None);
        };
        if !std::ptr::eq(selected.battle, self) {
            return Err(Error::IllegalAction);
        }
        let action = selected.action;
        let piece = self
            .board
            .piece_at(action.first.from)?
            .ok_or(Error::IllegalAction)?;
        let captured = action.steps().find_map(|s| {
            self.board
                .piece_at(s.take)
                .ok()
                .flatten()
                .filter(|p| p.owner != piece.owner)
        });
        self.apply_piece_action_unchecked(&action)?;
        Ok(Some((action, piece, captured)))
    }
    pub fn preview_actions(&self, from: Position) -> Result<Vec<ActionPreview>, Error> {
        let actions = self.actions_from(from, false)?;
        let owner = self
            .board
            .piece_at(from)?
            .ok_or(Error::IllegalAction)?
            .owner;
        actions
            .into_iter()
            .map(|action| {
                let destination = action.steps().last().unwrap().move_to;
                let mut captures = Vec::new();
                if action.continuation.is_empty() {
                    let step = &action.first;
                    if !matches!(step.effect, crate::Effect::Move | crate::Effect::Shift)
                        && self
                            .board
                            .piece_at(step.take)?
                            .is_some_and(|p| p.owner != owner)
                    {
                        captures.push(step.take);
                    }
                } else {
                    let mut board = self.board.clone();
                    for step in action.steps() {
                        if !matches!(step.effect, crate::Effect::Move | crate::Effect::Shift)
                            && board.piece_at(step.take)?.is_some_and(|p| p.owner != owner)
                        {
                            captures.push(step.take);
                        }
                        apply_default_action(&mut board, &DefaultAction::one(step.clone()))?;
                    }
                }
                Ok(ActionPreview {
                    destination,
                    captures,
                    route: action.route,
                })
            })
            .collect()
    }
    pub(in crate::battle) fn actions_from(
        &self,
        from: Position,
        enforce_turn: bool,
    ) -> Result<Vec<DefaultAction>, Error> {
        if self.phase != Phase::Active {
            return Err(Error::Finished);
        }
        let piece = self.board.piece_at(from)?.ok_or(Error::IllegalAction)?;
        if enforce_turn
            && piece.owner != self.turn
            && !(self.card_movement && self.turn == PLAYER && self.player_controls(piece))
        {
            return Err(Error::WrongTurn);
        }
        if (enforce_turn && self.pending_choice().is_some()) || !self.can_act(piece) {
            return Ok(vec![]);
        }
        if enforce_turn
            && !(self.card_movement && self.turn == PLAYER)
            && self.normal_actions_used >= self.side_actions(piece.owner)
            && self.agility(piece.id) == 0
            && !self.limiter.contains_key(&piece.id)
        {
            return Ok(vec![]);
        }
        if self
            .limiter
            .get(&piece.id)
            .is_some_and(|s| s.moves_left == 0)
        {
            return Ok(vec![]);
        }
        if enforce_turn
            && !(self.card_movement && self.turn == PLAYER)
            && self.limiter_turn
            && !self.limiter.contains_key(&piece.id)
            && self.agility(piece.id) == 0
            && self.normal_actions_used >= self.actions_per_turn
        {
            return Ok(vec![]);
        }
        let rolled_kind = self.gambler_die(piece.id).map(|die| {
            [
                pieces::PAWN,
                pieces::KNIGHT,
                pieces::BISHOP,
                pieces::ROOK,
                pieces::QUEEN,
                pieces::KING,
            ][usize::from(die - 1)]
        });
        let mut actions = self.piece_actions(rolled_kind.unwrap_or(piece.kind), from)?;
        if piece.kind == pieces::JOCKEY && rolled_kind.is_some_and(|kind| kind != pieces::KING) {
            actions.extend(self.piece_actions(pieces::KING, from)?);
        }
        actions.extend(self.special_actions(from));
        if piece.owner == PLAYER
            && self.turn == PLAYER
            && self.mechanics.reckless.contains(&piece.id)
        {
            let extra: Vec<_> = actions
                .iter()
                .filter_map(|a| {
                    if !a.continuation.is_empty() || a.first.effect != crate::Effect::Move {
                        return None;
                    }
                    let one = a.first.move_to;
                    let dx = one.file - from.file;
                    let dy = one.rank - from.rank;
                    if dx.abs().max(dy.abs()) != 1 {
                        return None;
                    }
                    let to = Position::new(one.file + dx, one.rank + dy);
                    if !self.board.contains(to)
                        || self.board.piece_at(to).ok().flatten().is_some()
                        || self.board.piece_at(one).ok().flatten().is_some()
                    {
                        return None;
                    }
                    Some(DefaultAction::one(crate::ActionStep::new(
                        from,
                        to,
                        to,
                        crate::Effect::Move,
                    )))
                })
                .collect();
            actions.extend(extra);
        }
        if piece.kind == pieces::EMPEROR || piece.kind == pieces::TRIBAL_DICTATOR {
            if let Some(absorbed) = self.absorbed.get(&piece.id) {
                let mut kinds: Vec<_> = absorbed.iter().copied().collect();
                kinds.sort_unstable();
                for kind in kinds {
                    if piece.kind == pieces::TRIBAL_DICTATOR {
                        for mut action in self.boss_copied_movement_actions(kind, from)? {
                            let original = action.first.transition.take().unwrap_or_default();
                            action.first.transition = Some(format!("boss_copy:{kind}:{original}"));
                            actions.push(action);
                        }
                    } else {
                        actions.extend(self.piece_actions(kind, from)?);
                    }
                }
            }
        }
        if (pieces::definition(piece.kind).is_some_and(|d| d.initial_double_step)
            || rolled_kind == Some(pieces::PAWN))
            && !self.moved_ids.contains(&piece.id)
            && from.rank == if piece.owner == PLAYER { 6 } else { 1 }
        {
            let double = self.initial_double_step.execute(&self.board, from)?;
            if let Some(second) = double.get(1) {
                actions.push(second.clone())
            }
        }
        if self.mechanics.suppressed.contains(&piece.id) {
            actions.retain(|a| a.steps().last().is_some_and(|s| s.move_to == from));
        }
        let definition = pieces::definition(piece.kind).ok_or(Error::InvalidSetup)?;
        let last_rank = if piece.owner == PLAYER { 0 } else { 7 };
        // A promotion is part of the validated move, including captures of the royal.
        // No unpromoted action is offered on the final rank for these families.
        let mut result = Vec::new();
        let mut seen: Option<HashMap<u64, Vec<usize>>> = None;
        for action in actions {
            if piece.owner == ENEMY
                && self.initiative_turns_left > 0
                && self.captures_player(&action)?
            {
                continue;
            }
            if piece.owner == PLAYER
                && self.enemy_initiative_turns_left > 0
                && self.captures_enemy(&action)?
            {
                continue;
            }
            let destination = action.steps().last().unwrap().move_to;
            if destination.rank == last_rank && !definition.promotions.is_empty() {
                for &kind in definition.promotions {
                    let mut promoted = action.clone();
                    let step = promoted
                        .continuation
                        .last_mut()
                        .unwrap_or(&mut promoted.first);
                    step.transition = Some(pieces::definition(kind).unwrap().key.into());
                    push_unique_action(&mut result, &mut seen, promoted);
                }
            } else {
                push_unique_action(&mut result, &mut seen, action);
            }
        }
        self.filter_hunger(piece, &mut result);
        if self.stats(piece.id).disarmed {
            result.retain(|a| {
                !a.steps().any(|s| {
                    !matches!(s.effect, crate::Effect::Move | crate::Effect::Shift)
                        && self
                            .board
                            .piece_at(s.take)
                            .ok()
                            .flatten()
                            .is_some_and(|p| p.owner != piece.owner)
                })
            });
        }
        if enforce_turn && self.card_movement && self.turn == PLAYER {
            result.retain(|a| {
                if a.first.effect == crate::Effect::Catch {
                    piece.owner == PLAYER && self.normal_actions_used == 0
                } else {
                    self.stats(piece.id).move_rights > 0
                }
            });
        }
        if enforce_turn && self.is_boss_actor(piece) {
            result.retain(|action| self.boss_action_available(action));
        }
        Ok(result)
    }
    pub(in crate::battle) fn piece_actions(
        &self,
        kind: PieceType,
        from: Position,
    ) -> Result<Vec<DefaultAction>, Error> {
        let definition = pieces::definition(kind).ok_or(Error::InvalidSetup)?;
        if let Some(calculate) = definition.native_movement {
            Ok(calculate(&self.board, from)?)
        } else {
            Ok(self
                .programs
                .get(&kind)
                .ok_or(Error::InvalidSetup)?
                .execute(&self.board, from)?)
        }
    }
    pub(in crate::battle) fn captures_player(&self, action: &DefaultAction) -> Result<bool, Error> {
        self.captures_owner(action, PLAYER)
    }
    pub(in crate::battle) fn captures_enemy(&self, action: &DefaultAction) -> Result<bool, Error> {
        self.captures_owner(action, ENEMY)
    }
    pub(in crate::battle) fn captures_owner(
        &self,
        action: &DefaultAction,
        owner: PlayerId,
    ) -> Result<bool, Error> {
        if action.continuation.is_empty() {
            let step = &action.first;
            return Ok(
                !matches!(step.effect, crate::Effect::Move | crate::Effect::Shift)
                    && self
                        .board
                        .piece_at(step.take)?
                        .is_some_and(|p| p.owner == owner),
            );
        }
        let mut next = self.board.clone();
        for step in action.steps() {
            if !matches!(step.effect, crate::Effect::Move | crate::Effect::Shift)
                && next.piece_at(step.take)?.is_some_and(|p| p.owner == owner)
            {
                return Ok(true);
            }
            apply_default_action(&mut next, &DefaultAction::one(step.clone()))?;
        }
        Ok(false)
    }
    pub fn forward_pawn_candidates(&self) -> Vec<PieceId> {
        self.board
            .cells()
            .iter()
            .enumerate()
            .filter_map(|(i, p)| {
                let piece = p.as_ref()?;
                if piece.owner != PLAYER || !piece_name(piece.kind).contains("폰") {
                    return None;
                }
                let from = Position::new(i as i32 % 8, i as i32 / 8);
                let to = Position::new(from.file, from.rank - 1);
                self.board
                    .index(to)
                    .filter(|_| self.board.piece_at(to).ok().flatten().is_none())
                    .map(|_| piece.id)
            })
            .collect()
    }
    pub fn advance_pawns(&mut self, ids: &[PieceId]) -> Result<(), Error> {
        if self.phase != Phase::Active || ids.is_empty() {
            return Err(Error::IllegalAction);
        }
        let eligible = self.forward_pawn_candidates();
        let mut seen = HashSet::new();
        if ids
            .iter()
            .any(|id| !seen.insert(*id) || !eligible.contains(id))
        {
            return Err(Error::IllegalAction);
        }
        let moves: Vec<_> = self
            .board
            .cells()
            .iter()
            .enumerate()
            .filter_map(|(i, p)| {
                let piece = p.as_ref()?;
                ids.contains(&piece.id).then(|| {
                    let from = Position::new(i as i32 % 8, i as i32 / 8);
                    (from, Position::new(from.file, from.rank - 1), *piece)
                })
            })
            .collect();
        let mut destinations = HashSet::new();
        if moves.len() != ids.len() || moves.iter().any(|(_, to, _)| !destinations.insert(*to)) {
            return Err(Error::IllegalAction);
        }
        let mut next = self.board.clone();
        for (from, _, _) in &moves {
            next.set_piece(*from, None)?;
        }
        for (_, to, piece) in &moves {
            next.set_piece(*to, Some(*piece))?;
        }
        self.board = next;
        self.sync_players();
        Ok(())
    }
    pub fn temporary_upgrades(&mut self, choices: &[(PieceId, PieceType)]) -> Result<(), Error> {
        if self.phase != Phase::Active || choices.is_empty() {
            return Err(Error::IllegalAction);
        }
        let mut seen = HashSet::new();
        let mut changes = vec![];
        for &(id, target) in choices {
            if !seen.insert(id) {
                return Err(Error::IllegalAction);
            }
            let (i, piece) = self
                .board
                .cells()
                .iter()
                .enumerate()
                .find_map(|(i, p)| p.filter(|p| p.id == id).map(|p| (i, p)))
                .ok_or(Error::IllegalAction)?;
            if !self.can_temporary_upgrade(piece, target) {
                return Err(Error::IllegalAction);
            }
            changes.push((
                Position::new(i as i32 % 8, i as i32 / 8),
                Piece {
                    kind: target,
                    ..piece
                },
            ));
        }
        let mut next = self.board.clone();
        for (at, piece) in changes {
            next.set_piece(at, Some(piece))?;
        }
        self.board = next;
        self.temporary_upgrades
            .extend(choices.iter().map(|(id, _)| *id));
        for &(id, _) in choices {
            if let Some((_, piece)) = self.piece_position(id) {
                self.roll_gambler_die(piece);
            }
        }
        Ok(())
    }
    pub fn temporary_upgrade_options(&self, id: PieceId) -> Vec<PieceType> {
        if self.phase != Phase::Active {
            return vec![];
        }
        let Some(piece) = self
            .board
            .cells()
            .iter()
            .flatten()
            .find(|piece| piece.id == id)
            .copied()
        else {
            return vec![];
        };
        pieces::definition(piece.kind).map_or_else(Vec::new, |definition| {
            definition
                .upgrades
                .iter()
                .filter(|upgrade| self.can_temporary_upgrade(piece, upgrade.target))
                .map(|upgrade| upgrade.target)
                .collect()
        })
    }
    pub(in crate::battle) fn can_temporary_upgrade(&self, piece: Piece, target: PieceType) -> bool {
        piece.owner == PLAYER
            && pieces::definition(piece.kind).is_some_and(|definition| {
                definition
                    .upgrades
                    .iter()
                    .any(|upgrade| upgrade.target == target)
            })
            && pieces::required_owned(target).iter().all(|&kind| {
                self.players.iter().any(|owned| owned.kind == kind)
                    || self.board.cells().iter().flatten().any(|donor| {
                        donor.owner == PLAYER
                            && donor.kind == kind
                            && self.temporary_upgrades.contains(&donor.id)
                    })
            })
    }
}
