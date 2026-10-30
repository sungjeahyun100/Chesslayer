//! An unsaved battle board for trying the real piece rules without an expedition.
use crate::{
    cards::{self, Card, CardCombat},
    content::{
        cards::{CardDefinition, CardKind, CardSelection},
        pieces, Rng, HAND_LIMIT,
    },
    Battle, Effect, PieceId, PieceType, PlayerId, Position,
};
use serde::Serialize;

fn position(cell: usize) -> Result<Position, String> {
    if cell >= 64 {
        return Err("보드 밖의 칸입니다.".into());
    }
    Ok(Position::new((cell % 8) as i32, (cell / 8) as i32))
}
fn cell(pos: Position) -> usize {
    (pos.rank as usize) * 8 + pos.file as usize
}
fn battle_error(error: crate::Error) -> String {
    match error {
        crate::Error::InvalidSetup => "그 칸에 기물을 배치할 수 없습니다.".into(),
        crate::Error::IllegalAction => "현재 기물에 허용되지 않는 행동입니다.".into(),
        crate::Error::Finished => "진행할 수 없는 보드 상태입니다.".into(),
        _ => format!("행동을 수행할 수 없습니다: {error:?}"),
    }
}

#[derive(Serialize)]
pub struct SandboxPiece {
    pub id: u32,
    pub kind: PieceType,
    pub key: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub owner: PlayerId,
    pub cell: usize,
    pub statuses: Vec<String>,
    pub ward: bool,
}
#[derive(Serialize)]
pub struct SandboxAction {
    pub index: usize,
    pub target: usize,
    pub destination: usize,
    pub capture: bool,
    pub special: bool,
    pub promotion: Option<PieceType>,
    pub route: Option<Vec<usize>>,
    pub chance: Option<u8>,
    pub blast: Vec<usize>,
    pub paths: Vec<Vec<usize>>,
}
#[derive(Serialize)]
pub struct SandboxSelfDestruct {
    pub name: &'static str,
    pub blast: Vec<usize>,
}
#[derive(Serialize)]
pub struct SandboxChoice {
    pub name: String,
    pub targets: Vec<u32>,
    pub can_finish: bool,
}
#[derive(Serialize)]
pub struct SandboxSnapshot {
    pub pieces: Vec<SandboxPiece>,
    pub actions: Vec<SandboxAction>,
    pub self_destruct: Option<SandboxSelfDestruct>,
    pub fallout: Vec<SandboxFallout>,
    pub turn: PlayerId,
    pub actions_left: u8,
    pub choice: Option<SandboxChoice>,
    pub events: Vec<String>,
    pub deck: Vec<SandboxDeckCard>,
    pub hand: Vec<SandboxHandCard>,
    pub draw_count: usize,
    pub discard_count: usize,
    pub exhausted_count: usize,
    pub mana: u32,
    pub relics: Vec<crate::content::Relic>,
    pub dead_pawns: Vec<SandboxDeadPawn>,
}
#[derive(Serialize)]
pub struct SandboxDeadPawn {
    pub id: PieceId,
    pub name: &'static str,
}
#[derive(Serialize)]
pub struct SandboxFallout {
    pub cell: usize,
    pub remaining: u32,
}
#[derive(Serialize)]
pub struct SandboxDeckCard {
    pub id: u32,
    pub definition: CardDefinition,
}
#[derive(Serialize)]
pub struct SandboxHandCard {
    pub id: u32,
    pub definition: CardDefinition,
    pub targets: Vec<u32>,
    pub disabled_reason: Option<&'static str>,
}

pub struct Sandbox {
    battle: Battle,
    next_id: u32,
    deck: Vec<Card>,
    cards: CardCombat,
    rng: Rng,
    next_card_id: u32,
}
impl Sandbox {
    pub fn new() -> Result<Self, String> {
        let mut rng = Rng::new(0);
        let cards = CardCombat::start(&[], &mut rng);
        let mut battle = Battle::sandbox().map_err(battle_error)?;
        battle.set_ammunition(cards.mana);
        Ok(Self {
            battle,
            next_id: 1,
            deck: vec![],
            cards,
            rng,
            next_card_id: 1,
        })
    }
    pub fn clear(&mut self) -> Result<(), String> {
        self.battle = Battle::sandbox().map_err(battle_error)?;
        self.battle.set_ammunition(self.cards.mana);
        Ok(())
    }
    pub fn set_relic(&mut self, relic: crate::content::Relic, enabled: bool) -> Result<(), String> {
        self.battle.sandbox_set_relic(relic, enabled)
    }
    pub fn place(&mut self, kind: PieceType, owner: PlayerId, at: usize) -> Result<(), String> {
        self.battle
            .sandbox_place(self.next_id, kind, owner, position(at)?)
            .map_err(battle_error)?;
        self.next_id += 1;
        Ok(())
    }
    pub fn remove(&mut self, at: usize) -> Result<(), String> {
        self.battle
            .sandbox_remove(position(at)?)
            .map_err(battle_error)
    }
    pub fn change_kind(&mut self, at: usize, kind: PieceType) -> Result<(), String> {
        self.battle
            .sandbox_change_kind(position(at)?, kind)
            .map_err(battle_error)
    }
    pub fn resolve_choice(&mut self, target: u32) -> Result<(), String> {
        self.battle.resolve_choice(target)?;
        self.resource_effects();
        Ok(())
    }
    pub fn act(&mut self, from: usize, index: usize) -> Result<(), String> {
        let actions = self
            .battle
            .sandbox_actions(position(from)?)
            .map_err(battle_error)?;
        let action = actions.get(index).ok_or("이동할 수 없는 칸입니다.")?;
        self.battle
            .sandbox_apply_action(action)
            .map_err(battle_error)?;
        self.resource_effects();
        Ok(())
    }
    pub fn detonate(&mut self, from: usize) -> Result<(), String> {
        self.battle
            .sandbox_drone_ability(position(from)?)
            .map_err(battle_error)?;
        self.resource_effects();
        Ok(())
    }
    pub fn progress_effects(&mut self) -> Result<(), String> {
        self.battle
            .sandbox_progress_effects()
            .map_err(battle_error)?;
        self.cards.end_turn();
        self.cards.begin_turn(&mut self.rng, false);
        self.resource_effects();
        Ok(())
    }
    fn resource_effects(&mut self) {
        let (draw, mana, ammo) = self.battle.take_resource_effects();
        self.cards.draw_cards(draw as usize, &mut self.rng);
        self.cards.mana = self
            .cards
            .mana
            .saturating_add_signed(mana.saturating_add(ammo));
        self.battle.set_ammunition(self.cards.mana);
    }
    pub fn add_card(&mut self, kind: CardKind) {
        let card = Card {
            id: self.next_card_id,
            kind,
        };
        self.next_card_id += 1;
        self.deck.push(card.clone());
        self.cards.draw.push(card);
    }
    pub fn remove_card(&mut self, id: u32) -> Result<(), String> {
        let index = self
            .deck
            .iter()
            .position(|card| card.id == id)
            .ok_or("덱에 없는 카드입니다.")?;
        self.deck.remove(index);
        for pile in [
            &mut self.cards.draw,
            &mut self.cards.hand,
            &mut self.cards.discard,
            &mut self.cards.exhausted,
        ] {
            pile.retain(|card| card.id != id);
        }
        Ok(())
    }
    pub fn clear_deck(&mut self) {
        self.deck.clear();
        self.cards = CardCombat::start(&[], &mut self.rng);
        self.battle.set_ammunition(self.cards.mana);
    }
    pub fn reset_cards(&mut self) {
        self.cards = CardCombat::start(&self.deck, &mut self.rng);
        self.battle.set_ammunition(self.cards.mana);
    }
    pub fn draw_card(&mut self) -> Result<(), String> {
        let before = self.cards.hand.len();
        self.cards.draw_cards(1, &mut self.rng);
        if self.cards.hand.len() == before {
            return Err("뽑을 카드가 없거나 손패가 가득 찼습니다.".into());
        }
        Ok(())
    }
    pub fn redraw_hand(&mut self) {
        for card in std::mem::take(&mut self.cards.hand) {
            if card.kind.definition().ethereal {
                self.cards.exhausted.push(card);
            } else {
                self.cards.discard.push(card);
            }
        }
        self.cards.draw_cards(HAND_LIMIT, &mut self.rng);
    }
    pub fn add_to_hand(&mut self, kind: CardKind) {
        self.cards.hand.push(Card {
            id: self.next_card_id,
            kind,
        });
        self.next_card_id += 1;
    }
    pub fn set_mana(&mut self, amount: u32) {
        self.cards.mana = amount;
        self.battle.set_ammunition(amount);
    }
    pub fn play_card(&mut self, id: u32, selection: CardSelection) -> Result<(), String> {
        if self.battle.pending_choice().is_some() {
            return Err("먼저 발동한 효과의 대상을 선택하세요.".into());
        }
        let index = self
            .cards
            .hand
            .iter()
            .position(|card| card.id == id)
            .ok_or("손패에 없는 카드입니다.")?;
        let definition = self.cards.hand[index].kind.definition();
        let (cost, removed) = cards::resolve_selection(
            &mut self.battle,
            &mut self.cards,
            index,
            &selection,
            definition.clone(),
        )?;
        self.battle.events.push(format!(
            "{} 사용 · 마나 −{}{}",
            definition.name,
            cost,
            removed
                .map(|n| format!(" · {n}기 처리"))
                .unwrap_or_default()
        ));
        self.resource_effects();
        Ok(())
    }
    pub fn snapshot(&self, selected: Option<usize>) -> SandboxSnapshot {
        let pieces = self
            .battle
            .board
            .cells()
            .iter()
            .enumerate()
            .filter_map(|(at, piece)| {
                let piece = piece.as_ref()?;
                let definition = pieces::definition(piece.kind)?;
                Some(SandboxPiece {
                    id: piece.id,
                    kind: piece.kind,
                    key: definition.key,
                    name: definition.name,
                    description: definition.description,
                    owner: piece.owner,
                    cell: at,
                    statuses: self.battle.status_labels(piece.id),
                    ward: self.battle.has_ward(piece.id),
                })
            })
            .collect();
        let self_destruct = selected.and_then(|at| position(at).ok()).and_then(|pos| {
            let piece = self.battle.board.piece_at(pos).ok().flatten()?;
            let name = match piece.kind {
                pieces::DRONE | pieces::MACHINE_GUN_DRONE | pieces::GRENADE_LAUNCHER_DRONE => {
                    "십자포화 자폭"
                }
                pieces::C4_DRONE => "광역 자폭",
                pieces::TACTICAL_NUKE_DRONE => "전술핵 자폭",
                _ => return None,
            };
            Some(SandboxSelfDestruct {
                name,
                blast: self
                    .battle
                    .blast_preview(piece.kind, pos)
                    .into_iter()
                    .map(cell)
                    .collect(),
            })
        });
        let actions = selected
            .and_then(|at| position(at).ok())
            .and_then(|pos| self.battle.sandbox_actions(pos).ok())
            .unwrap_or_default()
            .into_iter()
            .enumerate()
            .filter(|(_, action)| {
                !(action.first.effect == Effect::Catch && action.first.take == action.first.from)
            })
            .map(|(index, action)| {
                let kind = self
                    .battle
                    .board
                    .piece_at(action.first.from)
                    .ok()
                    .flatten()
                    .map(|p| p.kind)
                    .unwrap_or(0);
                let last = action.steps().last().expect("actions have a step");
                let special =
                    last.move_to == action.first.from && action.first.take != action.first.from;
                let target = if special {
                    action.first.take
                } else {
                    last.move_to
                };
                let capture = action.steps().any(|step| {
                    !matches!(step.effect, Effect::Move | Effect::Shift)
                        && self
                            .battle
                            .board
                            .piece_at(step.take)
                            .ok()
                            .flatten()
                            .is_some()
                });
                let (blast, paths) = if special {
                    self.battle
                        .shot_effect_preview(kind, action.first.from, action.first.take)
                } else {
                    (vec![], vec![])
                };
                let promotion = last.transition.as_deref().and_then(|key| {
                    pieces::ALL
                        .iter()
                        .find(|definition| definition.key == key)
                        .map(|definition| definition.kind)
                });
                SandboxAction {
                    index,
                    target: cell(target),
                    destination: cell(last.move_to),
                    capture,
                    special,
                    promotion,
                    route: action
                        .route
                        .map(|route| route.into_iter().map(cell).collect()),
                    chance: if special {
                        self.battle.hit_chance(action.first.from, action.first.take)
                    } else {
                        None
                    },
                    blast: blast.into_iter().map(cell).collect(),
                    paths: paths
                        .into_iter()
                        .map(|path| path.into_iter().map(cell).collect())
                        .collect(),
                }
            })
            .collect();
        let choice = self.battle.pending_choice().map(|view| SandboxChoice {
            name: view.name,
            targets: view.targets,
            can_finish: view.can_finish,
        });
        let deck = self
            .deck
            .iter()
            .map(|card| SandboxDeckCard {
                id: card.id,
                definition: card.kind.definition(),
            })
            .collect();
        let hand = self
            .cards
            .hand
            .iter()
            .map(|card| {
                let definition = card.kind.definition();
                let targets = if definition.targeted {
                    (definition.target_candidates)(&self.battle)
                } else {
                    vec![]
                };
                let disabled_reason = if self.battle.pending_choice().is_some() {
                    Some("발동 효과의 대상을 먼저 선택하세요.")
                } else if self.cards.mana
                    < if definition.cost_per_target {
                        1
                    } else {
                        definition.cost
                    }
                {
                    Some("마나가 부족합니다.")
                } else if definition.targeted && targets.is_empty() {
                    Some("선택 가능한 대상이 없습니다.")
                } else {
                    None
                };
                SandboxHandCard {
                    id: card.id,
                    definition,
                    targets,
                    disabled_reason,
                }
            })
            .collect();
        SandboxSnapshot {
            pieces,
            actions,
            self_destruct,
            fallout: self
                .battle
                .fallout_cells()
                .into_iter()
                .map(|(pos, remaining)| SandboxFallout {
                    cell: cell(pos),
                    remaining,
                })
                .collect(),
            turn: self.battle.turn,
            actions_left: self.battle.actions_left,
            choice,
            events: self.battle.events.clone(),
            deck,
            hand,
            draw_count: self.cards.draw.len(),
            discard_count: self.cards.discard.len(),
            exhausted_count: self.cards.exhausted.len(),
            mana: self.cards.mana,
            relics: self.battle.sandbox_relics().to_vec(),
            dead_pawns: self
                .battle
                .player_pieces()
                .iter()
                .filter(|p| p.position.is_none() && pieces::is_pawn(p.kind))
                .map(|p| SandboxDeadPawn {
                    id: p.id,
                    name: crate::piece_name(p.kind),
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn captures_and_royal_removal_do_not_end_the_sandbox() {
        let mut sandbox = Sandbox::new().unwrap();
        sandbox.place(pieces::KNIGHT, 0, 35).unwrap(); // d4
        sandbox.place(pieces::KING, 1, 20).unwrap(); // e6
        let action = sandbox
            .snapshot(Some(35))
            .actions
            .into_iter()
            .find(|action| action.target == 20)
            .unwrap();
        sandbox.act(35, action.index).unwrap();
        assert_eq!(sandbox.snapshot(None).pieces[0].cell, 20);
        assert!(!sandbox.snapshot(Some(20)).actions.is_empty());
        sandbox.place(pieces::ROOK, 1, 56).unwrap();
        assert!(!sandbox.snapshot(Some(56)).actions.is_empty());
    }

    #[test]
    fn card_uses_shared_effect_and_piles() {
        let mut sandbox = Sandbox::new().unwrap();
        sandbox.place(pieces::PAWN, 0, 52).unwrap();
        sandbox.add_to_hand(CardKind::Ward);
        let id = sandbox.snapshot(None).hand[0].id;
        sandbox
            .play_card(id, CardSelection::single(Some(1)))
            .unwrap();
        assert!(sandbox.battle.has_ward(1));
        assert_eq!(sandbox.snapshot(None).discard_count, 1);
        assert_eq!(sandbox.snapshot(None).mana, 2);
    }
}
