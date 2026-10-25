use crate::{cards::*, content::*, map::*, *};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};

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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reward {
    Card(CardKind),
    Relic(Relic),
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
    PlayCardSelection {
        id: u32,
        #[serde(default)]
        targets: Vec<u32>,
        #[serde(default)]
        upgrades: Vec<CardUpgradeChoice>,
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
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CardUpgradeChoice {
    pub id: PieceId,
    pub target: PieceType,
}
#[derive(Clone)]
pub struct Game {
    pub battle: Battle,
    pub cards: CardCombat,
    pub deck: Vec<Card>,
    /// Acquisition order, including the starter. Existing relics are never reordered.
    pub relics: Vec<Relic>,
    pub phase: RunPhase,
    pub map: ActMap,
    pub current_node: Option<usize>,
    pub path: Vec<usize>,
    pub completed: Vec<usize>,
    pub seed: u32,
    pub floor: u8,
    pub character: String,
    pub log: Vec<String>,
    pub round: u32,
    rng: Rng,
    next_card_id: u32,
    rewards: Vec<Reward>,
    /// Positions only, captured before boundary cleanup even when Hard Disk is not owned yet.
    last_battle_positions: BTreeMap<PieceId, Position>,
    hard_disk_deployed: bool,
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
            floor: 1,
            character: choice.name.into(),
            log: vec![],
            round: 1,
            rng: Rng::new(seed),
            next_card_id,
            rewards: vec![],
            last_battle_positions: BTreeMap::new(),
            hard_disk_deployed: false,
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
    fn latest_formation(&self) -> Option<Relic> {
        self.relics
            .iter()
            .rev()
            .copied()
            .find(|r| r.formation().is_some())
    }
    fn starting_roster(&self) -> Result<(Vec<PlayerPiece>, bool), String> {
        let mut roster = self.battle.outcome();
        let mut occupied = HashSet::new();
        let saved: Vec<_> = roster
            .iter()
            .filter(|p| p.position.is_some())
            .filter_map(|p| {
                self.last_battle_positions
                    .get(&p.id)
                    .map(|&pos| (p.id, pos))
            })
            .collect();
        let hard_disk = self.relics.contains(&Relic::HardDisk)
            && !saved.is_empty()
            && saved.iter().all(|(_, pos)| {
                (0..8).contains(&pos.file) && (0..8).contains(&pos.rank) && occupied.insert(*pos)
            });
        occupied.clear();
        let formation = self.latest_formation();
        let mut unplaced = vec![];
        for (index, p) in roster.iter_mut().enumerate() {
            if p.position.is_none() {
                continue; // A snapshot or a preset must never revive a dead ID.
            }
            let desired = if hard_disk {
                self.last_battle_positions.get(&p.id).copied()
            } else {
                formation.and_then(|r| r.position(p.id))
            };
            p.moved = false;
            if let Some(pos) = desired.filter(|pos| occupied.insert(*pos)) {
                p.position = Some(pos);
            } else {
                unplaced.push(index);
            }
        }
        // Rest-revived/new IDs missing from the snapshot use their permanent home.
        // Saved survivors have priority; a blocked home uses the first free square
        // in a1..h1, a2..h2, ... order. No survivor is displaced or removed.
        for index in unplaced {
            let p = &mut roster[index];
            let pos = if !occupied.contains(&p.origin) {
                p.origin
            } else {
                (0..8)
                    .rev()
                    .flat_map(|rank| (0..8).map(move |file| Position::new(file, rank)))
                    .find(|pos| !occupied.contains(pos))
                    .ok_or("아군을 배치할 빈 칸이 없습니다.")?
            };
            occupied.insert(pos);
            p.position = Some(pos);
        }
        Ok((roster, hard_disk))
    }
    fn revive_for_next_floor(&mut self) -> Result<usize, String> {
        let mut roster = self.battle.outcome();
        let mut occupied: HashSet<_> = roster.iter().filter_map(|p| p.position).collect();
        let mut revived = 0;
        for piece in roster.iter_mut().filter(|p| p.position.is_none()) {
            let position = if !occupied.contains(&piece.origin) {
                piece.origin
            } else {
                (0..8)
                    .rev()
                    .flat_map(|rank| (0..8).map(move |file| Position::new(file, rank)))
                    .find(|position| !occupied.contains(position))
                    .ok_or("아군을 부활시킬 빈 칸이 없습니다.")?
            };
            occupied.insert(position);
            piece.position = Some(position);
            piece.moved = false;
            revived += 1;
        }
        if revived > 0 {
            self.battle
                .replace_roster_at_rest(roster)
                .map_err(engine_error)?;
        }
        Ok(revived)
    }
    fn start_battle(&mut self) -> Result<(), String> {
        let index = self
            .current_node
            .and_then(|id| self.map.nodes[id].encounter)
            .ok_or("전투 노드가 아닙니다.")?;
        let mut encounter = encounter(index);
        content::encounters::choose_random_royal(index, &mut encounter, &mut self.rng);
        self.log(format!("{} · 전투 시작", ENCOUNTERS[index].name));
        if self.relics.contains(&Relic::Whirlwind) {
            let mut positions: Vec<_> = encounter.enemies.iter().map(|e| e.position).collect();
            self.rng.shuffle(&mut positions);
            for (enemy, position) in encounter.enemies.iter_mut().zip(positions) {
                enemy.position = position;
            }
            self.log("소용돌이: 적의 시작 위치를 섞었습니다.");
        }
        let (players, hard_disk) = self.starting_roster()?;
        if hard_disk {
            self.log("하드디스크: 직전 전투의 생존 아군 위치를 복원했습니다.");
        } else if let Some(relic) = self.latest_formation() {
            self.log(format!("{}: 시작 진형을 적용했습니다.", relic.name()));
        }
        let haste = self.relics.contains(&Relic::Haste);
        let options = BattleOptions {
            actions_per_turn: if haste { HASTE_ACTIONS } else { 1 },
            self_defense: self.relics.contains(&Relic::SelfDefense),
            initiative: self.relics.contains(&Relic::Initiative),
            enemy_initiative: index == content::encounters::TWO_EMPEROR,
            hard_disk,
        };
        self.battle =
            Battle::with_options(BattleStart { players, encounter }, options, &mut self.rng)
                .map_err(engine_error)?;
        if options.initiative && options.enemy_initiative {
            self.log("선공권: 양측 효과가 서로 상쇄되었습니다.");
        }
        self.hard_disk_deployed = hard_disk;
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
        self.grant_relic(relic);
    }
    fn add_card(&mut self, kind: CardKind) {
        self.deck.push(Card {
            id: self.next_card_id,
            kind,
        });
        self.next_card_id += 1;
    }
    fn grant_relic(&mut self, relic: Relic) {
        if self.relics.contains(&relic) {
            self.add_card(CardKind::Ward);
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
    fn generate_rewards(&mut self) {
        let mut cards = content::cards::REWARD_POOL.to_vec();
        let mut relics: Vec<_> = RELIC_REWARD_POOL
            .iter()
            .copied()
            .filter(|r| !self.relics.contains(r))
            .collect();
        let mut rewards = vec![Reward::Card(cards.swap_remove(self.rng.index(cards.len())))];
        if !relics.is_empty() {
            rewards.push(Reward::Relic(
                relics.swap_remove(self.rng.index(relics.len())),
            ));
        }
        let mut remaining: Vec<_> = cards
            .into_iter()
            .map(Reward::Card)
            .chain(relics.into_iter().map(Reward::Relic))
            .collect();
        // Registered card kinds fill every slot when relics run out.
        while rewards.len() < 3 {
            rewards.push(remaining.swap_remove(self.rng.index(remaining.len())));
        }
        self.rng.shuffle(&mut rewards);
        self.rewards = rewards;
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
            Command::PlayCard { id, target } => {
                self.play_card(id, content::cards::CardSelection::single(target))?
            }
            Command::PlayCardSelection {
                id,
                targets,
                upgrades,
            } => self.play_card(
                id,
                content::cards::CardSelection {
                    target: None,
                    targets,
                    upgrades: upgrades.into_iter().map(|u| (u.id, u.target)).collect(),
                },
            )?,
            Command::EndTurn => {
                self.require_player()?;
                self.battle.end_turn().map_err(engine_error)?;
                self.events();
                self.check_finish()?;
                if self.phase == RunPhase::Combat {
                    self.cards.end_turn();
                    self.log("플레이어가 턴을 마쳤습니다.");
                }
            }
            Command::EnemyStep => self.enemy_step()?,
            Command::ChooseReward { index } => {
                if self.phase != RunPhase::Reward {
                    return Err("지금은 보상을 선택할 수 없습니다.".into());
                }
                let reward = *self
                    .rewards
                    .get(index)
                    .ok_or("보상 선택이 잘못되었습니다.")?;
                match reward {
                    Reward::Card(kind) => {
                        self.add_card(kind);
                        self.log(format!(
                            "보상: {}을 덱에 추가했습니다.",
                            kind.definition().name
                        ));
                    }
                    Reward::Relic(relic) => self.grant_relic(relic),
                }
                if self
                    .current_node
                    .is_some_and(|id| self.map.nodes[id].kind == NodeKind::Elite)
                {
                    self.grant_node_relic();
                }
                self.rewards.clear();
                self.complete_node();
                if self.map.nodes[self.current_node.unwrap()].kind == NodeKind::Boss {
                    if self.floor == 1 {
                        let revived = self.revive_for_next_floor()?;
                        self.floor = 2;
                        self.map = ActMap::generate_for_floor(self.seed, 2);
                        self.current_node = None;
                        self.path.clear();
                        self.completed.clear();
                        self.log(format!("2층에 도착했습니다. 아군 전원이 회복되었습니다. 부활한 기물 {revived}기."));
                    } else {
                        self.phase = RunPhase::Victory;
                    }
                }
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
                let index = roster
                    .iter()
                    .position(|p| p.id == id && p.position.is_some())
                    .ok_or("살아 있는 아군 기물을 선택하세요.")?;
                let previous_kind = roster[index].kind;
                let upgrade = content::pieces::definition(previous_kind)
                    .and_then(|d| d.upgrades.iter().find(|u| u.target == target))
                    .ok_or("이 기물에 정의된 강화 경로가 없습니다.")?;
                let missing: Vec<_> = content::pieces::required_owned(target)
                    .iter()
                    .filter(|&&kind| !roster.iter().any(|p| p.kind == kind))
                    .map(|&kind| piece_name(kind))
                    .collect();
                if !missing.is_empty() {
                    return Err(format!(
                        "영구 로스터에 필요한 기물이 없습니다: {}",
                        missing.join(", ")
                    ));
                }
                let p = &mut roster[index];
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
                self.begin_player_turn();
            }
        }
        Ok(())
    }
    fn check_finish(&mut self) -> Result<(), String> {
        if self.phase != RunPhase::Combat {
            return Ok(());
        }
        if let GameResult::Win { winner, reason } = self.battle.result {
            self.last_battle_positions = self
                .battle
                .player_pieces()
                .iter()
                .filter_map(|p| p.position.map(|pos| (p.id, pos)))
                .collect();
            let keep_positions = self.relics.contains(&Relic::HardDisk);
            self.battle
                .reset_after_battle(keep_positions)
                .map_err(engine_error)?;
            self.cards = CardCombat::default();
            if winner == PLAYER {
                self.phase = RunPhase::Reward;
                self.generate_rewards();
                self.log(match reason {
                    ResultReason::SelfDefense => "승리! 호신용품으로 적 로얄이 사망했습니다.",
                    ResultReason::HardDisk => {
                        "승리! 하드디스크의 시작 배치와 겹친 적 로얄이 사망했습니다."
                    }
                    _ => "승리! 적 로얄을 직접 포획했습니다.",
                });
                self.log(if keep_positions {
                    "하드디스크: 생존 기물의 종료 배치를 유지합니다."
                } else {
                    "생존 기물이 시작 칸으로 복귀합니다."
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
    fn begin_player_turn(&mut self) {
        self.round = self.round.saturating_add(1);
        self.cards
            .begin_turn(&mut self.rng, self.relics.contains(&Relic::IceCream));
    }
    fn play_card(
        &mut self,
        id: u32,
        selection: content::cards::CardSelection,
    ) -> Result<(), String> {
        self.require_player()?;
        let index = self
            .cards
            .hand
            .iter()
            .position(|c| c.id == id)
            .ok_or("손패에 없는 카드입니다.")?;
        let d = self.cards.hand[index].kind.definition();
        self.resolve_selection(index, &selection, d)
    }
    #[cfg(test)]
    fn resolve_card(
        &mut self,
        index: usize,
        target: Option<u32>,
        d: CardDefinition,
    ) -> Result<(), String> {
        self.resolve_selection(index, &content::cards::CardSelection::single(target), d)
    }
    fn resolve_selection(
        &mut self,
        index: usize,
        selection: &content::cards::CardSelection,
        d: CardDefinition,
    ) -> Result<(), String> {
        use content::cards::SelectionMode;
        match d.selection_mode {
            SelectionMode::None
                if selection.target.is_some()
                    || !selection.targets.is_empty()
                    || !selection.upgrades.is_empty() =>
            {
                return Err("이 카드는 대상을 선택하지 않습니다.".into())
            }
            SelectionMode::Single
                if !selection.targets.is_empty() || !selection.upgrades.is_empty() =>
            {
                return Err("기물 하나만 선택하세요.".into())
            }
            SelectionMode::Multiple
                if selection.target.is_some() || !selection.upgrades.is_empty() =>
            {
                return Err("복수 기물 선택이 올바르지 않습니다.".into())
            }
            SelectionMode::Upgrade
                if selection.target.is_some() || !selection.targets.is_empty() =>
            {
                return Err("강화 경로 선택이 올바르지 않습니다.".into())
            }
            _ => {}
        }
        let cost = d.selected_cost(selection)?;
        if self.cards.mana < cost {
            return Err("마나가 부족합니다.".into());
        }
        let mut next = self.battle.clone();
        let removed = if let Some(effect) = d.selection_effect {
            effect(&mut next, selection)?
        } else {
            (d.effect)(&mut next, selection.target)?
        };
        self.battle = next;
        self.cards.spend_with_cost(index, &d, cost);
        self.log(format!(
            "{} 사용 · 마나 −{}{}",
            d.name,
            cost,
            removed
                .map(|n| format!(" · {n}기 제거 (로얄 보호)"))
                .unwrap_or_default()
        ));
        self.events();
        self.check_finish()?;
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
                let mut score = action
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
                if let Some(key) = action
                    .steps()
                    .last()
                    .and_then(|step| step.transition.as_deref())
                {
                    if let Some(promoted) = content::pieces::ALL.iter().find(|d| d.key == key) {
                        score += promoted.value
                            - content::pieces::definition(piece.unwrap().kind)
                                .map_or(10, |d| d.value);
                    }
                }
                if self
                    .battle
                    .limiter_status(piece.unwrap().id)
                    .is_some_and(|(active, left)| active && left > 0)
                {
                    score += 10_000;
                }
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
            self.events();
            self.check_finish()?;
            if self.phase == RunPhase::Combat {
                self.begin_player_turn();
                self.log("적에게 합법적인 수가 없어 턴을 넘겼습니다. 전투는 계속됩니다.");
            }
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
                        royal: self.battle.is_enemy_royal(p.id) || p.id == self.battle.king_id,
                        ward: self.battle.has_ward(p.id),
                        self_defense: self.battle.has_self_defense(p.id),
                        temporary_upgrade: self.battle.has_temporary_upgrade(p.id),
                        limiter: self
                            .battle
                            .limiter_status(p.id)
                            .map(|(active, moves_left)| LimiterView { active, moves_left }),
                        absorbed_moves: self
                            .battle
                            .absorbed_kinds(p.id)
                            .into_iter()
                            .map(piece_name)
                            .collect(),
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
                    } else if self.cards.mana < if d.cost_per_target { 1 } else { d.cost } {
                        Some("마나가 부족합니다.")
                    } else if d.targeted && targets.is_empty() {
                        Some(if c.kind == CardKind::SacredDomain {
                            "희생할 아군 비숍이 없습니다."
                        } else {
                            "선택 가능한 기물이 없습니다."
                        })
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
        let next_battle_pieces = if self.phase == RunPhase::Map {
            self.starting_roster()
                .map(|(players, _)| {
                    players
                        .into_iter()
                        .filter_map(|p| {
                            p.position.map(|position| {
                                let definition = content::pieces::definition(p.kind).unwrap();
                                PieceView {
                                    id: p.id,
                                    kind: p.kind,
                                    name: definition.name,
                                    key: definition.key,
                                    description: definition.description,
                                    owner: PLAYER,
                                    cell: cell(position),
                                    square: square(position),
                                    royal: p.id == self.battle.king_id,
                                    ward: false,
                                    self_defense: false,
                                    temporary_upgrade: false,
                                    limiter: None,
                                    absorbed_moves: vec![],
                                }
                            })
                        })
                        .collect()
                })
                .unwrap_or_default()
        } else {
            vec![]
        };
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
                        && content::pieces::definition(p.kind)
                            .unwrap()
                            .upgrades
                            .iter()
                            .any(|u| {
                                content::pieces::required_owned(u.target)
                                    .iter()
                                    .all(|&kind| roster.iter().any(|owned| owned.kind == kind))
                            }),
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
                    id: r,
                    status: None,
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
            mana_rule: if self.relics.contains(&Relic::IceCream) {
                "아이스크림: 내 턴 시작마다 남은 코스트에 3을 더합니다."
            } else {
                "내 턴 시작 시 코스트가 3으로 회복됩니다."
            },
            initiative_turns_left: self.battle.initiative_turns_left(),
            enemy_initiative_turns_left: self.battle.enemy_initiative_turns_left(),
            hard_disk_deployed: self.phase == RunPhase::Combat && self.hard_disk_deployed,
            pieces,
            moves,
            hand,
            draw_count: self.cards.draw.len(),
            discard_count: self.cards.discard.len(),
            exhausted_count: self.cards.exhausted.len(),
            deck_count: self.deck.len(),
            deck: self
                .deck
                .iter()
                .map(|card| card.kind.definition())
                .collect(),
            next_battle_pieces,
            relics: self
                .relics
                .iter()
                .map(|r| RelicView {
                    id: *r,
                    status: if *r == Relic::Initiative {
                        Some(if self.phase != RunPhase::Combat {
                            "다음 전투 시작 시 보호가 활성화됩니다.".into()
                        } else if self.battle.initiative_turns_left() > 0 {
                            format!(
                                "보호 중 · 남은 적 턴 {}회",
                                self.battle.initiative_turns_left()
                            )
                        } else {
                            "보호 종료 · 다음 전투에 다시 적용됩니다.".into()
                        })
                    } else if r.formation().is_some() {
                        Some(if Some(*r) == self.latest_formation() {
                            "선택된 시작 진형 · 하드디스크 기록이 있으면 기록 우선".into()
                        } else {
                            "비활성 · 더 최근에 획득한 시작 진형이 있습니다.".into()
                        })
                    } else if *r == Relic::HardDisk {
                        Some(
                            if self.phase == RunPhase::Combat && self.hard_disk_deployed {
                                "직전 전투 종료 배치를 이번 전투 시작에 복원했습니다.".into()
                            } else if self.last_battle_positions.is_empty() {
                                "아직 복원할 이전 전투 배치가 없습니다.".into()
                            } else {
                                "직전 전투 종료 배치를 다음 전투에 복원합니다.".into()
                            },
                        )
                    } else if *r == Relic::IceCream && self.phase == RunPhase::Combat {
                        Some(format!("이번 전투 누적 코스트 잔액 {}", self.cards.mana))
                    } else {
                        None
                    },
                    name: r.name(),
                    description: r.description(),
                })
                .collect(),
            log: self.log.clone(),
            rewards: self
                .rewards
                .iter()
                .map(|r| match r {
                    Reward::Card(kind) => RewardView::Card(kind.definition()),
                    Reward::Relic(relic) => RewardView::Relic(RelicView {
                        id: *relic,
                        status: None,
                        name: relic.name(),
                        description: relic.description(),
                    }),
                })
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
                        required_pieces: content::pieces::required_owned(u.target)
                            .iter()
                            .map(|&kind| RequiredPieceView {
                                kind,
                                name: piece_name(kind),
                                owned: roster.iter().any(|p| p.kind == kind),
                            })
                            .collect(),
                        unlocked: content::pieces::required_owned(u.target)
                            .iter()
                            .all(|&kind| roster.iter().any(|p| p.kind == kind)),
                    })
                })
                .collect(),
            royal_name: self
                .battle
                .board
                .cells()
                .iter()
                .flatten()
                .find(|p| p.owner == ENEMY && self.battle.is_enemy_royal(p.id))
                .map(|p| piece_name(p.kind))
                .unwrap_or("포획됨"),
            seed: self.seed,
            floor: self.floor,
        }
    }
    pub fn inspect_enemy(&self, index: usize) -> Result<EnemyInspectionView, String> {
        if self.phase != RunPhase::Combat {
            return Err("전투 중에만 적 기물을 조회할 수 있습니다.".into());
        }
        let from = pos(index)?;
        let piece = self
            .battle
            .board
            .piece_at(from)
            .map_err(|e| engine_error(e.into()))?;
        if !piece.is_some_and(|p| p.owner == ENEMY) {
            return Err("해당 칸에 적 기물이 없습니다.".into());
        }
        let moves = self
            .battle
            .preview_actions(from)
            .map_err(engine_error)?
            .into_iter()
            .map(|preview| {
                let route = preview
                    .route
                    .filter(|points| {
                        points.len() > 2
                            && (points.windows(2).any(|pair| {
                                (pair[1].file - pair[0].file).abs() > 1
                                    || (pair[1].rank - pair[0].rank).abs() > 1
                            }) || points.windows(3).any(|triple| {
                                let (ax, ay) = (
                                    triple[1].file - triple[0].file,
                                    triple[1].rank - triple[0].rank,
                                );
                                let (bx, by) = (
                                    triple[2].file - triple[1].file,
                                    triple[2].rank - triple[1].rank,
                                );
                                ax * by != ay * bx
                            }))
                    })
                    .map(|points| points.into_iter().map(cell).collect());
                EnemyMoveView {
                    to: cell(preview.destination),
                    captures: preview.captures.into_iter().map(cell).collect(),
                    route,
                }
            })
            .collect();
        Ok(EnemyInspectionView {
            piece_id: piece.unwrap().id,
            cell: index,
            moves,
        })
    }
}
#[derive(Serialize)]
pub struct EnemyInspectionView {
    pub piece_id: PieceId,
    pub cell: usize,
    pub moves: Vec<EnemyMoveView>,
}
#[derive(Serialize)]
pub struct EnemyMoveView {
    pub to: usize,
    pub captures: Vec<usize>,
    pub route: Option<Vec<usize>>,
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
    pub mana: u32,
    pub mana_rule: &'static str,
    pub initiative_turns_left: u8,
    pub enemy_initiative_turns_left: u8,
    pub hard_disk_deployed: bool,
    pub pieces: Vec<PieceView>,
    pub moves: Vec<MoveView>,
    pub hand: Vec<CardView>,
    pub draw_count: usize,
    pub discard_count: usize,
    pub exhausted_count: usize,
    pub deck_count: usize,
    pub deck: Vec<CardDefinition>,
    pub next_battle_pieces: Vec<PieceView>,
    pub relics: Vec<RelicView>,
    pub log: Vec<String>,
    pub rewards: Vec<RewardView>,
    pub rest_options: Vec<RestOption>,
    pub upgrade_paths: Vec<UpgradeView>,
    pub royal_name: &'static str,
    pub seed: u32,
    pub floor: u8,
}
#[derive(Serialize)]
#[serde(tag = "type", content = "definition", rename_all = "snake_case")]
pub enum RewardView {
    Card(CardDefinition),
    Relic(RelicView),
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
    pub temporary_upgrade: bool,
    pub limiter: Option<LimiterView>,
    pub absorbed_moves: Vec<&'static str>,
}
#[derive(Serialize)]
pub struct LimiterView {
    pub active: bool,
    pub moves_left: u8,
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
    pub id: Relic,
    pub name: &'static str,
    pub description: &'static str,
    pub status: Option<String>,
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
    pub required_pieces: Vec<RequiredPieceView>,
    pub unlocked: bool,
}

#[derive(Serialize)]
pub struct RequiredPieceView {
    pub kind: PieceType,
    pub name: &'static str,
    pub owned: bool,
}

#[cfg(test)]
mod relic_resource_tests {
    use super::*;

    #[test]
    fn six_cost_card_uses_the_same_resolution_and_payment_path() {
        let mut game = Game::new("vanguard", 2026).unwrap();
        game.dispatch(Command::SelectNode {
            id: game.available_nodes()[0],
        })
        .unwrap();
        game.cards.hand.push(Card {
            id: 1,
            kind: CardKind::Ward,
        });
        let mut definition = CardKind::Ward.definition();
        definition.cost = 6;
        game.cards.mana = 5;
        assert!(game.resolve_card(0, Some(5), definition.clone()).is_err());
        assert!(!game.battle.has_ward(5));
        assert_eq!(game.cards.hand.len(), 1);
        game.cards.mana = 6;
        game.resolve_card(0, Some(5), definition).unwrap();
        assert!(game.battle.has_ward(5));
        assert_eq!(game.cards.mana, 0);
        assert!(game.cards.hand.is_empty());
    }
}
