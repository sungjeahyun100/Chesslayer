use crate::{cards::*, content::*, map::*, *};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RunPhase {
    Map,
    Combat,
    Reward,
    Treasure,
    Rest,
    Victory,
    Defeat,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Command {
    Move {
        from: usize,
        to: usize,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        promotion: Option<PieceType>,
    },
    PlayCard {
        id: u32,
        target: Option<u32>,
    },
    EndTurn,
    EnemyStep,
    ChooseReward {
        index: usize,
    },
    SelectNode {
        id: usize,
    },
    ClaimTreasure,
    RestUpgrade {
        id: u32,
        target: PieceType,
    },
    RestRevive {
        id: u32,
    },
    RestSkip,
}
#[derive(Clone)]
pub struct Game {
    pub battle: Battle,
    pub cards: CardCombat,
    pub deck: Vec<Card>,
    pub relics: Vec<Relic>,
    pub phase: RunPhase,
    pub map: ActMap,
    pub current_node: Option<usize>,
    pub path: Vec<usize>,
    pub completed: Vec<usize>,
    pub seed: u32,
    pub character: String,
    pub log: Vec<String>,
    pub round: u32,
    rng: Rng,
    next_card_id: u32,
}
pub(crate) fn engine_error(e: Error) -> String {
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
        let next_card_id = deck.iter().map(|card| card.id).max().unwrap_or(0) + 1;
        let battle = Battle::new(BattleStart {
            players: Battle::standard_roster(),
            encounter: encounter(0),
        })
        .map_err(engine_error)?;
        let game = Self {
            battle,
            cards: CardCombat::default(),
            deck,
            relics: vec![choice.relic],
            phase: RunPhase::Map,
            map: ActMap::generate(seed),
            current_node: None,
            path: vec![],
            completed: vec![],
            seed,
            character: choice.name.into(),
            log: vec![],
            round: 1,
            rng: Rng::new(seed),
            next_card_id,
        };
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
        let index = self
            .current_node
            .and_then(|id| self.map.nodes[id].encounter)
            .ok_or("전투 노드가 아닙니다.")?;
        let mut encounter = encounter(index);
        self.log(format!("{} · 전투 시작", ENCOUNTERS[index].name));
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
            &mut self.rng,
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
        self.check_finish()?;
        Ok(())
    }
    pub fn available_nodes(&self) -> Vec<usize> {
        if self.phase != RunPhase::Map {
            return vec![];
        }
        match self.current_node {
            None => self.map.starts(),
            Some(id) if self.completed.contains(&id) => self.map.nodes[id].next.clone(),
            Some(_) => vec![],
        }
    }
    fn complete_node(&mut self) {
        let id = self.current_node.expect("content requires a selected node");
        self.completed.push(id);
        self.phase = RunPhase::Map;
    }
    fn grant_node_relic(&mut self) {
        let relic = self.map.nodes[self.current_node.unwrap()]
            .relic
            .expect("reward node has relic");
        if self.relics.contains(&relic) {
            self.deck.push(Card {
                id: self.next_card_id,
                kind: CardKind::Ward,
            });
            self.next_card_id += 1;
            self.log(format!(
                "{}은 이미 보유 중입니다. 대신 가호 카드 1장을 얻었습니다.",
                relic.name()
            ));
        } else {
            self.relics.push(relic);
            self.log(format!(
                "유물 획득: {} · {}",
                relic.name(),
                relic.description()
            ));
        }
    }
    fn map_view(&self) -> MapView {
        let available = self.available_nodes();
        MapView {
            rows: self.map.rows,
            lanes: MAP_LANES,
            current: self.current_node,
            path: self.path.clone(),
            nodes: self
                .map
                .nodes
                .iter()
                .map(|n| NodeView {
                    id: n.id,
                    row: n.row,
                    lane: n.lane,
                    kind: n.kind,
                    name: n.name(),
                    description: n.description(),
                    next: n.next.clone(),
                    available: available.contains(&n.id),
                    visited: self.completed.contains(&n.id),
                    current: self.current_node == Some(n.id),
                    skipped: self
                        .current_node
                        .is_some_and(|id| n.row <= self.map.nodes[id].row)
                        && !self.path.contains(&n.id),
                })
                .collect(),
        }
    }
    fn require_player(&self) -> Result<(), String> {
        if self.phase != RunPhase::Combat || self.battle.turn != PLAYER {
            return Err("플레이어의 전투 턴에만 사용할 수 있습니다.".into());
        }
        Ok(())
    }
    pub fn dispatch(&mut self, command: Command) -> Result<(), String> {
        match command {
            Command::Move {
                from,
                to,
                promotion,
            } => {
                self.require_player()?;
                let from = pos(from)?;
                let to = pos(to)?;
                let promotion_key = promotion
                    .map(|kind| {
                        content::pieces::definition(kind)
                            .map(|d| d.key)
                            .ok_or("등록되지 않은 승격 대상입니다.")
                    })
                    .transpose()?;
                let action = self
                    .battle
                    .legal_actions(from)
                    .map_err(engine_error)?
                    .into_iter()
                    .find(|a| {
                        a.steps().last().is_some_and(|s| {
                            s.move_to == to && s.transition.as_deref() == promotion_key
                        })
                    })
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
                let kind = *content::cards::REWARD_POOL
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
                if self
                    .current_node
                    .is_some_and(|id| self.map.nodes[id].kind == NodeKind::Elite)
                {
                    self.grant_node_relic();
                }
                self.complete_node();
            }
            Command::SelectNode { id } => {
                if !self.available_nodes().contains(&id) {
                    return Err("현재 위치에서 연결된 다음 노드만 선택할 수 있습니다.".into());
                }
                self.current_node = Some(id);
                self.path.push(id);
                match self.map.nodes[id].kind {
                    NodeKind::Combat | NodeKind::Elite | NodeKind::Boss => self.start_battle()?,
                    NodeKind::Rest => {
                        self.phase = RunPhase::Rest;
                        self.log("작은 쉼터 · 강화 또는 부활을 한 번 선택하세요.");
                    }
                    NodeKind::Treasure => {
                        self.phase = RunPhase::Treasure;
                        self.log("남겨진 보물 · 새로운 힘을 발견했습니다.");
                    }
                }
            }
            Command::ClaimTreasure => {
                if self.phase != RunPhase::Treasure {
                    return Err("지금은 보물을 받을 수 없습니다.".into());
                }
                self.grant_node_relic();
                self.complete_node();
            }
            Command::RestUpgrade { id, target } => {
                if self.phase != RunPhase::Rest {
                    return Err("쉼터에서만 강화할 수 있습니다.".into());
                }
                let mut roster = self.battle.outcome();
                let p = roster
                    .iter_mut()
                    .find(|p| p.id == id && p.position.is_some())
                    .ok_or("살아 있는 아군 기물을 선택하세요.")?;
                let upgrade = content::pieces::definition(p.kind)
                    .and_then(|d| d.upgrades.iter().find(|u| u.target == target))
                    .ok_or("이 기물에 정의된 강화 경로가 없습니다.")?;
                let previous_kind = p.kind;
                p.kind = upgrade.target;
                p.upgrades.push(upgrade.key.into());
                self.battle
                    .replace_roster_at_rest(roster)
                    .map_err(engine_error)?;
                self.log(format!(
                    "휴식: {} → {} 강화",
                    piece_name(previous_kind),
                    piece_name(target)
                ));
                self.complete_node();
            }
            Command::RestRevive { id } => {
                if self.phase != RunPhase::Rest {
                    return Err("쉼터에서만 부활할 수 있습니다.".into());
                }
                let mut roster = self.battle.outcome();
                let origin = roster
                    .iter()
                    .find(|p| p.id == id)
                    .map(|p| p.origin)
                    .ok_or("잘못된 기물입니다.")?;
                if roster.iter().any(|p| p.position == Some(origin)) {
                    return Err("원래 초기 위치가 점유되어 부활할 수 없습니다.".into());
                }
                let p = roster
                    .iter_mut()
                    .find(|p| p.id == id && p.position.is_none() && p.kind != KING)
                    .ok_or("사망한 아군 기물을 선택하세요.")?;
                p.position = Some(origin);
                p.moved = false;
                self.battle
                    .replace_roster_at_rest(roster)
                    .map_err(engine_error)?;
                self.log(format!("휴식: {}에서 아군이 부활했습니다.", square(origin)));
                self.complete_node();
            }
            Command::RestSkip => {
                if self.phase != RunPhase::Rest {
                    return Err("휴식 중이 아닙니다.".into());
                }
                self.complete_node();
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
        if !still_at {
            if let Some(key) = action.steps().last().and_then(|s| s.transition.as_deref()) {
                if let Some(d) = content::pieces::ALL.iter().find(|d| d.key == key) {
                    self.log(format!("승격: {} → {}", piece_name(piece.kind), d.name));
                }
            }
        }
        self.events();
        self.check_finish()?;
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
    fn check_finish(&mut self) -> Result<(), String> {
        if let GameResult::Win { winner, reason } = self.battle.result {
            self.battle.reset_after_battle().map_err(engine_error)?;
            self.cards = CardCombat::default();
            if winner == PLAYER {
                self.phase = if self
                    .current_node
                    .is_some_and(|id| self.map.nodes[id].kind == NodeKind::Boss)
                {
                    RunPhase::Victory
                } else {
                    RunPhase::Reward
                };
                if self.phase == RunPhase::Victory {
                    self.completed.push(self.current_node.unwrap());
                }
                self.log(if reason == ResultReason::SelfDefense {
                    "승리! 호신용품으로 적 로얄이 사망했습니다. 생존 기물이 시작 칸으로 복귀합니다."
                } else {
                    "승리! 적 로얄을 직접 포획했습니다. 생존 기물이 시작 칸으로 복귀합니다."
                });
            } else {
                self.phase = RunPhase::Defeat;
                self.log(if reason == ResultReason::Counterattack {
                    "역습으로 킹을 잃었습니다. 원정이 끝났습니다."
                } else {
                    "킹이 포획되었습니다. 원정이 끝났습니다."
                });
            }
        }
        Ok(())
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
        let removed = (d.effect)(&mut self.battle, target)?;
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
                            content::pieces::definition(p.kind).map_or(10, |d| d.value)
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
        let resting = matches!(
            self.phase,
            RunPhase::Rest | RunPhase::Map | RunPhase::Treasure
        );
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
                        key: content::pieces::definition(p.kind).unwrap().key,
                        description: content::pieces::definition(p.kind).unwrap().description,
                        owner: p.owner,
                        cell: i,
                        square: square(pos(i).unwrap()),
                        royal: p.id == self.battle.royal_id || p.id == self.battle.king_id,
                        ward: self.battle.has_ward(p.id),
                        self_defense: self.battle.has_self_defense(p.id),
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
                            promotion: a.steps().last().unwrap().transition.as_ref().and_then(
                                |key| {
                                    content::pieces::ALL.iter().find(|d| d.key == key).map(|d| {
                                        PromotionView {
                                            kind: d.kind,
                                            key: d.key,
                                            name: d.name,
                                            description: d.description,
                                        }
                                    })
                                },
                            ),
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
                    (d.target_candidates)(&self.battle)
                        .into_iter()
                        .filter(|id| pieces.iter().any(|p| p.id == *id))
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
        let rest_options = roster
            .iter()
            .map(|p| {
                let origin = Some(p.origin);
                RestOption {
                    id: p.id,
                    kind: p.kind,
                    key: content::pieces::definition(p.kind).unwrap().key,
                    name: piece_name(p.kind),
                    description: content::pieces::definition(p.kind).unwrap().description,
                    cell: p.position.map(cell),
                    origin: origin.map(cell),
                    upgrades: p.upgrades.clone(),
                    ward: self.battle.has_ward(p.id),
                    square: p.position.or(origin).map(square).unwrap_or_default(),
                    can_upgrade: self.phase == RunPhase::Rest
                        && p.position.is_some()
                        && !content::pieces::definition(p.kind)
                            .unwrap()
                            .upgrades
                            .is_empty(),
                    can_revive: self.phase == RunPhase::Rest
                        && origin.is_some()
                        && p.position.is_none()
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
            map: self.map_view(),
            encounter: self
                .current_node
                .map(|id| self.map.nodes[id].name())
                .unwrap_or("갈림길의 입구"),
            node_kind: self.current_node.map(|id| self.map.nodes[id].kind),
            node_reward: self
                .current_node
                .and_then(|id| self.map.nodes[id].relic)
                .map(|r| RelicView {
                    name: r.name(),
                    description: if self.relics.contains(&r) {
                        "이미 가진 유물입니다. 대신 가호 카드 1장을 얻습니다."
                    } else {
                        r.description()
                    },
                }),
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
            rewards: content::cards::REWARD_POOL
                .iter()
                .map(|k| k.definition())
                .collect(),
            rest_options,
            upgrade_paths: content::pieces::ALL
                .iter()
                .flat_map(|d| {
                    d.upgrades.iter().map(|u| UpgradeView {
                        from: d.kind,
                        from_key: d.key,
                        from_name: d.name,
                        to: u.target,
                        to_key: content::pieces::definition(u.target).unwrap().key,
                        to_name: piece_name(u.target),
                        effect: u.effect,
                        requirement: u.requirement,
                    })
                })
                .collect(),
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
    pub map: MapView,
    pub node_kind: Option<NodeKind>,
    pub node_reward: Option<RelicView>,
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
    pub upgrade_paths: Vec<UpgradeView>,
    pub royal_name: &'static str,
    pub seed: u32,
}
#[derive(Serialize)]
pub struct PieceView {
    pub id: u32,
    pub kind: u16,
    pub name: &'static str,
    pub key: &'static str,
    pub description: &'static str,
    pub owner: u16,
    pub cell: usize,
    pub square: String,
    pub royal: bool,
    pub ward: bool,
    pub self_defense: bool,
}
#[derive(Serialize)]
pub struct MoveView {
    pub from: usize,
    pub to: usize,
    pub promotion: Option<PromotionView>,
}
#[derive(Serialize)]
pub struct PromotionView {
    pub kind: PieceType,
    pub key: &'static str,
    pub name: &'static str,
    pub description: &'static str,
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
    pub kind: PieceType,
    pub key: &'static str,
    pub description: &'static str,
    pub cell: Option<usize>,
    pub origin: Option<usize>,
    pub upgrades: Vec<String>,
    pub ward: bool,
    pub name: &'static str,
    pub square: String,
    pub can_upgrade: bool,
    pub can_revive: bool,
    pub dead: bool,
}

#[derive(Serialize)]
pub struct UpgradeView {
    pub from: PieceType,
    pub from_key: &'static str,
    pub from_name: &'static str,
    pub to: PieceType,
    pub to_key: &'static str,
    pub to_name: &'static str,
    pub effect: &'static str,
    pub requirement: &'static str,
}
