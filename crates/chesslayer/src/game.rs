use crate::{
    ai::{self, Budget},
    battle::GeneratedAction,
    cards::*,
    content::*,
    map::*,
    *,
};
use serde::{Deserialize, Serialize};
mod battle_flow;
mod card_play;
mod events;
mod expansion;
mod progression;
mod rewards;
mod shop;
mod snapshot;
pub use snapshot::*;
#[cfg(test)]
mod expansion_tests;
pub use events::{EventKind, EventView};
use std::collections::{HashSet, VecDeque};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunMode {
    #[default]
    Campaign,
    Endless,
}

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
            unlocked_squads: vec!["guardian".into(), "avenger".into()],
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
    PlayNamedCard {
        id: u32,
        target: u32,
        name: String,
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
    KingReposition {
        from: usize,
        targets: Vec<usize>,
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
    ShopBurn {
        id: u32,
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
    SkipTreasure,
    RestUpgrade {
        id: u32,
        target: PieceType,
    },
    DeveloperSetUpgrade {
        id: u32,
        target: PieceType,
    },
    RestRevive {
        id: u32,
    },
    RestSkip,
    RestBurn {
        id: u32,
    },
    RestReplaceRelic {
        relic: Relic,
    },
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CardUpgradeChoice {
    pub id: PieceId,
    pub target: PieceType,
}
#[derive(Clone)]
pub struct Game {
    event_state: events::EventState,
    pub endless_scaling: Option<content::endless::Scaling>,
    pub bosses_defeated: u32,
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
    pub floor: u32,
    pub mode: RunMode,
    pub cleared_stages: u64,
    last_counted_node: Option<usize>,
    pub character: String,
    pub squad_id: String,
    pub progress: Progress,
    pub final_battle: bool,
    pub gold: u32,
    pub cards_burned: u32,
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
    boss_plans: Vec<BossPlan>,
    boss_sequence_step: u8,
    replaying_command: bool,
    /// Replay version 23's seeded content before the migration boundary.
    pub legacy_content: bool,
    legacy_boss_turn: bool,
    legacy_boss_choice: bool,
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
        let squad = content::characters::definition(character).ok_or("캐릭터를 선택하세요.")?;
        let choice = squad.character;
        let deck = content::cards::starting_deck(character);
        let next_card_id = deck.iter().map(|card| card.id).max().unwrap_or(0) + 1;
        let battle = Battle::new(BattleStart {
            players: content::characters::roster(character),
            encounter: encounter(0),
        })
        .map_err(engine_error)?;
        let mut game = Self {
            event_state: events::EventState::default(),
            endless_scaling: None,
            bosses_defeated: 0,
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
            mode: RunMode::Campaign,
            cleared_stages: 0,
            last_counted_node: None,
            character: choice.name.into(),
            squad_id: character.into(),
            progress: Progress::default(),
            final_battle: false,
            gold: 0,
            cards_burned: 0,
            ammo: match squad.resource {
                content::characters::ResourceRule::Mana => None,
                content::characters::ResourceRule::ArtilleryAmmo { initial } => Some(initial),
            },
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
            boss_plans: vec![],
            boss_sequence_step: 0,
            replaying_command: false,
            legacy_content: false,
            legacy_boss_turn: false,
            legacy_boss_choice: false,
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
        Self::new_run_with_mode(character, seed, floor, progress, RunMode::Campaign)
    }
    pub fn new_run_with_mode(
        character: &str,
        seed: u32,
        floor: u8,
        progress: Progress,
        mode: RunMode,
    ) -> Result<Self, String> {
        if character != "avenger" && !progress.unlocked_squads.iter().any(|id| id == character) {
            return Err("아직 해금되지 않은 부대입니다.".into());
        }
        if floor > 4 {
            return Err("존재하지 않는 층입니다.".into());
        }
        if mode == RunMode::Endless && floor != 1 {
            return Err("무한 원정은 1층에서 시작합니다.".into());
        }
        let mut game = Self::new(character, seed)?;
        game.mode = mode;
        game.progress = progress;
        game.final_battle = floor == 0;
        game.floor = u32::from(if floor == 0 { 4 } else { floor });
        game.map = if mode == RunMode::Endless {
            ActMap::generate_endless(seed, game.floor)
        } else if game.final_battle {
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
    fn require_player(&self) -> Result<(), String> {
        if self.phase != RunPhase::Combat || self.battle.turn != PLAYER {
            return Err("플레이어의 전투 턴에만 사용할 수 있습니다.".into());
        }
        if self.battle.pending_choice().is_some() {
            return Err("먼저 발동한 효과의 대상을 선택하세요.".into());
        }
        Ok(())
    }
    /// Restore recorded selections without asking a newer AI to choose the past
    /// again. Live commands still enforce the boss's announced action.
    pub fn replay_command(&mut self, command: Command) -> Result<(), String> {
        self.replaying_command = true;
        let result = self.dispatch(command);
        self.replaying_command = false;
        result
    }

    /// Version 19 histories finished the boss turn after one recorded action.
    /// Keep that boundary only while replaying the migrated history prefix.
    pub fn replay_legacy_command(&mut self, command: Command) -> Result<(), String> {
        self.legacy_boss_turn = true;
        let result = self.replay_command(command);
        self.legacy_boss_turn = false;
        result
    }

    pub fn finish_legacy_replay(&mut self) {
        // A save may stop between a legacy action and its target selection.
        // Keep that action's sequence until the final choice resolves; planning
        // cannot advance a projected turn while a choice is still pending.
        self.legacy_boss_choice = self.battle.pending_choice().is_some();
        if !self.legacy_boss_choice {
            self.plan_boss();
        }
    }

    pub fn dispatch(&mut self, command: Command) -> Result<(), String> {
        // One dispatch owns one presentation batch. Queries and AI clones never
        // drain or append to the live game's resolved effect records.
        self.battle.combat_events.clear();
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
            Command::PlayNamedCard { id, target, name } => {
                self.require_player()?;
                let index = self
                    .cards
                    .hand
                    .iter()
                    .position(|c| c.id == id && c.kind == CardKind::Rename)
                    .ok_or("이름 변경 카드가 없습니다.")?;
                self.play_avenger_card(index, Some(target), Some(&name))?;
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
                if (self.legacy_boss_turn || self.legacy_boss_choice)
                    && side == ENEMY
                    && self.battle.turn == ENEMY
                    && self.battle.boss().is_some()
                    && self.boss_sequence_step >= 1
                    && self.battle.phase == Phase::Active
                    && self.battle.pending_choice().is_none()
                {
                    self.battle.end_turn().map_err(engine_error)?;
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
                if self.legacy_boss_choice && self.battle.pending_choice().is_none() {
                    self.legacy_boss_choice = false;
                    self.plan_boss();
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
            Command::KingReposition { from, targets } => {
                self.require_player()?;
                let side = self.battle.turn;
                let positions = targets
                    .into_iter()
                    .map(pos)
                    .collect::<Result<Vec<_>, _>>()?;
                self.battle
                    .reposition_enemies(pos(from)?, &positions)
                    .map_err(engine_error)?;
                self.events();
                self.check_finish()?;
                if self.phase == RunPhase::Combat && self.battle.turn != side {
                    self.cards.end_turn();
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
            Command::EventChoice { choice } => self.choose_event(&choice)?,
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
            Command::ShopBurn { id } => {
                if self.phase != RunPhase::Shop {
                    return Err("상점에 있지 않습니다.".into());
                }
                if !self.deck.iter().any(|card| card.id == id) {
                    return Err("소각할 카드가 없습니다.".into());
                }
                let cost = self.next_burn_cost();
                if self.gold < cost {
                    return Err("금화가 부족합니다.".into());
                }
                self.gold -= cost;
                self.cards_burned = self.cards_burned.saturating_add(1);
                let card = self.burn_card(id)?;
                self.log(format!(
                    "상점: {} 소각 · 금화 {cost}",
                    card.kind.definition().name
                ));
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
                        self.enter_random_event();
                    }
                    NodeKind::Shop => {
                        self.phase = RunPhase::Shop;
                        self.generate_shop_offers();
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
            Command::SkipTreasure => {
                if self.phase != RunPhase::Treasure {
                    return Err("지금은 보물방을 떠날 수 없습니다.".into());
                }
                self.complete_node();
                self.log("보물을 두고 떠났습니다.");
            }
            Command::RestUpgrade { id, target } => {
                if self.phase != RunPhase::Rest {
                    return Err("쉼터에서만 강화할 수 있습니다.".into());
                }
                self.upgrade_piece(id, target, "휴식")?;
                self.complete_node();
            }
            Command::DeveloperSetUpgrade { id, target } => {
                self.developer_set_upgrade(id, target)?;
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
            Command::RestBurn { id } => {
                if self.phase != RunPhase::Rest || self.mode != RunMode::Endless {
                    return Err("무한 원정의 휴식에서만 무료 소각할 수 있습니다.".into());
                }
                let card = self.burn_card(id)?;
                self.log(format!("휴식: {} 무료 소각", card.kind.definition().name));
                self.complete_node();
            }
            Command::RestReplaceRelic { relic } => {
                if self.phase != RunPhase::Rest || self.mode != RunMode::Endless {
                    return Err("무한 원정의 휴식에서만 유물을 교체할 수 있습니다.".into());
                }
                let index = self
                    .relics
                    .iter()
                    .position(|r| *r == relic)
                    .ok_or("보유하지 않은 유물입니다.")?;
                let candidates: Vec<_> = content::relics::reward_pool(&self.squad_id)
                    .iter()
                    .copied()
                    .filter(|r| !self.relics.contains(r))
                    .collect();
                if candidates.is_empty() {
                    return Err("교체할 미보유 유물이 없습니다.".into());
                }
                let replacement = candidates[self.rng.index(candidates.len())];
                self.relics.remove(index);
                self.grant_relic(replacement);
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
}
#[cfg(test)]
mod shop_burn_tests {
    use super::*;

    #[test]
    fn burn_removes_only_selected_copy_from_deck_and_all_piles() {
        let mut game = Game::new("vanguard", 7).unwrap();
        game.phase = RunPhase::Shop;
        game.gold = 600;
        let card = game.deck[0].clone();
        game.add_card(card.kind);
        let same_kind = game.deck.last().unwrap().id;
        game.cards.hand.push(card.clone());
        game.cards.draw.push(card.clone());
        game.cards.discard.push(card.clone());
        game.cards.exhausted.push(card.clone());

        game.dispatch(Command::ShopBurn { id: card.id }).unwrap();

        assert_eq!(game.gold, 500);
        assert_eq!(game.cards_burned, 1);
        assert_eq!(game.next_burn_cost(), 200);
        assert!(!game.deck.iter().any(|other| other.id == card.id));
        for pile in [
            &game.cards.hand,
            &game.cards.draw,
            &game.cards.discard,
            &game.cards.exhausted,
        ] {
            assert!(!pile.iter().any(|other| other.id == card.id));
        }
        assert!(game.deck.iter().any(|other| other.id == same_kind));
        assert_eq!(game.snapshot().deck_count, game.deck.len());
    }

    #[test]
    fn burn_cost_persists_across_shops_and_rejects_invalid_requests() {
        let mut game = Game::new("vanguard", 8).unwrap();
        let first = game.deck[0].id;
        let second = game.deck[1].id;
        game.gold = 199;
        assert!(game.dispatch(Command::ShopBurn { id: first }).is_err());
        game.phase = RunPhase::Shop;
        assert!(game.dispatch(Command::ShopBurn { id: u32::MAX }).is_err());
        game.dispatch(Command::ShopBurn { id: first }).unwrap();
        assert_eq!(game.gold, 99);
        assert!(game.dispatch(Command::ShopBurn { id: second }).is_err());
        assert_eq!(game.gold, 99);
        assert!(game.deck.iter().any(|card| card.id == second));
        game.phase = RunPhase::Map;
        game.phase = RunPhase::Shop;
        game.gold = 300;
        game.dispatch(Command::ShopBurn { id: second }).unwrap();
        assert_eq!(game.gold, 100);
        assert_eq!(game.next_burn_cost(), 300);
        game.shop_offers = vec![CardKind::Stimpack];
        let deck_size = game.deck.len();
        game.dispatch(Command::ShopBuy { index: 0 }).unwrap();
        assert_eq!(game.gold, 80);
        assert_eq!(game.deck.len(), deck_size + 1);
        assert_eq!(game.next_burn_cost(), 300);
        assert_eq!(Game::new("vanguard", 8).unwrap().next_burn_cost(), 100);
    }
}

#[cfg(test)]
mod card_pile_view_tests {
    use super::*;

    #[test]
    fn combat_snapshot_lists_piles_without_draw_order() {
        let mut game = Game::new("vanguard", 2026).unwrap();
        game.dispatch(Command::SelectNode { id: 0 }).unwrap();
        let state = game.snapshot();
        let ids: Vec<_> = state.draw_pile.iter().map(|card| card.id).collect();
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        assert_eq!(ids, sorted);
        assert_eq!(state.draw_pile.len(), state.draw_count);
        assert_eq!(state.discard_pile.len(), state.discard_count);
        assert_eq!(state.exhausted_pile.len(), state.exhausted_count);
        assert_eq!(
            state.hand.len() + state.draw_count + state.discard_count + state.exhausted_count,
            state.deck_count
        );
    }
}

#[cfg(test)]
mod developer_upgrade_tests {
    use super::*;

    #[test]
    fn changes_branches_and_resets_the_permanent_king_without_moving_it() {
        let mut game = Game::new("guardian", 2026).unwrap();
        let king = game
            .battle
            .player_pieces()
            .iter()
            .find(|p| p.kind == content::pieces::KING)
            .unwrap()
            .clone();
        game.phase = RunPhase::Combat;
        game.battle.configure_v030(&[]);
        for target in [
            content::pieces::JOCKEY,
            content::pieces::HILL_KING,
            content::pieces::KING,
        ] {
            game.dispatch(Command::DeveloperSetUpgrade {
                id: king.id,
                target,
            })
            .unwrap();
            let current = game
                .battle
                .player_pieces()
                .iter()
                .find(|p| p.id == king.id)
                .unwrap();
            assert_eq!(current.kind, target);
            assert_eq!(current.position, king.position);
            assert_eq!(
                game.battle
                    .board
                    .piece_at(king.position.unwrap())
                    .unwrap()
                    .unwrap()
                    .kind,
                target
            );
            if target == content::pieces::HILL_KING {
                assert!(!game.battle.has_ward(king.id));
                assert!(!game
                    .battle
                    .status_labels(king.id)
                    .iter()
                    .any(|label| label.contains("민첩")));
            }
        }
        assert!(game
            .battle
            .player_pieces()
            .iter()
            .find(|p| p.id == king.id)
            .unwrap()
            .upgrades
            .is_empty());
        assert!(game
            .dispatch(Command::DeveloperSetUpgrade {
                id: king.id,
                target: content::pieces::PAWN
            })
            .is_err());
    }
}

#[cfg(test)]
mod relic_resource_tests {
    use super::*;

    #[test]
    fn volley_hand_cost_tracks_the_current_board() {
        let mut game = Game::new("artillery", 2026).unwrap();
        game.phase = RunPhase::Combat;
        game.battle = Battle::sandbox().unwrap();
        game.cards.hand = vec![Card {
            id: 1,
            kind: CardKind::Volley,
        }];
        game.cards.mana = 2;
        let empty = &game.snapshot().hand[0];
        assert_eq!(empty.definition.cost, 0);
        assert!(empty.disabled_reason.is_some());
        game.battle
            .sandbox_place(1, content::pieces::RIFLEMAN, PLAYER, Position::new(3, 6))
            .unwrap();
        game.battle
            .sandbox_place(101, content::pieces::PAWN, ENEMY, Position::new(3, 5))
            .unwrap();
        let ready = &game.snapshot().hand[0];
        assert_eq!(ready.definition.cost, 1);
        assert!(ready.disabled_reason.is_none());
        game.battle.sandbox_remove(Position::new(3, 5)).unwrap();
        assert_eq!(game.snapshot().hand[0].definition.cost, 0);
    }

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
mod combat_event_tests {
    use super::*;

    #[test]
    fn snapshot_serializes_resolved_events_and_next_command_clears_them() {
        let mut game = Game::new("guardian", 1).unwrap();
        game.battle.combat_events.push(CombatEvent::Shotgun {
            actor_id: 100,
            from: Position::new(4, 4),
            toward: Position::new(4, 3),
            rays: vec![ShotgunRay {
                direction: (0, -1),
                cells: vec![Position::new(4, 3)],
            }],
            hits: vec![ShotgunHit {
                at: Position::new(4, 3),
                piece_id: 2,
                result: ShotgunResult::Ward,
            }],
        });
        let view = serde_json::to_value(game.snapshot()).unwrap();
        let event = &view["combat_events"][0];
        assert_eq!(event["kind"], "shotgun");
        assert_eq!(event["from"], 36);
        assert_eq!(event["toward"], 28);
        assert_eq!(event["area"], serde_json::json!([28]));
        assert_eq!(event["rays"][0]["direction"], serde_json::json!([0, -1]));
        assert_eq!(event["hits"][0]["result"], "ward");
        // Read-only queries cannot consume a batch; commands own its lifetime,
        // including rejected commands that must not replay old effects.
        assert_eq!(game.snapshot().combat_events.len(), 1);
        assert!(game.dispatch(Command::EndTurn).is_err());
        assert!(game.snapshot().combat_events.is_empty());
    }
}

#[cfg(test)]
mod final_boss_tests {
    use super::*;

    #[test]
    fn recorded_boss_selection_replays_without_changing_live_plan_enforcement() {
        let mut game = Game::new_run("guardian", 19, 0, Progress::default()).unwrap();
        game.dispatch(Command::SelectNode { id: 0 }).unwrap();
        game.dispatch(Command::EndTurn).unwrap();
        let announced = game.boss_plans.first().unwrap().action.clone();
        let from = announced.first.from;
        let (index, recorded) = game
            .battle
            .legal_actions(from)
            .unwrap()
            .into_iter()
            .enumerate()
            .find(|(_, action)| action.first.move_to != from && action != &announced)
            .unwrap();
        let command = Command::EnemyStepSelected {
            from: Some(cell(from)),
            index: Some(index),
        };
        let mut replayed = game.clone();
        replayed.replay_command(command.clone()).unwrap();
        assert_eq!(
            replayed
                .battle
                .piece_position(replayed.battle.royal_id)
                .unwrap()
                .0,
            recorded.first.move_to
        );
        assert!(!replayed.replaying_command);
        game.dispatch(command).unwrap();
        assert_eq!(
            game.battle.piece_position(game.battle.royal_id).unwrap().0,
            announced.first.move_to
        );
    }

    #[test]
    fn final_battle_publishes_and_keeps_the_planned_action_during_player_turn() {
        let mut game = Game::new_run("guardian", 19, 0, Progress::default()).unwrap();
        game.dispatch(Command::SelectNode { id: 0 }).unwrap();
        let announced = game.boss_plans.first().unwrap().action.clone();
        let announced_turn: Vec<_> = game
            .boss_plans
            .iter()
            .map(|plan| plan.action.clone())
            .collect();
        let view = game.snapshot().boss.unwrap();
        assert_eq!(view.hp, 5);
        assert_eq!(view.ammo, 2);
        assert!(view.plan.is_some());
        assert_eq!(view.plans.len(), announced_turn.len());
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
        assert_eq!(game.boss_plans.first().unwrap().action, announced);
        assert_eq!(
            game.boss_plans
                .iter()
                .map(|plan| plan.action.clone())
                .collect::<Vec<_>>(),
            announced_turn
        );
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
        if game.phase == RunPhase::Combat && game.battle.turn == ENEMY {
            assert_eq!(game.boss_sequence_step, 1);
            assert_eq!(
                game.snapshot().boss.unwrap().plans.len(),
                announced_turn.len() - 1
            );
            game.enemy_step().unwrap();
        }
        assert!(game.phase != RunPhase::Combat || game.battle.turn == PLAYER);
    }

    #[test]
    fn boss_move_and_shot_are_separate_steps_of_the_same_turn() {
        let mut game = Game::new_run("guardian", 19, 0, Progress::default()).unwrap();
        game.dispatch(Command::SelectNode { id: 0 }).unwrap();
        let mut projected = game.battle.clone();
        projected.end_turn().unwrap();
        let from = projected.piece_position(projected.royal_id).unwrap().0;
        let movement = projected
            .legal_actions(from)
            .unwrap()
            .into_iter()
            .find(Battle::boss_action_is_movement)
            .unwrap();
        let first = Game::describe_boss_action(&projected, movement.clone());
        projected.apply_piece_action(&movement).unwrap();
        let at = projected.piece_position(projected.royal_id).unwrap().0;
        let shot = projected
            .legal_actions(at)
            .unwrap()
            .into_iter()
            .find(|action| action.first.transition.as_deref() == Some("boss_shot"))
            .unwrap();
        let second = Game::describe_boss_action(&projected, shot.clone());
        game.boss_plans = vec![first, second];
        game.dispatch(Command::EndTurn).unwrap();
        let round = game.round;
        assert_eq!(game.battle.actions_left, 2);
        game.enemy_step().unwrap();
        assert_eq!(game.battle.turn, ENEMY);
        assert_eq!(game.round, round);
        assert_eq!(game.battle.actions_left, 1);
        assert_eq!(
            game.battle.piece_position(game.battle.royal_id).unwrap().0,
            at
        );
        assert_eq!(game.battle.boss().unwrap().ammo, 2);
        assert_eq!(game.snapshot().boss.unwrap().plan.unwrap().from, cell(at));
        game.enemy_step().unwrap();
        assert_eq!(game.battle.turn, PLAYER);
        assert_eq!(game.round, round + 1);
        assert_eq!(game.battle.boss().unwrap().ammo, 1);
        assert!(game
            .battle
            .combat_events
            .iter()
            .any(|event| matches!(event, CombatEvent::Shotgun { from, .. } if *from == at)));
    }

    #[test]
    fn legacy_enemy_choice_keeps_sequence_and_finishes_old_turn_after_resolution() {
        let mut game = Game::new_run("guardian", 19, 0, Progress::default()).unwrap();
        game.dispatch(Command::SelectNode { id: 0 }).unwrap();
        game.battle = Battle::new(BattleStart {
            players: vec![
                PlayerPiece {
                    id: 1,
                    kind: pieces::KING,
                    origin: Position::new(7, 7),
                    position: Some(Position::new(7, 7)),
                    moved: false,
                    upgrades: vec![],
                    persistent: Default::default(),
                },
                PlayerPiece {
                    id: 2,
                    kind: pieces::PAWN,
                    origin: Position::new(3, 4),
                    position: Some(Position::new(3, 4)),
                    moved: false,
                    upgrades: vec![],
                    persistent: Default::default(),
                },
            ],
            encounter: Encounter {
                enemies: vec![EnemyPiece {
                    id: 100,
                    kind: pieces::TRIBAL_DICTATOR,
                    position: Position::new(4, 4),
                }],
                royal_id: 100,
                additional_royal_ids: vec![],
            },
        })
        .unwrap();
        game.battle
            .v030_card_effect(
                crate::content::cards::CardKind::ReliefForce,
                &crate::content::cards::CardSelection::single(None),
            )
            .unwrap();
        game.dispatch(Command::EndTurn).unwrap();
        let from = Position::new(4, 4);
        let (index, capture) = game
            .battle
            .legal_actions(from)
            .unwrap()
            .into_iter()
            .enumerate()
            .find(|(_, action)| action.first.move_to == Position::new(3, 4))
            .unwrap();
        game.boss_plans = vec![Game::describe_boss_action(&game.battle, capture.clone())];
        game.boss_sequence_step = 0;
        let round = game.round;
        game.replay_legacy_command(Command::EnemyStepSelected {
            from: Some(cell(from)),
            index: Some(index),
        })
        .unwrap();
        let reaction = game.battle.pending_choice().unwrap();
        assert_eq!(game.battle.turn, ENEMY);
        assert_eq!(game.boss_sequence_step, 1);
        game.finish_legacy_replay();
        assert!(game.legacy_boss_choice);
        assert_eq!(game.boss_sequence_step, 1);
        assert_eq!(game.boss_plans[0].action, capture);

        let hash = game.battle.search_hash();
        assert!(game
            .dispatch(Command::EnemyStepSelected {
                from: None,
                index: None,
            })
            .is_err());
        assert_eq!(game.battle.search_hash(), hash);
        assert_eq!(game.boss_sequence_step, 1);
        assert!(game.legacy_boss_choice);
        game.dispatch(Command::ResolveEffect {
            target: reaction.targets[0],
        })
        .unwrap();
        assert_eq!(game.battle.turn, PLAYER);
        assert_eq!(game.round, round + 1);
        assert_eq!(game.battle.boss().unwrap().ammo, 2);
        assert!(game.battle.pending_choice().is_none());
        assert!(!game.legacy_boss_choice);
        assert_eq!(game.boss_sequence_step, 0);
        assert!(!game.boss_plans.is_empty());
    }
}
