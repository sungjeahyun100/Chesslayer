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
mod debugger;
mod events;
mod expansion;
mod progression;
mod rewards;
mod shop;
mod snapshot;
pub use snapshot::*;
#[cfg(test)]
mod expansion_tests;
#[cfg(test)]
mod v1_integration_tests;
pub use events::{EventKind, EventView};
pub use debugger::ExpeditionDebugConfig;
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
            unlocked_squads: vec!["avenger".into()],
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
/// Disposable setup for a single battle. It is kept in the WASM replay envelope,
/// never in the expedition save slots.
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SimulatorConfig {
    pub squad: String,
    pub encounter: usize,
    pub seed: u32,
    pub deck: Vec<CardKind>,
    pub relics: Vec<Relic>,
    pub upgrades: Vec<CardUpgradeChoice>,
}
#[derive(Clone)]
pub struct Game {
    pub simulator: bool,
    pub expedition_debugger: bool,
    pub debug_log: Vec<String>,
    pub debug_events: Vec<serde_json::Value>,
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
    /// Retain legacy command decoding while retiring cards from the live run.
    pub fn remove_retired_cards(&mut self) {
        let retired = |kind| matches!(kind, CardKind::Rename | CardKind::EnPassant);
        let removed_ids: HashSet<_> = self
            .deck
            .iter()
            .filter(|card| retired(card.kind))
            .map(|card| card.id)
            .collect();
        self.deck.retain(|card| !retired(card.kind));
        self.cards.hand.retain(|card| !retired(card.kind));
        self.cards.draw.retain(|card| !retired(card.kind));
        self.cards.discard.retain(|card| !retired(card.kind));
        self.cards.exhausted.retain(|card| !retired(card.kind));
        self.cards
            .cost_overrides
            .retain(|id, _| !removed_ids.contains(id));
        self.shop_offers.retain(|kind| !retired(*kind));
    }
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
            simulator: false,
            expedition_debugger: false,
            debug_log: vec![],
            debug_events: vec![],
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
            ammo: Some(10),
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
    pub fn new_simulator(config: &SimulatorConfig) -> Result<Self, String> {
        let definition = content::ENCOUNTERS
            .get(config.encounter)
            .ok_or("등록되지 않은 전투 상대입니다.")?;
        let squad =
            content::characters::definition(&config.squad).ok_or("등록되지 않은 부대입니다.")?;
        let allowed_cards: HashSet<_> = squad
            .starting_deck
            .iter()
            .copied()
            .chain(squad.card_pool.iter().copied())
            .collect();
        if config.deck.iter().any(|kind| !allowed_cards.contains(kind)) {
            return Err("이 부대에서 사용할 수 없는 카드가 있습니다.".into());
        }
        if config.relics.iter().copied().collect::<HashSet<_>>().len() != config.relics.len() {
            return Err("같은 유물을 중복 장착할 수 없습니다.".into());
        }
        if config
            .relics
            .iter()
            .any(|relic| *relic != squad.character.relic && !squad.relic_pool.contains(relic))
        {
            return Err("이 부대에서 획득할 수 없는 유물이 있습니다.".into());
        }
        let mut game = Self::new(&config.squad, config.seed)?;
        game.simulator = true;
        game.deck.clear();
        game.next_card_id = 1;
        for &kind in &config.deck {
            game.add_card(kind);
        }
        game.relics = config.relics.clone();
        let mut upgraded = HashSet::new();
        for upgrade in &config.upgrades {
            if !upgraded.insert(upgrade.id) {
                return Err("같은 기물의 강화가 중복되었습니다.".into());
            }
            game.developer_set_upgrade(upgrade.id, upgrade.target)?;
        }
        game.map = ActMap {
            rows: 1,
            nodes: vec![MapNode {
                id: 0,
                row: 0,
                lane: 2,
                kind: match definition.tier {
                    content::encounters::EncounterTier::Normal => NodeKind::Combat,
                    content::encounters::EncounterTier::Elite => NodeKind::Elite,
                    content::encounters::EncounterTier::Boss => NodeKind::Boss,
                },
                encounter: Some(config.encounter),
                relic: None,
                next: vec![],
            }],
        };
        game.floor = u32::from(definition.floor.max(1));
        game.final_battle = config.encounter == content::encounters::DICTATOR_BOSS;
        game.current_node = Some(0);
        game.log.clear();
        game.start_battle()?;
        Ok(game)
    }
    pub fn new_run_with_mode(
        character: &str,
        seed: u32,
        floor: u8,
        progress: Progress,
        mode: RunMode,
    ) -> Result<Self, String> {
        if character != "avenger" {
            return Err("이 버전에서는 복수귀 부대만 플레이할 수 있습니다.".into());
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
            10 + self
                .battle
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
        self.cards.mana = self.cards.mana.saturating_add_signed(mana);
        if let Some(current) = &mut self.ammo {
            *current = current.saturating_add_signed(ammo);
        }
        if let Some(capacity) = self.ammo_capacity() {
            self.ammo = self.ammo.map(|current| current.min(capacity));
        }
        self.battle.set_ammunition(self.ammo.unwrap_or(0));
    }
    fn log(&mut self, message: impl Into<String>) {
        let message = message.into();
        if self.expedition_debugger {
            self.debug_log.push(message.clone());
            let actor = self.relics.iter()
                .find(|relic| message.starts_with(&format!("{}:", relic.name())))
                .map(|relic| relic.name().to_string());
            self.debug_event("engine_log", actor, None, &message);
        }
        self.log.push(message);
        if self.log.len() > 60 {
            self.log.remove(0);
        }
    }
    fn debug_event(&mut self, kind: &str, actor: Option<String>, target: Option<String>, result: &str) {
        if !self.expedition_debugger { return; }
        let state = serde_json::json!({
            "phase": self.phase, "floor": self.floor, "current_node": self.current_node,
            "gold": self.gold, "deck": self.deck, "relics": self.relics,
            "roster": self.battle.outcome().iter().map(|p| serde_json::json!({
                "id": p.id, "kind": p.kind, "alive": p.position.is_some(),
                "position": p.position.map(cell), "persistent": p.persistent,
            })).collect::<Vec<_>>(),
            "cards": {"mana": self.cards.mana, "hand": self.cards.hand,
                "draw": self.cards.draw, "discard": self.cards.discard,
                "exhausted": self.cards.exhausted},
            "battle": self.battle.debug_state(), "round": self.round,
            "cleared_stages": self.cleared_stages,
        });
        self.debug_events.push(serde_json::json!({
            "order": self.debug_events.len() + 1, "kind": kind,
            "actor": actor, "target": target, "result": result, "state": state,
        }));
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
                    .ok_or("전투불능인 기물을 선택하세요.")?;
                p.persistent.remove("incapacitated");
                self.battle
                    .replace_roster_at_rest(roster)
                    .map_err(engine_error)?;
                self.complete_node();
                self.log("고해성사: 전투불능이 해제되었습니다.");
            }
            Command::RestSupply { id: _ } => {
                if self.phase != RunPhase::Rest {
                    return Err("쉼터에서만 포탄을 보충합니다.".into());
                }
                let capacity = self.ammo_capacity().ok_or("포탄 자원이 없습니다.")?;
                let current = self.ammo.unwrap();
                if current >= capacity {
                    return Err("포탄이 이미 가득 찼습니다.".into());
                }
                // The artillery depot restored 10% of current ammunition.
                // A one-shell minimum keeps the scaled 10-shell pool refillable.
                let restored = (current / 10).max(1);
                self.ammo = Some(current.saturating_add(restored).min(capacity));
                self.log(format!("휴식: 포탄량 {} → {}", current, self.ammo.unwrap()));
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
                    .find(|p| p.id == id && p.position.is_none() && p.id != self.battle.king_id)
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
mod v1_content_tests {
    use super::*;

    #[test]
    fn generated_shop_and_rewards_use_only_avenger_cards() {
        let mut game = Game::new("avenger", 19).unwrap();
        let allowed = content::cards::avenger::ALL;
        for _ in 0..32 {
            game.generate_shop_offers();
            assert!(game.shop_offers.iter().all(|card| allowed.contains(card)));
            game.generate_rewards();
            assert!(game
                .rewards
                .iter()
                .all(|reward| matches!(reward, Reward::Card(card) if allowed.contains(card))));
        }
    }
}
#[cfg(test)]
mod combat_event_tests {
    use super::*;

    #[test]
    fn snapshot_serializes_resolved_events_and_next_command_clears_them() {
        let mut game = Game::new("avenger", 1).unwrap();
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
        let mut game = Game::new_run("avenger", 19, 0, Progress::default()).unwrap();
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
        let mut game = Game::new_run("avenger", 19, 0, Progress::default()).unwrap();
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
        game.battle.grant_move_rights(9, 1).unwrap();
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
        let mut game = Game::new_run("avenger", 19, 0, Progress::default()).unwrap();
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
}
