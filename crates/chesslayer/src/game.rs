use crate::{cards::*, content::*, *};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RunPhase {
    Combat,
    Reward,
    Intermission,
    Rest,
    Victory,
    Defeat,
}
#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Command {
    Move { from: usize, to: usize },
    PlayCard { id: u32, target: Option<u32> },
    EndTurn,
    EnemyStep,
    ChooseReward { index: usize },
    Continue,
    RestUpgrade { id: u32 },
    RestRevive { id: u32 },
    RestSkip,
}
#[derive(Clone)]
pub struct Game {
    pub battle: Battle,
    pub cards: CardCombat,
    pub deck: Vec<Card>,
    pub relics: Vec<Relic>,
    pub phase: RunPhase,
    pub node: usize,
    pub seed: u32,
    pub character: String,
    pub log: Vec<String>,
    pub round: u32,
    rng: Rng,
    next_card_id: u32,
}
fn engine_error(e: Error) -> String {
    format!("요청한 행동을 수행할 수 없습니다: {e:?}")
}
fn pos(index: usize) -> Result<Position, String> {
    if index >= 64 {
        return Err("보드 밖의 칸입니다.".into());
    }
    Ok(Position::new((index % 8) as i32, (index / 8) as i32))
}
fn cell(p: Position) -> usize {
    (p.rank * 8 + p.file) as usize
}
impl Game {
    pub fn new(character: &str, seed: u32) -> Result<Self, String> {
        let choice = characters()
            .into_iter()
            .find(|c| c.id == character)
            .ok_or("캐릭터를 선택하세요.")?;
        let deck = starter_deck();
        let battle = Battle::new(BattleStart {
            players: Battle::standard_roster(),
            encounter: encounter(0),
        })
        .map_err(engine_error)?;
        let mut game = Self {
            battle,
            cards: CardCombat::default(),
            deck,
            relics: vec![choice.relic],
            phase: RunPhase::Combat,
            node: 0,
            seed,
            character: choice.name.into(),
            log: vec![],
            round: 1,
            rng: Rng::new(seed),
            next_card_id: 11,
        };
        game.start_battle()?;
        Ok(game)
    }
    fn log(&mut self, message: impl Into<String>) {
        self.log.push(message.into());
        if self.log.len() > 60 {
            self.log.remove(0);
        }
    }
    fn events(&mut self) {
        for e in std::mem::take(&mut self.battle.events) {
            self.log(e);
        }
    }
    fn start_battle(&mut self) -> Result<(), String> {
        let index = ROUTE[self.node].ok_or("전투 노드가 아닙니다.")?;
        let mut encounter = encounter(index);
        self.log(format!("{} · 전투 시작", ENCOUNTER_NAMES[index]));
        if self.relics.contains(&Relic::Whirlwind) {
            let mut positions: Vec<_> = encounter.enemies.iter().map(|e| e.position).collect();
            self.rng.shuffle(&mut positions);
            for (enemy, position) in encounter.enemies.iter_mut().zip(positions) {
                enemy.position = position;
            }
            self.log("소용돌이: 적의 시작 위치를 섞었습니다.");
        }
        let haste = self.relics.contains(&Relic::Haste);
        let options = BattleOptions {
            actions_per_turn: if haste { HASTE_ACTIONS } else { 1 },
            self_defense: self.relics.contains(&Relic::SelfDefense),
        };
        self.battle = Battle::with_options(
            BattleStart {
                players: self.battle.outcome(),
                encounter,
            },
            options,
        )
        .map_err(engine_error)?;
        if haste {
            self.log(format!(
                "가속: 양 진영이 턴마다 최대 {HASTE_ACTIONS}회 이동합니다."
            ));
        }
        self.phase = RunPhase::Combat;
        self.round = 1;
        self.cards = CardCombat::start(&self.deck, &mut self.rng);
        self.events();
        self.check_finish();
        Ok(())
    }
    fn require_player(&self) -> Result<(), String> {
        if self.phase != RunPhase::Combat || self.battle.turn != PLAYER {
            return Err("플레이어의 전투 턴에만 사용할 수 있습니다.".into());
        }
        Ok(())
    }
    pub fn dispatch(&mut self, command: Command) -> Result<(), String> {
        match command {
            Command::Move { from, to } => {
                self.require_player()?;
                let from = pos(from)?;
                let to = pos(to)?;
                let action = self
                    .battle
                    .legal_actions(from)
                    .map_err(engine_error)?
                    .into_iter()
                    .find(|a| a.steps().last().is_some_and(|s| s.move_to == to))
                    .ok_or("합법적인 이동이 아닙니다.")?;
                self.perform_action(action)?;
            }
            Command::PlayCard { id, target } => self.play_card(id, target)?,
            Command::EndTurn => {
                self.require_player()?;
                self.battle.end_turn().map_err(engine_error)?;
                self.cards.end_turn();
                self.log("플레이어가 턴을 마쳤습니다.");
            }
            Command::EnemyStep => self.enemy_step()?,
            Command::ChooseReward { index } => {
                if self.phase != RunPhase::Reward {
                    return Err("지금은 보상을 선택할 수 없습니다.".into());
                }
                let kind = *[CardKind::Ward, CardKind::Jeans, CardKind::Gun]
                    .get(index)
                    .ok_or("보상 선택이 잘못되었습니다.")?;
                self.deck.push(Card {
                    id: self.next_card_id,
                    kind,
                });
                self.next_card_id += 1;
                self.log(format!(
                    "보상: {}을 덱에 추가했습니다.",
                    kind.definition().name
                ));
                self.phase = RunPhase::Intermission;
            }
            Command::Continue => {
                if self.phase != RunPhase::Intermission {
                    return Err("보상 또는 휴식을 먼저 완료하세요.".into());
                }
                self.node += 1;
                if self.node >= ROUTE.len() {
                    self.phase = RunPhase::Victory;
                } else if ROUTE[self.node].is_none() {
                    self.phase = RunPhase::Rest;
                    self.log("작은 쉼터 · 강화 또는 부활을 한 번 선택하세요.");
                } else {
                    self.start_battle()?;
                }
            }
            Command::RestUpgrade { id } => {
                if self.phase != RunPhase::Rest {
                    return Err("쉼터에서만 강화할 수 있습니다.".into());
                }
                let mut roster = self.battle.outcome();
                let p = roster
                    .iter_mut()
                    .find(|p| p.id == id && p.kind == PAWN && p.position.is_some())
                    .ok_or("살아 있는 일반 폰을 선택하세요.")?;
                p.kind = TEMPEST_PAWN;
                p.upgrades.push("tempest".into());
                self.battle
                    .replace_roster_at_rest(roster)
                    .map_err(engine_error)?;
                self.log("휴식: 폰이 템페스트 폰으로 강화되었습니다.");
                self.phase = RunPhase::Intermission;
            }
            Command::RestRevive { id } => {
                if self.phase != RunPhase::Rest {
                    return Err("쉼터에서만 부활할 수 있습니다.".into());
                }
                let origin = Battle::standard_roster()
                    .into_iter()
                    .find(|p| p.id == id)
                    .and_then(|p| p.position)
                    .ok_or("잘못된 기물입니다.")?;
                let mut roster = self.battle.outcome();
                if roster.iter().any(|p| p.position == Some(origin)) {
                    return Err("원래 초기 위치가 점유되어 부활할 수 없습니다.".into());
                }
                let p = roster
                    .iter_mut()
                    .find(|p| p.id == id && p.position.is_none() && p.kind != KING)
                    .ok_or("사망한 아군 기물을 선택하세요.")?;
                p.position = Some(origin); // Identity, upgrades and prior movement history persist.
                self.battle
                    .replace_roster_at_rest(roster)
                    .map_err(engine_error)?;
                self.log(format!("휴식: {}에서 아군이 부활했습니다.", square(origin)));
                self.phase = RunPhase::Intermission;
            }
            Command::RestSkip => {
                if self.phase != RunPhase::Rest {
                    return Err("휴식 중이 아닙니다.".into());
                }
                self.phase = RunPhase::Intermission;
                self.log("휴식을 마쳤습니다.");
            }
        }
        Ok(())
    }
    fn perform_action(&mut self, action: DefaultAction) -> Result<(), String> {
        let side = self.battle.turn;
        let piece = self
            .battle
            .board
            .piece_at(action.first.from)
            .map_err(|e| engine_error(e.into()))?
            .ok_or("기물이 없습니다.")?;
        let destination = action.steps().last().unwrap().move_to;
        let captured = action.steps().find_map(|s| {
            self.battle
                .board
                .piece_at(s.take)
                .ok()
                .flatten()
                .filter(|p| p.owner != side)
        });
        self.battle
            .apply_piece_action(&action)
            .map_err(engine_error)?;
        let still_at = self
            .battle
            .board
            .piece_at(action.first.from)
            .ok()
            .flatten()
            .is_some_and(|p| p.id == piece.id);
        self.log(format!(
            "{} {} · {} → {}{}",
            if side == PLAYER { "백" } else { "흑" },
            piece_name(piece.kind),
            square(action.first.from),
            square(destination),
            if still_at {
                " (가호로 저지)".to_string()
            } else {
                captured
                    .map(|p| format!(" · {} 포획", piece_name(p.kind)))
                    .unwrap_or_default()
            }
        ));
        self.events();
        self.check_finish();
        if self.phase == RunPhase::Combat && self.battle.turn != side {
            if side == PLAYER {
                self.cards.end_turn();
            } else {
                self.round += 1;
                self.cards.begin_turn(&mut self.rng);
            }
        }
        Ok(())
    }
    fn check_finish(&mut self) {
        if let GameResult::Win { winner, reason } = self.battle.result {
            self.cards = CardCombat::default();
            if winner == PLAYER {
                self.phase = if self.node == ROUTE.len() - 1 {
                    RunPhase::Victory
                } else {
                    RunPhase::Reward
                };
                self.log("승리! 적 로얄을 직접 포획했습니다. 생존 진형을 계승합니다.");
            } else {
                self.phase = RunPhase::Defeat;
                self.log(if reason == ResultReason::Counterattack {
                    "역습으로 킹을 잃었습니다. 원정이 끝났습니다."
                } else {
                    "킹이 포획되었습니다. 원정이 끝났습니다."
                });
            }
        }
    }
    fn play_card(&mut self, id: u32, target: Option<u32>) -> Result<(), String> {
        self.require_player()?;
        let index = self
            .cards
            .hand
            .iter()
            .position(|c| c.id == id)
            .ok_or("손패에 없는 카드입니다.")?;
        let d = self.cards.hand[index].kind.definition();
        if self.cards.mana < d.cost {
            return Err("마나가 부족합니다.".into());
        }
        let mut removed = None;
        match d.kind {
            CardKind::Ward | CardKind::FleetingWard => {
                self.battle
                    .grant_ward(target.ok_or("가호를 받을 기물을 선택하세요.")?)
                    .map_err(engine_error)?;
            }
            CardKind::Jeans => {
                if target.is_some() {
                    return Err("청바지는 대상을 선택하지 않습니다.".into());
                }
                let ids: Vec<_> = self
                    .battle
                    .board
                    .cells()
                    .iter()
                    .flatten()
                    .filter(|p| matches!(p.kind, ROOK | QUEEN))
                    .map(|p| p.id)
                    .collect();
                removed = Some(self.battle.remove_nonroyals(&ids).map_err(engine_error)?);
            }
            CardKind::Gun => {
                let target = target.ok_or("진형 맨 앞의 아군 룩을 선택하세요.")?;
                let front = self
                    .gun_front(target)
                    .ok_or("전방부터 룩·룩·퀸을 같은 파일의 연속 세 칸에 배치하세요.")?;
                let ids: Vec<_> = self
                    .battle
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
                removed = Some(self.battle.remove_nonroyals(&ids).map_err(engine_error)?);
            }
        }
        self.cards.spend(index);
        self.log(format!(
            "{} 사용 · 마나 −{}{}",
            d.name,
            d.cost,
            removed
                .map(|n| format!(" · {n}기 제거 (로얄 보호)"))
                .unwrap_or_default()
        ));
        Ok(())
    }
    fn gun_front(&self, id: u32) -> Option<Position> {
        let (i, _) = self
            .battle
            .board
            .cells()
            .iter()
            .enumerate()
            .find(|(_, p)| p.is_some_and(|p| p.id == id && p.owner == PLAYER && p.kind == ROOK))?;
        let front = pos(i).ok()?;
        let behind: Vec<_> = ((front.rank + 1)..8)
            .filter_map(|rank| {
                self.battle
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
        if GUN_ADJACENT_FORMATION
            && (behind[0].0 != front.rank + 1 || behind[1].0 != front.rank + 2)
        {
            return None;
        }
        Some(front)
    }
    fn enemy_step(&mut self) -> Result<(), String> {
        if self.phase != RunPhase::Combat || self.battle.turn != ENEMY {
            return Err("적의 턴이 아닙니다.".into());
        }
        let mut best = vec![];
        let mut best_score = i32::MIN;
        for (i, piece) in self.battle.board.cells().iter().enumerate() {
            if !piece.is_some_and(|p| p.owner == ENEMY) {
                continue;
            }
            for action in self.battle.legal_actions(pos(i)?).map_err(engine_error)? {
                let score = action
                    .steps()
                    .filter_map(|s| {
                        self.battle
                            .board
                            .piece_at(s.take)
                            .ok()
                            .flatten()
                            .filter(|p| p.owner == PLAYER)
                    })
                    .map(|p| {
                        if self.battle.has_ward(p.id) {
                            1
                        } else if p.id == self.battle.king_id {
                            100_000
                        } else {
                            match p.kind {
                                QUEEN | AMAZON => 90,
                                ROOK => 50,
                                BISHOP | KNIGHT => 30,
                                _ => 10,
                            }
                        }
                    })
                    .sum();
                if score > best_score {
                    best.clear();
                    best_score = score;
                }
                if score == best_score {
                    best.push(action);
                }
            }
        }
        if best.is_empty() {
            self.battle.end_turn().map_err(engine_error)?;
            self.round += 1;
            self.cards.begin_turn(&mut self.rng);
            self.log("적에게 합법적인 수가 없어 턴을 넘겼습니다. 전투는 계속됩니다.");
        } else {
            let i = self.rng.index(best.len());
            self.perform_action(best.swap_remove(i))?;
        }
        Ok(())
    }
    pub fn snapshot(&self) -> Snapshot {
        let resting = self.phase == RunPhase::Rest
            || (self.phase == RunPhase::Intermission && ROUTE[self.node].is_none());
        let pieces: Vec<_> = self
            .battle
            .board
            .cells()
            .iter()
            .enumerate()
            .filter_map(|(i, p)| {
                p.filter(|p| !resting || p.owner == PLAYER)
                    .map(|p| PieceView {
                        id: p.id,
                        kind: p.kind,
                        name: piece_name(p.kind),
                        owner: p.owner,
                        cell: i,
                        square: square(pos(i).unwrap()),
                        royal: p.id == self.battle.royal_id || p.id == self.battle.king_id,
                        ward: self.battle.has_ward(p.id),
                    })
            })
            .collect();
        let moves = if self.phase == RunPhase::Combat && self.battle.turn == PLAYER {
            pieces
                .iter()
                .filter(|p| p.owner == PLAYER)
                .flat_map(|p| {
                    self.battle
                        .legal_actions(pos(p.cell).unwrap())
                        .unwrap_or_default()
                        .into_iter()
                        .map(move |a| MoveView {
                            from: p.cell,
                            to: cell(a.steps().last().unwrap().move_to),
                        })
                })
                .collect()
        } else {
            vec![]
        };
        let hand = self
            .cards
            .hand
            .iter()
            .map(|c| {
                let d = c.kind.definition();
                let targets = if d.targeted {
                    pieces
                        .iter()
                        .filter(|p| d.kind != CardKind::Gun || self.gun_front(p.id).is_some())
                        .map(|p| p.id)
                        .collect::<Vec<_>>()
                } else {
                    vec![]
                };
                let disabled_reason =
                    if self.phase != RunPhase::Combat || self.battle.turn != PLAYER {
                        Some("내 턴을 기다리세요.")
                    } else if self.cards.mana < d.cost {
                        Some("마나가 부족합니다.")
                    } else if d.targeted && targets.is_empty() {
                        Some("사용 가능한 진형이 없습니다.")
                    } else {
                        None
                    };
                CardView {
                    id: c.id,
                    definition: d,
                    targets,
                    disabled_reason,
                }
            })
            .collect();
        let roster = self.battle.outcome();
        let standard = Battle::standard_roster();
        let rest_options = roster
            .iter()
            .map(|p| {
                let origin = standard
                    .iter()
                    .find(|s| s.id == p.id)
                    .and_then(|s| s.position);
                RestOption {
                    id: p.id,
                    name: piece_name(p.kind),
                    square: p.position.or(origin).map(square).unwrap_or_default(),
                    can_upgrade: p.position.is_some() && p.kind == PAWN,
                    can_revive: p.position.is_none()
                        && p.kind != KING
                        && !roster
                            .iter()
                            .any(|other| other.position.is_some() && other.position == origin),
                    dead: p.position.is_none(),
                }
            })
            .collect();
        Snapshot {
            phase: self.phase,
            node: self.node,
            encounter: ROUTE[self.node]
                .map(|i| ENCOUNTER_NAMES[i])
                .unwrap_or("작은 쉼터"),
            character: self.character.clone(),
            turn: self.battle.turn,
            round: self.round,
            actions_left: self.battle.actions_left,
            mana: self.cards.mana,
            pieces,
            moves,
            hand,
            draw_count: self.cards.draw.len(),
            discard_count: self.cards.discard.len(),
            exhausted_count: self.cards.exhausted.len(),
            deck_count: self.deck.len(),
            relics: self
                .relics
                .iter()
                .map(|r| RelicView {
                    name: r.name(),
                    description: r.description(),
                })
                .collect(),
            log: self.log.clone(),
            rewards: [CardKind::Ward, CardKind::Jeans, CardKind::Gun]
                .into_iter()
                .map(|k| k.definition())
                .collect(),
            rest_options,
            royal_name: self
                .battle
                .board
                .cells()
                .iter()
                .flatten()
                .find(|p| p.id == self.battle.royal_id)
                .map(|p| piece_name(p.kind))
                .unwrap_or("포획됨"),
            seed: self.seed,
        }
    }
}
#[derive(Serialize)]
pub struct Snapshot {
    pub phase: RunPhase,
    pub node: usize,
    pub encounter: &'static str,
    pub character: String,
    pub turn: u16,
    pub round: u32,
    pub actions_left: u8,
    pub mana: u8,
    pub pieces: Vec<PieceView>,
    pub moves: Vec<MoveView>,
    pub hand: Vec<CardView>,
    pub draw_count: usize,
    pub discard_count: usize,
    pub exhausted_count: usize,
    pub deck_count: usize,
    pub relics: Vec<RelicView>,
    pub log: Vec<String>,
    pub rewards: Vec<CardDefinition>,
    pub rest_options: Vec<RestOption>,
    pub royal_name: &'static str,
    pub seed: u32,
}
#[derive(Serialize)]
pub struct PieceView {
    pub id: u32,
    pub kind: u16,
    pub name: &'static str,
    pub owner: u16,
    pub cell: usize,
    pub square: String,
    pub royal: bool,
    pub ward: bool,
}
#[derive(Serialize)]
pub struct MoveView {
    pub from: usize,
    pub to: usize,
}
#[derive(Serialize)]
pub struct CardView {
    pub id: u32,
    pub definition: CardDefinition,
    pub targets: Vec<u32>,
    pub disabled_reason: Option<&'static str>,
}
#[derive(Serialize)]
pub struct RelicView {
    pub name: &'static str,
    pub description: &'static str,
}
#[derive(Serialize)]
pub struct RestOption {
    pub id: u32,
    pub name: &'static str,
    pub square: String,
    pub can_upgrade: bool,
    pub can_revive: bool,
    pub dead: bool,
}
