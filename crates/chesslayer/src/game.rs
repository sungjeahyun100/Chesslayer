use crate::{
    ai::{self, Budget},
    battle::GeneratedAction,
    cards::*,
    content::*,
    map::*,
    *,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

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
    Event,
    Shop,
}
/// Campaign unlocks are independent of the disposable run and are saved by the host.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Progress {
    /// Legacy save metadata. Neither field gates floor progression.
    pub highest_floor: u8,
    pub unlocked_squads: Vec<String>,
    pub final_unlocked: bool,
    pub final_defeated: bool,
}
impl Default for Progress {
    fn default() -> Self {
        Self {
            highest_floor: 1,
            unlocked_squads: vec!["guardian".into()],
            final_unlocked: false,
            final_defeated: false,
        }
    }
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
    SpecialAction {
        from: usize,
        to: usize,
    },
    ResolveEffect {
        target: u32,
    },
    RestConfess {
        id: u32,
    },
    RestSupply {
        id: u32,
    },
    Reinforce {
        id: u32,
    },
    ProtectHunger {
        id: u32,
    },
    EventChoice {
        choice: String,
    },
    ShopBuy {
        index: usize,
    },
    ShopLeave,
    EndTurn,
    EnemyStep,
    EnemyStepSelected {
        from: Option<usize>,
        index: Option<usize>,
    },
    ChooseReward {
        index: usize,
    },
    ClaimReward {
        index: usize,
    },
    ClaimUpgrade {
        id: u32,
        target: PieceType,
    },
    ContinueReward,
    AdvanceFloor,
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
    pub squad_id: String,
    pub progress: Progress,
    pub final_battle: bool,
    pub gold: u32,
    pub ammo: Option<u32>,
    reinforcement_used: bool,
    hunger_protection_used: bool,
    shop_offers: Vec<CardKind>,
    shop_bought: Vec<usize>,
    pub log: Vec<String>,
    pub round: u32,
    rng: Rng,
    next_card_id: u32,
    rewards: Vec<Reward>,
    reward_node: Option<usize>,
    reward_choice: Option<usize>,
    upgrade_claimed: bool,
    last_cleared_node: Option<usize>,
    boss_plan: Option<BossPlan>,
    boss_sequence_step: u8,
}
#[derive(Clone)]
struct BossPlan {
    action: DefaultAction,
    name: String,
    area: Vec<usize>,
    targets: Vec<BossTargetView>,
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
        let deck = content::cards::starting_deck(character);
        let next_card_id = deck.iter().map(|card| card.id).max().unwrap_or(0) + 1;
        let battle = Battle::new(BattleStart {
            players: content::characters::roster(character),
            encounter: encounter(0),
        })
        .map_err(engine_error)?;
        let mut game = Self {
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
            squad_id: character.into(),
            progress: Progress::default(),
            final_battle: false,
            gold: 0,
            ammo: (character == "artillery").then_some(400),
            reinforcement_used: false,
            hunger_protection_used: false,
            shop_offers: vec![],
            shop_bought: vec![],
            log: vec![],
            round: 1,
            rng: Rng::new(seed),
            next_card_id,
            rewards: vec![],
            reward_node: None,
            reward_choice: None,
            upgrade_claimed: false,
            last_cleared_node: None,
            boss_plan: None,
            boss_sequence_step: 0,
        };
        game.assign_map_relics();
        Ok(game)
    }
    pub fn new_run(
        character: &str,
        seed: u32,
        floor: u8,
        progress: Progress,
    ) -> Result<Self, String> {
        if !progress.unlocked_squads.iter().any(|id| id == character) {
            return Err("아직 해금되지 않은 부대입니다.".into());
        }
        if floor > 4 {
            return Err("존재하지 않는 층입니다.".into());
        }
        let mut game = Self::new(character, seed)?;
        game.progress = progress;
        game.final_battle = floor == 0;
        game.floor = if floor == 0 { 4 } else { floor };
        game.map = if game.final_battle {
            ActMap {
                rows: 1,
                nodes: vec![MapNode {
                    id: 0,
                    row: 0,
                    lane: 2,
                    kind: NodeKind::Boss,
                    encounter: Some(content::encounters::DICTATOR_BOSS),
                    relic: None,
                    next: vec![],
                }],
            }
        } else {
            ActMap::generate_for_floor(seed, floor)
        };
        game.assign_map_relics();
        Ok(game)
    }
    fn assign_map_relics(&mut self) {
        let pool = content::relics::reward_pool(&self.squad_id);
        let mut rng = Rng::new(self.seed ^ 0x0300_00ab);
        for node in &mut self.map.nodes {
            if node.relic.is_some() {
                node.relic = Some(pool[rng.index(pool.len())]);
            }
        }
    }
    fn ammo_capacity(&self) -> Option<u32> {
        self.ammo.map(|_| {
            self.battle
                .outcome()
                .iter()
                .filter(|p| p.kind == content::pieces::WAREHOUSE && p.position.is_some())
                .count() as u32
                * 200
        })
    }
    fn resource_effects(&mut self) {
        let (draw, mana, ammo) = self.battle.take_resource_effects();
        self.cards.draw_cards(draw as usize, &mut self.rng);
        if let Some(current) = &mut self.ammo {
            *current = current.saturating_add_signed(ammo);
            self.cards.mana = *current;
        } else {
            self.cards.mana = self.cards.mana.saturating_add_signed(mana);
        }
        self.battle.set_ammunition(self.ammo.unwrap_or(0));
    }
    fn log(&mut self, message: impl Into<String>) {
        self.log.push(message.into());
        if self.log.len() > 60 {
            self.log.remove(0);
        }
    }
    fn events(&mut self) {
        self.resource_effects();
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
    fn starting_roster(&self) -> Result<Vec<PlayerPiece>, String> {
        let mut roster = self.battle.outcome();
        let mut occupied = HashSet::new();
        let formation = self.latest_formation();
        let mut unplaced = vec![];
        for (index, p) in roster.iter_mut().enumerate() {
            if p.position.is_none() {
                continue; // A preset must never revive a dead ID.
            }
            let desired = formation.and_then(|r| r.position(p.id));
            p.moved = false;
            if let Some(pos) = desired.filter(|pos| occupied.insert(*pos)) {
                p.position = Some(pos);
            } else {
                unplaced.push(index);
            }
        }
        // IDs missing from a preset use their permanent home.
        // Preset positions have priority; a blocked home uses the first free square
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
        Ok(roster)
    }
    fn start_battle(&mut self) -> Result<(), String> {
        let index = self
            .current_node
            .and_then(|id| self.map.nodes[id].encounter)
            .ok_or("전투 노드가 아닙니다.")?;
        let mut encounter = encounter(index);
        content::encounters::choose_random_royal(index, &mut encounter, &mut self.rng);
        self.log(format!("{} · 전투 시작", ENCOUNTERS[index].name));
        let mut players = self.starting_roster()?;
        for piece in &mut players {
            piece.persistent.remove("hunger_worsened");
        }
        if let Some(relic) = self.latest_formation() {
            self.log(format!("{}: 시작 진형을 적용했습니다.", relic.name()));
        }
        let haste = self.relics.contains(&Relic::Haste);
        let options = BattleOptions {
            actions_per_turn: if haste { HASTE_ACTIONS } else { 1 },
            self_defense: self.relics.contains(&Relic::SelfDefense),
            initiative: self.relics.contains(&Relic::Initiative),
            enemy_initiative: index == content::encounters::TWO_EMPEROR,
        };
        self.battle =
            Battle::with_options(BattleStart { players, encounter }, options, &mut self.rng)
                .map_err(engine_error)?;
        if options.initiative && options.enemy_initiative {
            self.log("선공권: 양측 효과가 서로 상쇄되었습니다.");
        }
        if haste {
            self.log(format!("민첩성: 턴마다 최대 {HASTE_ACTIONS}회 이동합니다."));
        }
        self.phase = RunPhase::Combat;
        self.round = 1;
        self.cards = CardCombat::start(&self.deck, &mut self.rng);
        if let Some(ammo) = self.ammo {
            self.cards.mana = ammo;
        }
        self.battle.configure_v030(&self.relics);
        self.resource_effects();
        self.events();
        self.check_finish()?;
        self.plan_boss();
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
        self.last_cleared_node = Some(id);
        self.phase = RunPhase::Map;
    }
    fn claim_reward(&mut self, index: usize) -> Result<(), String> {
        if self.phase != RunPhase::Reward || self.reward_choice.is_some() {
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
        self.reward_choice = Some(index);
        Ok(())
    }
    fn claim_upgrade(&mut self, id: u32, target: PieceType) -> Result<(), String> {
        if self.phase != RunPhase::Reward
            || !self.current_node.is_some_and(|node| {
                matches!(self.map.nodes[node].kind, NodeKind::Elite | NodeKind::Boss)
            })
            || self.upgrade_claimed
        {
            return Err("지금은 기물 강화 보상을 받을 수 없습니다.".into());
        }
        self.upgrade_piece(id, target, "보상")?;
        self.upgrade_claimed = true;
        Ok(())
    }
    fn upgrade_piece(&mut self, id: u32, target: PieceType, source: &str) -> Result<(), String> {
        let mut roster = self.battle.outcome();
        let index = roster
            .iter()
            .position(|p| p.id == id && p.position.is_some())
            .ok_or("살아 있는 아군 기물을 선택하세요.")?;
        let previous_kind = roster[index].kind;
        if previous_kind == content::pieces::WAREHOUSE
            && target == previous_kind
            && self.ammo.is_some()
        {
            let capacity = self.ammo_capacity().unwrap();
            let current = self.ammo.unwrap();
            self.ammo = Some(current.saturating_add(current / 10).min(capacity));
            self.log(format!(
                "{source}: 창고 강화 · 탄약 {} → {}",
                current,
                self.ammo.unwrap()
            ));
            return Ok(());
        }

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
            "{source}: {} → {} 강화",
            piece_name(previous_kind),
            piece_name(target)
        ));
        Ok(())
    }
    fn continue_reward(&mut self) -> Result<(), String> {
        if self.phase != RunPhase::Reward {
            return Err("지금은 다음으로 진행할 수 없습니다.".into());
        }
        let id = self.current_node.expect("reward requires a selected node");
        if self.completed.contains(&id) {
            self.phase = RunPhase::Map;
            if self.map.nodes[id].kind == NodeKind::Boss {
                self.finish_boss()?;
            }
            return Ok(());
        }
        self.complete_node();
        if self.map.nodes[id].kind == NodeKind::Boss {
            self.finish_boss()?;
        }
        Ok(())
    }
    fn finish_boss(&mut self) -> Result<(), String> {
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
                    .find(|pos| !occupied.contains(pos))
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
            self.log(format!(
                "보스 격파: 아군 기물 {revived}기가 회복되었습니다."
            ));
        }
        if let Some(capacity) = self.ammo_capacity() {
            self.ammo = Some(capacity);
        }
        if self.final_battle {
            self.phase = RunPhase::Victory;
            self.log("부족의 독재자를 쓰러뜨렸습니다. 원정을 완수했습니다.");
            return Ok(());
        }
        self.floor += 1;
        self.final_battle = self.floor == 5;
        self.current_node = None;
        self.path.clear();
        self.completed.clear();
        self.reward_node = None;
        self.reward_choice = None;
        self.last_cleared_node = None;
        self.upgrade_claimed = false;
        self.phase = RunPhase::Map;
        self.map = if self.final_battle {
            self.floor = 4;
            ActMap {
                rows: 1,
                nodes: vec![MapNode {
                    id: 0,
                    row: 0,
                    lane: 2,
                    kind: NodeKind::Boss,
                    encounter: Some(content::encounters::DICTATOR_BOSS),
                    relic: None,
                    next: vec![],
                }],
            }
        } else {
            ActMap::generate_for_floor(
                self.seed ^ u32::from(self.floor).wrapping_mul(0x9e37_79b9),
                self.floor,
            )
        };
        self.assign_map_relics();
        self.log(if self.final_battle {
            "최종 결전으로 향합니다."
        } else {
            "다음 층으로 향합니다."
        });
        Ok(())
    }
    fn unlock_boss(&mut self) {
        if self.final_battle {
            self.progress.final_defeated = true;
        } else {
            let unlocked =
                ["vanguard", "heretic", "artillery", "special"][(self.floor - 1) as usize];
            if !self
                .progress
                .unlocked_squads
                .iter()
                .any(|id| id == unlocked)
            {
                self.progress.unlocked_squads.push(unlocked.into());
            }
        }
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
            self.add_card(content::cards::reward_pool(&self.squad_id)[0]);
            self.log(format!(
                "{}은 이미 보유 중입니다. 대신 부대 전용 카드 1장을 얻었습니다.",
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
        let mut cards = content::cards::reward_pool(&self.squad_id).to_vec();
        if self
            .current_node
            .is_some_and(|id| matches!(self.map.nodes[id].kind, NodeKind::Elite | NodeKind::Boss))
        {
            let mut rewards = Vec::with_capacity(3);
            for _ in 0..3 {
                if cards.is_empty() {
                    cards = content::cards::reward_pool(&self.squad_id).to_vec();
                }
                rewards.push(Reward::Card(cards.swap_remove(self.rng.index(cards.len()))));
            }
            self.rewards = rewards;
            return;
        }
        let mut relics: Vec<_> = content::relics::reward_pool(&self.squad_id)
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
            if remaining.is_empty() {
                remaining.extend(
                    content::cards::reward_pool(&self.squad_id)
                        .iter()
                        .copied()
                        .map(Reward::Card),
                );
            }
            rewards.push(remaining.swap_remove(self.rng.index(remaining.len())));
        }
        self.rng.shuffle(&mut rewards);
        self.rewards = rewards;
    }
    fn map_view(&self) -> MapView {
        let reviewing_reward = self.phase == RunPhase::Reward
            && self.current_node.is_some_and(|id| {
                self.completed.contains(&id)
                    && self.last_cleared_node == Some(id)
                    && self.reward_node == Some(id)
            });
        let available = if reviewing_reward {
            self.map.nodes[self.current_node.unwrap()].next.clone()
        } else {
            self.available_nodes()
        };
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
                    reward_revisit: (self.phase == RunPhase::Map || reviewing_reward)
                        && self.last_cleared_node == Some(n.id)
                        && self.reward_node == Some(n.id),
                    unclaimed_reward: (self.phase == RunPhase::Map || reviewing_reward)
                        && self.last_cleared_node == Some(n.id)
                        && self.reward_node == Some(n.id)
                        && (self.reward_choice.is_none()
                            || (matches!(n.kind, NodeKind::Elite | NodeKind::Boss)
                                && !self.upgrade_claimed)),
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
        if self.battle.pending_choice().is_some() {
            return Err("먼저 발동한 효과의 대상을 선택하세요.".into());
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
                if !self.perform_action(|battle| {
                    Ok(battle.generated_legal_actions(from)?.into_iter().find(|a| {
                        a.action().first.effect != Effect::Catch
                            && a.action().steps().last().is_some_and(|s| {
                                s.move_to == to && s.transition.as_deref() == promotion_key
                            })
                    }))
                })? {
                    return Err("합법적인 이동이 아닙니다.".into());
                }
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
            Command::ResolveEffect { target } => {
                if self.phase != RunPhase::Combat {
                    return Err("지금은 효과 대상을 선택할 수 없습니다.".into());
                }
                let side = self.battle.turn;
                self.battle.resolve_choice(target)?;
                self.events();
                self.check_finish()?;
                if self.phase == RunPhase::Combat && self.battle.turn != side {
                    if side == PLAYER {
                        self.cards.end_turn();
                    } else {
                        self.begin_player_turn();
                    }
                }
            }
            Command::SpecialAction { from, to } => {
                self.require_player()?;
                let from = pos(from)?;
                let to = pos(to)?;
                if from == to
                    && self
                        .battle
                        .board
                        .piece_at(from)
                        .ok()
                        .flatten()
                        .is_some_and(|p| {
                            matches!(
                                p.kind,
                                content::pieces::DRONE
                                    | content::pieces::C4_DRONE
                                    | content::pieces::MACHINE_GUN_DRONE
                                    | content::pieces::TACTICAL_NUKE_DRONE
                                    | content::pieces::GRENADE_LAUNCHER_DRONE
                            )
                        })
                {
                    self.battle.use_drone_ability(from).map_err(engine_error)?;
                    self.events();
                    self.check_finish()?;
                    if self.phase == RunPhase::Combat && self.battle.turn != PLAYER {
                        self.cards.end_turn();
                    }
                    return Ok(());
                }
                if !self.perform_action(|battle| {
                    Ok(battle.generated_legal_actions(from)?.into_iter().find(|a| {
                        a.action().first.effect == Effect::Catch && a.action().first.take == to
                    }))
                })? {
                    return Err("사용할 수 없는 특수 능력 대상입니다.".into());
                }
            }
            Command::RestConfess { id } => {
                if self.phase != RunPhase::Rest {
                    return Err("쉼터에서만 고해성사할 수 있습니다.".into());
                }
                let mut roster = self.battle.outcome();
                let p = roster
                    .iter_mut()
                    .find(|p| {
                        p.id == id && p.persistent.get("incapacitated").copied().unwrap_or(0) > 0
                    })
                    .ok_or("영구 전투불능인 기물을 선택하세요.")?;
                p.persistent.remove("incapacitated");
                self.battle
                    .replace_roster_at_rest(roster)
                    .map_err(engine_error)?;
                self.complete_node();
                self.log("고해성사: 영구 전투불능이 해제되었습니다.");
            }
            Command::RestSupply { id } => {
                if self.phase != RunPhase::Rest {
                    return Err("쉼터에서만 창고를 강화합니다.".into());
                }
                self.upgrade_piece(id, content::pieces::WAREHOUSE, "휴식")?;
                self.complete_node();
            }
            Command::Reinforce { id } => {
                if self.phase != RunPhase::Reward
                    || self.reinforcement_used
                    || !self.relics.contains(&Relic::Reinforcement)
                {
                    return Err("지금은 충원할 수 없습니다.".into());
                }
                let mut roster = self.battle.outcome();
                let baseline = content::characters::roster(&self.squad_id);
                let base = baseline
                    .iter()
                    .find(|p| p.id == id)
                    .ok_or("초기 부대 기물만 충원할 수 있습니다.")?;
                if roster.iter().any(|p| p.position == Some(base.origin)) {
                    return Err("초기 칸이 점유되어 있습니다.".into());
                }
                let p = roster
                    .iter_mut()
                    .find(|p| p.id == id && p.position.is_none())
                    .ok_or("사망한 기물을 선택하세요.")?;
                let permanent_incapacitated = p.persistent.get("incapacitated").copied();
                *p = base.clone();
                if let Some(value) = permanent_incapacitated {
                    p.persistent.insert("incapacitated".into(), value);
                }
                self.battle
                    .replace_roster_at_rest(roster)
                    .map_err(engine_error)?;
                self.reinforcement_used = true;
                self.log("충원: 선택한 기물이 초기 단계로 돌아왔습니다.");
            }
            Command::ProtectHunger { id } => {
                if self.phase != RunPhase::Reward
                    || self.hunger_protection_used
                    || !self.relics.contains(&Relic::LastPlate)
                {
                    return Err("지금은 마지막 접시를 사용할 수 없습니다.".into());
                }
                let mut roster = self.battle.outcome();
                let p = roster
                    .iter_mut()
                    .find(|p| {
                        p.id == id && p.persistent.get("hunger_worsened").copied().unwrap_or(0) > 0
                    })
                    .ok_or("허기가 악화된 아군을 선택하세요.")?;
                if let Some(hunger) = p.persistent.get_mut("hunger") {
                    *hunger = (*hunger - 1).max(0);
                }
                p.persistent.remove("hunger_worsened");
                self.battle
                    .replace_roster_at_rest(roster)
                    .map_err(engine_error)?;
                self.hunger_protection_used = true;
                self.log("마지막 접시: 선택한 아군의 허기 악화를 방지했습니다.");
            }
            Command::EventChoice { choice } => {
                if self.phase != RunPhase::Event {
                    return Err("이벤트 중이 아닙니다.".into());
                }
                match choice.as_str() {
                    "eat" => {
                        let mut roster = self.battle.outcome();
                        for p in &mut roster {
                            p.persistent.remove("hunger");
                            p.persistent.remove("hunger_worsened");
                        }
                        self.battle
                            .replace_roster_at_rest(roster)
                            .map_err(engine_error)?;
                        self.log("식당: 모든 아군이 식사를 마치고 허기를 해소했습니다.");
                    }
                    "leave" => self.log("식당을 떠났습니다."),
                    _ => return Err("올바른 이벤트 선택이 아닙니다.".into()),
                }
                self.complete_node();
            }
            Command::ShopBuy { index } => {
                if self.phase != RunPhase::Shop || self.shop_bought.contains(&index) {
                    return Err("구입할 수 없는 상품입니다.".into());
                }
                let kind = *self.shop_offers.get(index).ok_or("상품이 없습니다.")?;
                if self.gold < 20 {
                    return Err("금화가 부족합니다.".into());
                }
                self.gold -= 20;
                self.shop_bought.push(index);
                self.add_card(kind);
                self.log(format!("상점: {} 구입 · 금화 20", kind.definition().name));
            }
            Command::ShopLeave => {
                if self.phase != RunPhase::Shop {
                    return Err("상점에 있지 않습니다.".into());
                }
                self.complete_node();
            }
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
            Command::EnemyStepSelected { from, index } => self.enemy_step_selected(from, index)?,
            Command::ChooseReward { index } => {
                // Retain the original one-step command for saved command streams and callers.
                self.claim_reward(index)?;
                self.continue_reward()?;
                if self.phase == RunPhase::Map
                    && self.current_node.is_some_and(|id| {
                        self.map.nodes[id].kind == NodeKind::Boss && self.completed.contains(&id)
                    })
                {
                    self.finish_boss()?;
                }
            }
            Command::ClaimReward { index } => self.claim_reward(index)?,
            Command::ClaimUpgrade { id, target } => self.claim_upgrade(id, target)?,
            Command::ContinueReward => self.continue_reward()?,
            Command::AdvanceFloor => {
                // Older saved command streams could explicitly finish a boss map.
                if self.phase == RunPhase::Map && self.current_node.is_none() {
                    return Ok(());
                }
                let id = self.current_node.ok_or("완료한 보스 노드가 없습니다.")?;
                if self.phase != RunPhase::Map
                    || self.map.nodes[id].kind != NodeKind::Boss
                    || !self.completed.contains(&id)
                {
                    return Err("지금은 다음 층으로 진행할 수 없습니다.".into());
                }
                self.finish_boss()?;
            }
            Command::SelectNode { id } => {
                if self.phase == RunPhase::Map
                    && self.last_cleared_node == Some(id)
                    && self.reward_node == Some(id)
                {
                    self.phase = RunPhase::Reward;
                    return Ok(());
                }
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
                    NodeKind::Event => {
                        self.phase = RunPhase::Event;
                        self.log("작은 식당에서 따뜻한 음식을 내놓았습니다.");
                    }
                    NodeKind::Shop => {
                        self.phase = RunPhase::Shop;
                        self.shop_offers = content::cards::reward_pool(&self.squad_id).to_vec();
                        self.rng.shuffle(&mut self.shop_offers);
                        self.shop_offers.truncate(3);
                        self.shop_bought.clear();
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
                self.upgrade_piece(id, target, "휴식")?;
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
                    .find(|p| {
                        p.id == id
                            && p.position.is_none()
                            && (self.ammo.is_some() || p.id != self.battle.king_id)
                    })
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
    fn perform_action<F>(&mut self, select: F) -> Result<bool, String>
    where
        F: for<'a> FnOnce(&'a Battle) -> Result<Option<GeneratedAction<'a>>, Error>,
    {
        let side = self.battle.turn;
        let Some((action, piece, captured)) =
            self.battle.select_and_apply(select).map_err(engine_error)?
        else {
            return Ok(false);
        };
        let destination = action.steps().last().unwrap().move_to;
        let still_at = self
            .battle
            .board
            .piece_at(action.first.from)
            .ok()
            .flatten()
            .is_some_and(|p| p.id == piece.id);
        let (dx, dy) = (
            (destination.file - action.first.from.file).abs(),
            (destination.rank - action.first.from.rank).abs(),
        );
        let rule = if side != ENEMY {
            ""
        } else if piece.kind == content::pieces::UNICORN && !matches!((dx, dy), (1, 2) | (2, 1)) {
            " · 규칙: 빈칸 순간이동"
        } else if piece.kind == content::pieces::EMPEROR && (dx > 1 || dy > 1) {
            " · 규칙: 포획한 기물의 행마 흡수"
        } else if !action.continuation.is_empty() {
            " · 규칙: 특수 경로 행마"
        } else if still_at
            && action.first.effect == Effect::Catch
            && content::pieces::definition(piece.kind)
                .is_some_and(|d| d.description.contains("사격"))
        {
            " · 규칙: 제자리 사격 (명중·빗나감은 아래 기록)"
        } else if piece.kind == content::pieces::TEMPEST_PAWN && dy == 2 {
            " · 규칙: 템페스트 폰의 전방 두 칸 포획"
        } else {
            ""
        };
        self.log(format!(
            "{} {} · {} → {}{}{}",
            if side == PLAYER { "백" } else { "흑" },
            piece_name(piece.kind),
            square(action.first.from),
            square(destination),
            if still_at {
                " (제자리 판정)".to_string()
            } else {
                captured
                    .map(|p| format!(" · {} 포획", piece_name(p.kind)))
                    .unwrap_or_default()
            },
            rule
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
        Ok(true)
    }
    fn check_finish(&mut self) -> Result<(), String> {
        if self.phase != RunPhase::Combat {
            return Ok(());
        }
        if let GameResult::Win { winner, reason } = self.battle.result {
            self.resource_effects();
            self.battle.reset_after_battle().map_err(engine_error)?;
            self.cards = CardCombat::default();
            if winner == PLAYER {
                self.phase = RunPhase::Reward;
                self.gold += if self
                    .current_node
                    .is_some_and(|id| self.map.nodes[id].kind == NodeKind::Combat)
                {
                    15
                } else {
                    30
                };
                self.reinforcement_used = false;
                self.hunger_protection_used = false;
                if self
                    .current_node
                    .is_some_and(|id| self.map.nodes[id].kind == NodeKind::Boss)
                {
                    self.unlock_boss();
                }
                self.generate_rewards();
                self.reward_node = self.current_node;
                self.reward_choice = None;
                self.upgrade_claimed = false;
                self.log(match reason {
                    ResultReason::SelfDefense => "승리! 호신용품으로 적 로얄이 사망했습니다.",
                    _ => "승리! 적 로얄을 직접 포획했습니다.",
                });
                self.log("생존 기물이 시작 칸으로 복귀합니다.");
            } else {
                self.phase = RunPhase::Defeat;
                self.log(match reason {
                    ResultReason::Counterattack => "역습으로 킹을 잃었습니다. 원정이 끝났습니다.",
                    _ => "킹이 포획되었습니다. 원정이 끝났습니다.",
                });
            }
        }
        Ok(())
    }
    fn begin_player_turn(&mut self) {
        self.round = self.round.saturating_add(1);
        self.cards
            .begin_turn(&mut self.rng, self.relics.contains(&Relic::IceCream));
        if let Some(ammo) = self.ammo {
            self.cards.mana = ammo;
        }
        self.plan_boss();
    }
    fn plan_boss(&mut self) {
        self.boss_plan = None;
        self.boss_sequence_step = 0;
        if self.phase != RunPhase::Combat || self.battle.boss().is_none() {
            return;
        }
        let mut ranked = self
            .battle
            .boss_action_candidates()
            .into_iter()
            .map(|action| {
                let from = action.first.from;
                let to = action.steps().last().unwrap().move_to;
                let tag = action.first.transition.as_deref();
                let copied = tag
                    .and_then(|tag| tag.strip_prefix("boss_copy:"))
                    .and_then(|tag| tag.split_once(':'))
                    .and_then(|(kind, _)| kind.parse::<PieceType>().ok());
                let (mut area, mut target_positions) = self.battle.boss_shot_area(&action);
                if let Some(kind) = copied {
                    if action.first.effect == Effect::Catch && action.first.take == from {
                        area = self.battle.blast_preview(kind, from);
                        target_positions = area
                            .iter()
                            .copied()
                            .filter(|&at| self.battle.board.piece_at(at).ok().flatten().is_some())
                            .collect();
                    } else if action.first.effect == Effect::Catch && to == from {
                        let (blast, paths) =
                            self.battle
                                .shot_effect_preview(kind, from, action.first.take);
                        area.push(action.first.take);
                        area.extend(blast);
                        area.extend(paths.into_iter().flatten());
                        target_positions.push(action.first.take);
                    } else {
                        area.push(to);
                        if self
                            .battle
                            .board
                            .piece_at(action.first.take)
                            .ok()
                            .flatten()
                            .is_some()
                        {
                            target_positions.push(action.first.take);
                        }
                    }
                } else if tag != Some("boss_shot") && tag != Some("boss_reload") {
                    area.push(to);
                    if self
                        .battle
                        .board
                        .piece_at(action.first.take)
                        .ok()
                        .flatten()
                        .is_some()
                    {
                        target_positions.push(action.first.take);
                    }
                }
                let targets: Vec<_> = target_positions
                    .iter()
                    .filter_map(|&at| {
                        self.battle
                            .board
                            .piece_at(at)
                            .ok()
                            .flatten()
                            .map(|p| BossTargetView {
                                cell: cell(at),
                                chance: copied
                                    .and_then(|kind| {
                                        self.battle.boss_copied_hit_chance(kind, from, at)
                                    })
                                    .or_else(|| self.battle.hit_chance(from, at))
                                    .unwrap_or(100),
                                name: piece_name(p.kind).into(),
                            })
                    })
                    .collect();
                let score = if tag == Some("boss_reload") {
                    80
                } else if tag == Some("boss_shot") {
                    if targets.is_empty() {
                        -20
                    } else {
                        20 + targets
                            .iter()
                            .map(|t| {
                                let owner = self
                                    .battle
                                    .board
                                    .piece_at(pos(t.cell).unwrap())
                                    .ok()
                                    .flatten()
                                    .map(|p| p.owner);
                                if owner == Some(PLAYER) {
                                    i32::from(t.chance)
                                } else {
                                    8
                                }
                            })
                            .sum::<i32>()
                    }
                } else if copied.is_some() && action.first.effect == Effect::Catch {
                    30 + targets
                        .iter()
                        .filter(|t| {
                            self.battle
                                .board
                                .piece_at(pos(t.cell).unwrap())
                                .ok()
                                .flatten()
                                .is_some_and(|p| p.owner == PLAYER)
                        })
                        .count() as i32
                        * 45
                } else if self
                    .battle
                    .board
                    .piece_at(action.first.take)
                    .ok()
                    .flatten()
                    .is_some_and(|p| p.owner == PLAYER)
                {
                    125
                } else {
                    let nearest = self
                        .battle
                        .board
                        .cells()
                        .iter()
                        .enumerate()
                        .filter_map(|(i, p)| {
                            p.filter(|p| p.owner == PLAYER).map(|_| pos(i).unwrap())
                        })
                        .map(|p| (p.file - to.file).abs().max((p.rank - to.rank).abs()))
                        .min()
                        .unwrap_or(8);
                    16 - nearest * 2
                };
                let name = match tag {
                    Some("boss_reload") => "샷건 재장전 · 행동 1회".into(),
                    Some("boss_shot") => format!(
                        "3갈래 샷건 · {} 방향 · 탄도별 첫 기물 공격",
                        square(action.first.take)
                    ),
                    _ if copied.is_some() && action.first.effect == Effect::Catch => format!(
                        "습득 능력 · {} · {}",
                        piece_name(copied.unwrap()),
                        if action.first.take == from {
                            "자폭".into()
                        } else {
                            square(action.first.take)
                        }
                    ),
                    _ if copied.is_some() => format!(
                        "습득 행마 · {} · {}",
                        piece_name(copied.unwrap()),
                        square(to)
                    ),
                    _ if self
                        .battle
                        .board
                        .piece_at(action.first.take)
                        .ok()
                        .flatten()
                        .is_some_and(|p| p.owner == PLAYER) =>
                    {
                        format!("포획 · {}", square(action.first.take))
                    }
                    _ => format!("이동 · {}", square(to)),
                };
                (
                    score,
                    BossPlan {
                        action,
                        name,
                        area: area.into_iter().map(cell).collect(),
                        targets,
                    },
                )
            })
            .collect::<Vec<_>>();
        ranked.sort_by_key(|a| std::cmp::Reverse(a.0));
        self.boss_plan = ranked.into_iter().next().map(|(_, plan)| plan);
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
        let (cost, removed) = crate::cards::resolve_selection(
            &mut self.battle,
            &mut self.cards,
            index,
            selection,
            d.clone(),
        )?;
        if self.ammo.is_some() {
            self.ammo = Some(self.cards.mana);
        }
        self.log(format!(
            "{} 사용 · {} −{}{}",
            d.name,
            if self.ammo.is_some() {
                "탄약"
            } else {
                "마나"
            },
            cost,
            removed
                .map(|n| format!(" · {n}기 처리"))
                .unwrap_or_default()
        ));
        self.events();
        self.check_finish()?;
        Ok(())
    }
    pub fn enemy_recommendation(&self) -> Result<Option<(usize, usize)>, String> {
        if self.phase != RunPhase::Combat || self.battle.turn != ENEMY {
            return Err("적의 턴이 아닙니다.".into());
        }
        if self.battle.boss().is_some() {
            let plan = (self.boss_sequence_step == 0)
                .then_some(self.boss_plan.as_ref())
                .flatten();
            return Ok(plan.and_then(|plan| {
                let from = plan.action.first.from;
                self.battle
                    .legal_actions(from)
                    .ok()?
                    .iter()
                    .position(|action| action == &plan.action)
                    .map(|index| (cell(from), index))
            }));
        }
        let budget = match self.current_node.map(|id| self.map.nodes[id].kind) {
            Some(NodeKind::Boss) => Budget::boss(),
            Some(NodeKind::Elite) => Budget::elite(),
            _ => Budget::normal(),
        };
        let choice = ai::choose(&self.battle, budget).map_err(engine_error)?;
        let Some(choice) = choice else {
            return Ok(None);
        };
        let actions = self
            .battle
            .legal_actions(choice.first.from)
            .map_err(engine_error)?;
        let index = actions
            .iter()
            .position(|a| a == &choice)
            .ok_or("AI가 선택한 행동이 현재 전투에서 유효하지 않습니다.")?;
        Ok(Some((cell(choice.first.from), index)))
    }
    fn enemy_step(&mut self) -> Result<(), String> {
        let selected = self.enemy_recommendation()?;
        self.enemy_step_selected(selected.map(|s| s.0), selected.map(|s| s.1))
    }
    fn enemy_step_selected(
        &mut self,
        from: Option<usize>,
        index: Option<usize>,
    ) -> Result<(), String> {
        if self.phase != RunPhase::Combat
            || self.battle.turn != ENEMY
            || from.is_some() != index.is_some()
        {
            return Err("적의 턴이 아닙니다.".into());
        }
        let boss_fight = self.battle.boss().is_some();
        let planned = (self.boss_sequence_step == 0)
            .then_some(self.boss_plan.as_ref())
            .flatten()
            .map(|p| p.action.clone());
        let acted = self.perform_action(|battle| {
            let (Some(from), Some(index)) = (from, index) else {
                if boss_fight {
                    return Ok(None);
                }
                for (i, piece) in battle.board.cells().iter().enumerate() {
                    if piece.is_some_and(|p| p.owner == ENEMY)
                        && !battle
                            .legal_actions(pos(i).map_err(|_| Error::IllegalAction)?)?
                            .is_empty()
                    {
                        return Err(Error::IllegalAction);
                    }
                }
                return Ok(None);
            };
            let from = pos(from).map_err(|_| Error::IllegalAction)?;
            if boss_fight {
                let Some(plan) = planned.as_ref() else {
                    return Ok(None);
                };
                if plan.first.from != from {
                    return Err(Error::IllegalAction);
                }
                return Ok(battle
                    .generated_legal_actions(from)
                    .unwrap_or_default()
                    .into_iter()
                    .find(|candidate| candidate.action() == plan));
            }
            battle
                .generated_legal_actions(from)?
                .into_iter()
                .nth(index)
                .map(Some)
                .ok_or(Error::IllegalAction)
        })?;
        if !acted {
            if boss_fight {
                self.battle.forfeit_boss_action().map_err(engine_error)?;
                self.boss_sequence_step = self.boss_sequence_step.saturating_add(1);
            } else {
                self.battle.end_turn().map_err(engine_error)?;
            }
            self.events();
            self.check_finish()?;
            if boss_fight && self.phase == RunPhase::Combat {
                self.log("예고된 적 행동이 불가능해져 실패했습니다.");
            }
            if self.phase == RunPhase::Combat && self.battle.turn == PLAYER {
                self.begin_player_turn();
                if !boss_fight {
                    self.log("적에게 합법적인 수가 없어 턴을 넘겼습니다. 전투는 계속됩니다.");
                }
            }
        } else if boss_fight && self.phase == RunPhase::Combat && self.battle.turn == ENEMY {
            self.boss_sequence_step = self.boss_sequence_step.saturating_add(1);
        }
        if boss_fight
            && self.phase == RunPhase::Combat
            && self.battle.turn == ENEMY
            && self.boss_sequence_step >= 1
            && self.battle.pending_choice().is_none()
        {
            self.battle.end_turn().map_err(engine_error)?;
            self.events();
            self.check_finish()?;
            if self.phase == RunPhase::Combat {
                self.begin_player_turn();
            }
        }
        Ok(())
    }
    pub fn snapshot(&self) -> Snapshot {
        let resting = matches!(
            self.phase,
            RunPhase::Rest | RunPhase::Map | RunPhase::Treasure | RunPhase::Event | RunPhase::Shop
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
                        royal: self.battle.is_royal_piece(p.id),
                        ward: self.battle.has_ward(p.id),
                        self_defense: self.battle.has_self_defense(p.id),
                        temporary_upgrade: self.battle.has_temporary_upgrade(p.id),
                        limiter: self
                            .battle
                            .limiter_status(p.id)
                            .map(|(active, moves_left)| LimiterView { active, moves_left }),
                        statuses: self.battle.status_labels(p.id),
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
                        .filter(|a| a.first.effect != Effect::Catch)
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
                        .filter(|id| {
                            (*id >= 10_000 && *id < 10_064) || pieces.iter().any(|p| p.id == *id)
                        })
                        .collect::<Vec<_>>()
                } else {
                    vec![]
                };
                let disabled_reason = if self.battle.pending_choice().is_some() {
                    Some("발동 효과의 대상을 먼저 선택하세요.")
                } else if self.phase != RunPhase::Combat || self.battle.turn != PLAYER {
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
        let next_battle_pieces = if !matches!(self.phase, RunPhase::Victory | RunPhase::Defeat) {
            self.starting_roster()
                .map(|players| {
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
                                    royal: self.battle.is_royal_piece(p.id),
                                    ward: false,
                                    self_defense: false,
                                    temporary_upgrade: false,
                                    limiter: None,
                                    absorbed_moves: vec![],
                                    statuses: self.battle.status_labels(p.id),
                                }
                            })
                        })
                        .collect()
                })
                .unwrap_or_default()
        } else {
            vec![]
        };
        let rest_options: Vec<RestOption> = roster
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
                    can_upgrade: (self.phase == RunPhase::Rest
                        || (self.phase == RunPhase::Reward
                            && self.current_node.is_some_and(|id| {
                                matches!(self.map.nodes[id].kind, NodeKind::Elite | NodeKind::Boss)
                            })
                            && !self.upgrade_claimed))
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
                        && (self.ammo.is_some() || p.id != self.battle.king_id)
                        && !roster
                            .iter()
                            .any(|other| other.position.is_some() && other.position == origin),
                    dead: p.position.is_none(),
                    statuses: self.battle.status_labels(p.id),
                    can_confess: self.phase == RunPhase::Rest
                        && p.persistent.get("incapacitated").copied().unwrap_or(0) > 0,
                }
            })
            .collect();
        let reinforcement_options = rest_options
            .iter()
            .filter(|p| {
                self.phase == RunPhase::Reward
                    && !self.reinforcement_used
                    && self.relics.contains(&Relic::Reinforcement)
                    && p.dead
                    && content::characters::roster(&self.squad_id)
                        .iter()
                        .any(|base| base.id == p.id)
                    && !roster
                        .iter()
                        .any(|r| r.position.is_some() && r.position.map(cell) == p.origin)
            })
            .cloned()
            .collect();
        let hunger_protection_options = rest_options
            .iter()
            .filter(|p| {
                self.phase == RunPhase::Reward
                    && !self.hunger_protection_used
                    && self.relics.contains(&Relic::LastPlate)
                    && roster.iter().find(|r| r.id == p.id).is_some_and(|r| {
                        r.persistent.get("hunger_worsened").copied().unwrap_or(0) > 0
                    })
            })
            .cloned()
            .collect();
        let abilities = if self.phase == RunPhase::Combat && self.battle.turn == PLAYER {
            pieces.iter().filter(|p| p.owner == PLAYER).flat_map(|p| {
                let from = pos(p.cell).unwrap();
                let actions = self.battle.legal_actions(from).unwrap_or_default();
                let has_self_destruct = actions.iter().any(|a| a.first.effect == Effect::Catch && a.first.take == from);
                let targets: Vec<_> = actions.into_iter().filter(|a| a.first.effect == Effect::Catch && a.first.take != from).map(|a| {
                    let (blast, paths) = self.battle.shot_effect_preview(p.kind, from, a.first.take);
                    AbilityTarget { cell: cell(a.first.take), chance: self.battle.hit_chance(from, a.first.take), blast: blast.into_iter().map(cell).collect(), paths: paths.into_iter().map(|path| path.into_iter().map(cell).collect()).collect() }
                }).collect();
                let mut views = vec![];
                if !targets.is_empty() { views.push(AbilityView { piece_id: p.id, kind: "special", name: match p.kind { content::pieces::MACHINE_GUN_DRONE => "기관총 사격", content::pieces::GRENADE_LAUNCHER_DRONE => "유탄 발사", _ => "특수 능력" }, description: match p.kind { content::pieces::MACHINE_GUN_DRONE => "직교 4칸 이내 적 한 기물을 사격합니다. 탄약 1과 행동 1회를 소모합니다.", content::pieces::GRENADE_LAUNCHER_DRONE => "직교 4칸 이내 적을 사격합니다. 명중하면 탄착 칸과 직교 인접 칸을 폭발시킵니다. 탄약 1과 행동 1회를 소모합니다.", _ => "대상을 선택해 사격 또는 제물 능력을 사용합니다. 기물 행동 1회를 소모합니다." }, targets }); }
                if has_self_destruct && matches!(p.kind, content::pieces::DRONE | content::pieces::C4_DRONE | content::pieces::MACHINE_GUN_DRONE | content::pieces::TACTICAL_NUKE_DRONE | content::pieces::GRENADE_LAUNCHER_DRONE) {
                    views.push(AbilityView { piece_id: p.id, kind: "self_destruct", name: if p.kind == content::pieces::TACTICAL_NUKE_DRONE { "전술핵 자폭" } else if p.kind == content::pieces::C4_DRONE { "광역 자폭" } else { "십자포화 자폭" }, description: "자신을 포함한 폭발 범위의 모든 기물을 확정 제거합니다. 아군도 피해를 받습니다.", targets: vec![AbilityTarget { cell: p.cell, chance: None, blast: self.battle.blast_preview(p.kind, from).into_iter().map(cell).collect(), paths: vec![] }] });
                }
                views
            }).collect()
        } else {
            vec![]
        };
        Snapshot {
            boss: if self.phase == RunPhase::Combat {
                self.battle.boss().map(|state| {
                    let hook = state.hook.and_then(|id| self.battle.piece_position(id));
                    BossView {
                        hp: state.hp,
                        ammo: state.ammo,
                        max_ammo: 2,
                        hook_target: hook.map(|(at, _)| cell(at)),
                        hook_name: hook.map(|(_, p)| piece_name(p.kind)),
                        learned: self
                            .battle
                            .absorbed_kinds(self.battle.royal_id)
                            .into_iter()
                            .map(piece_name)
                            .collect(),
                        learned_abilities: self
                            .battle
                            .absorbed_kinds(self.battle.royal_id)
                            .into_iter()
                            .filter_map(content::pieces::definition)
                            .map(|definition| BossAbilityView {
                                name: definition.name,
                                description: definition.description,
                            })
                            .collect(),
                        plan: self.boss_plan.as_ref().map(|plan| BossPlanView {
                            name: plan.name.clone(),
                            from: cell(plan.action.first.from),
                            to: cell(plan.action.steps().last().unwrap().move_to),
                            area: plan.area.clone(),
                            targets: plan.targets.clone(),
                        }),
                    }
                })
            } else {
                None
            },
            effect_choice: self.battle.pending_choice(),
            squad_id: self.squad_id.clone(),
            progress: self.progress.clone(),
            final_battle: self.final_battle,
            gold: self.gold,
            ammo: self.ammo,
            ammo_capacity: self.ammo_capacity(),
            shop_offers: self.shop_offers.iter().map(|c| c.definition()).collect(),
            shop_bought: self.shop_bought.clone(),
            reinforcement_options,
            hunger_protection_options,
            abilities,
            fallout: self
                .battle
                .fallout_cells()
                .into_iter()
                .map(|(pos, remaining)| FalloutView {
                    cell: cell(pos),
                    remaining,
                })
                .collect(),
            phase: self.phase,
            map: self.map_view(),
            encounter: self
                .current_node
                .map(|id| self.map.nodes[id].name())
                .unwrap_or("갈림길의 입구"),
            node_kind: self.current_node.map(|id| self.map.nodes[id].kind),
            node_reward: self
                .current_node
                .filter(|&id| self.map.nodes[id].kind == NodeKind::Treasure)
                .and_then(|id| self.map.nodes[id].relic)
                .map(|r| RelicView {
                    id: r,
                    status: self.relics.contains(&r).then(|| {
                        "이미 가진 유물입니다. 대신 부대 전용 카드 1장을 얻습니다.".into()
                    }),
                    name: r.name(),
                    description: r.description(),
                }),
            character: self.character.clone(),
            turn: self.battle.turn,
            round: self.round,
            actions_left: self.battle.actions_left,
            mana: self.cards.mana,
            mana_rule: if self.ammo.is_some() {
                "탄약은 턴이나 전투가 끝나도 회복되지 않습니다. 보스 격파 시 전량 보급됩니다."
            } else if self.relics.contains(&Relic::IceCream) {
                "아이스크림: 내 턴 시작마다 남은 코스트에 3을 더합니다."
            } else {
                "내 턴 시작 시 코스트가 3으로 회복됩니다."
            },
            initiative_turns_left: self.battle.initiative_turns_left(),
            enemy_initiative_turns_left: self.battle.enemy_initiative_turns_left(),
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
                            "선택된 시작 진형".into()
                        } else {
                            "비활성 · 더 최근에 획득한 시작 진형이 있습니다.".into()
                        })
                    } else if *r == Relic::IceCream && self.phase == RunPhase::Combat {
                        Some(format!("이번 전투 누적 코스트 잔액 {}", self.cards.mana))
                    } else if self.phase == RunPhase::Combat {
                        self.battle.relic_status(*r)
                    } else {
                        None
                    },
                    name: r.name(),
                    description: r.description(),
                })
                .collect(),
            log: self.log.clone(),
            reward_choice: self.reward_choice,
            upgrade_claimed: self.upgrade_claimed,
            rewards: self
                .rewards
                .iter()
                .filter(|_| self.phase == RunPhase::Reward)
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
                        key: u.key,
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
                .find(|p| p.owner == ENEMY && p.id == self.battle.royal_id)
                .map(|p| piece_name(p.kind))
                .unwrap_or("포획됨"),
            royal_names: self
                .battle
                .board
                .cells()
                .iter()
                .flatten()
                .filter(|p| p.owner == ENEMY && self.battle.is_enemy_royal(p.id))
                .map(|p| piece_name(p.kind))
                .collect(),
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
pub struct AbilityView {
    pub piece_id: PieceId,
    pub kind: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub targets: Vec<AbilityTarget>,
}
#[derive(Serialize)]
pub struct AbilityTarget {
    pub cell: usize,
    pub chance: Option<u8>,
    pub blast: Vec<usize>,
    pub paths: Vec<Vec<usize>>,
}
#[derive(Serialize)]
pub struct FalloutView {
    pub cell: usize,
    pub remaining: u32,
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
#[derive(Clone, Serialize)]
pub struct BossTargetView {
    pub cell: usize,
    pub chance: u8,
    pub name: String,
}
#[derive(Serialize)]
pub struct BossPlanView {
    pub name: String,
    pub from: usize,
    pub to: usize,
    pub area: Vec<usize>,
    pub targets: Vec<BossTargetView>,
}
#[derive(Serialize)]
pub struct BossView {
    pub hp: i32,
    pub ammo: u8,
    pub max_ammo: u8,
    pub hook_target: Option<usize>,
    pub hook_name: Option<&'static str>,
    pub learned: Vec<&'static str>,
    pub learned_abilities: Vec<BossAbilityView>,
    pub plan: Option<BossPlanView>,
}
#[derive(Serialize)]
pub struct BossAbilityView {
    pub name: &'static str,
    pub description: &'static str,
}
#[derive(Serialize)]
pub struct Snapshot {
    pub boss: Option<BossView>,
    pub effect_choice: Option<crate::EffectChoice>,
    pub squad_id: String,
    pub progress: Progress,
    pub final_battle: bool,
    pub gold: u32,
    pub ammo: Option<u32>,
    pub ammo_capacity: Option<u32>,
    pub shop_offers: Vec<CardDefinition>,
    pub shop_bought: Vec<usize>,
    pub reinforcement_options: Vec<RestOption>,
    pub hunger_protection_options: Vec<RestOption>,
    pub abilities: Vec<AbilityView>,
    pub fallout: Vec<FalloutView>,
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
    pub reward_choice: Option<usize>,
    pub upgrade_claimed: bool,
    pub rewards: Vec<RewardView>,
    pub rest_options: Vec<RestOption>,
    pub upgrade_paths: Vec<UpgradeView>,
    pub royal_name: &'static str,
    pub royal_names: Vec<&'static str>,
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
    pub statuses: Vec<String>,
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
#[derive(Clone, Serialize)]
pub struct RestOption {
    pub statuses: Vec<String>,
    pub can_confess: bool,
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
    pub key: &'static str,
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
        game.cards.hand.clear();
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

#[cfg(test)]
mod final_boss_tests {
    use super::*;

    #[test]
    fn final_battle_publishes_and_keeps_the_planned_action_during_player_turn() {
        let mut game = Game::new_run("guardian", 19, 0, Progress::default()).unwrap();
        game.dispatch(Command::SelectNode { id: 0 }).unwrap();
        let announced = game.boss_plan.as_ref().unwrap().action.clone();
        let view = game.snapshot().boss.unwrap();
        assert_eq!(view.hp, 5);
        assert_eq!(view.ammo, 2);
        assert!(view.plan.is_some());
        assert_eq!(
            game.battle
                .board
                .cells()
                .iter()
                .flatten()
                .filter(|p| p.owner == ENEMY)
                .count(),
            1
        );
        let move_view = game.snapshot().moves.into_iter().next().unwrap();
        game.dispatch(Command::Move {
            from: move_view.from,
            to: move_view.to,
            promotion: move_view.promotion.map(|p| p.kind),
        })
        .unwrap();
        assert_eq!(game.boss_plan.as_ref().unwrap().action, announced);
        if game.battle.turn == PLAYER {
            game.dispatch(Command::EndTurn).unwrap();
        }
        if let Some((from, index)) = game.enemy_recommendation().unwrap() {
            assert_eq!(
                game.battle.legal_actions(pos(from).unwrap()).unwrap()[index],
                announced
            );
        }
        game.enemy_step().unwrap();
        assert!(game.phase != RunPhase::Combat || game.battle.turn == PLAYER);
    }
}
