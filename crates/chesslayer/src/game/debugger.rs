//! Developer-only entry points into the ordinary expedition state machine.
use super::*;

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExpeditionDebugConfig {
    pub squad: String,
    pub seed: u32,
    pub floor: u8,
    pub gold: u32,
    pub deck: Vec<CardKind>,
    pub relics: Vec<Relic>,
    pub upgrades: Vec<CardUpgradeChoice>,
    #[serde(default)]
    pub dead: Vec<PieceId>,
    #[serde(default)]
    pub cleared_stages: u64,
    #[serde(default)]
    pub bosses_defeated: u32,
    #[serde(default)]
    pub ammo: Option<u32>,
    #[serde(default)]
    pub cards_burned: u32,
    #[serde(default)]
    pub reinforcement_used: bool,
    #[serde(default)]
    pub hunger_protection_used: bool,
    #[serde(default)]
    pub next_battle_penalty: bool,
    #[serde(default)]
    pub ambushed: bool,
}

impl Game {
    pub fn new_expedition_debugger(config: &ExpeditionDebugConfig) -> Result<Self, String> {
        if !(1..=4).contains(&config.floor) {
            return Err("시작 층은 1~4여야 합니다.".into());
        }
        let squad = content::characters::definition(&config.squad)
            .ok_or("등록되지 않은 부대입니다.")?;
        let allowed_cards: HashSet<_> = squad.starting_deck.iter().copied()
            .chain(squad.card_pool.iter().copied()).collect();
        if config.deck.iter().any(|kind| !allowed_cards.contains(kind)) {
            return Err("이 부대에서 사용할 수 없는 카드가 있습니다.".into());
        }
        if config.relics.iter().copied().collect::<HashSet<_>>().len() != config.relics.len()
            || config.relics.iter().any(|relic| *relic != squad.character.relic
                && !squad.relic_pool.contains(relic)) {
            return Err("이 부대에서 사용할 수 없는 유물이 있습니다.".into());
        }
        let mut game = Self::new(&config.squad, config.seed)?;
        game.expedition_debugger = true;
        game.floor = config.floor.into();
        game.map = ActMap::generate_for_floor(config.seed, config.floor);
        game.assign_map_relics();
        game.gold = config.gold;
        game.cleared_stages = config.cleared_stages;
        game.bosses_defeated = config.bosses_defeated;
        game.cards_burned = config.cards_burned;
        game.reinforcement_used = config.reinforcement_used;
        game.hunger_protection_used = config.hunger_protection_used;
        game.event_state.next_battle_penalty = config.next_battle_penalty;
        game.event_state.ambushed = config.ambushed;
        if let Some(ammo) = config.ammo { game.ammo = Some(ammo); }
        game.deck.clear();
        game.next_card_id = 1;
        for &kind in &config.deck { game.add_card(kind); }
        game.relics = config.relics.clone();
        let mut upgraded = HashSet::new();
        for upgrade in &config.upgrades {
            if !upgraded.insert(upgrade.id) { return Err("같은 기물의 강화가 중복되었습니다.".into()); }
            game.developer_set_upgrade(upgrade.id, upgrade.target)?;
        }
        if !config.dead.is_empty() {
            let mut roster = game.battle.outcome();
            for id in &config.dead {
                let piece = roster.iter_mut().find(|p| p.id == *id)
                    .ok_or("등록되지 않은 사망 기물입니다.")?;
                if piece.id == game.battle.king_id { return Err("로얄 기물은 시작 전에 사망시킬 수 없습니다.".into()); }
                piece.position = None;
            }
            game.battle.phase = Phase::Finished;
            game.battle.replace_roster_at_rest(roster).map_err(engine_error)?;
        }
        game.log.clear();
        game.debug_log.clear();
        game.debug_events.clear();
        Ok(game)
    }

    /// Explicit debugger jump: preserve run resources, but replace traversal position.
    pub fn debug_jump(&mut self, floor: u8, row: usize) -> Result<(), String> {
        if !(1..=4).contains(&floor) { return Err("층은 1~4여야 합니다.".into()); }
        if self.phase == RunPhase::Combat { return Err("전투 중에는 강제 이동할 수 없습니다.".into()); }
        if row >= ActMap::generate_for_floor(self.seed, floor).rows { return Err("해당 진행 단계가 없습니다.".into()); }
        self.floor = floor.into();
        self.final_battle = false;
        self.map = ActMap::generate_for_floor(self.seed ^ self.floor.wrapping_mul(0x9e37_79b9), floor);
        self.assign_map_relics();
        self.current_node = None;
        self.path.clear();
        self.completed.clear();
        self.last_cleared_node = None;
        self.last_counted_node = None;
        self.reward_node = None;
        self.reward_choice = None;
        self.upgrade_claimed = false;
        self.phase = RunPhase::Map;
        if row > 0 {
            let id = self.map.nodes.iter().find(|n| n.row == row - 1)
                .map(|n| n.id).ok_or("이전 단계가 없습니다.")?;
            self.current_node = Some(id);
            self.path.push(id);
            self.completed.push(id);
        }
        self.log(format!("디버거 강제 이동: {floor}층 {row}단계"));
        Ok(())
    }

    /// Replace an available node's definition, then enter through SelectNode.
    pub fn debug_enter_node(&mut self, kind: NodeKind, encounter_index: Option<usize>) -> Result<(), String> {
        if self.phase != RunPhase::Map { return Err("지도 화면에서만 노드를 생성할 수 있습니다.".into()); }
        let id = *self.available_nodes().first().ok_or("진입 가능한 다음 노드가 없습니다.")?;
        let encounter_index = if matches!(kind, NodeKind::Combat | NodeKind::Elite | NodeKind::Boss) {
            let index = encounter_index.ok_or("전투 상대를 선택하세요.")?;
            let definition = content::ENCOUNTERS.get(index).ok_or("등록되지 않은 전투 상대입니다.")?;
            let expected = match kind {
                NodeKind::Combat => content::encounters::EncounterTier::Normal,
                NodeKind::Elite => content::encounters::EncounterTier::Elite,
                _ => content::encounters::EncounterTier::Boss,
            };
            if definition.tier != expected { return Err("노드 종류와 전투 상대 등급이 다릅니다.".into()); }
            Some(index)
        } else { None };
        let previous = self.clone();
        if encounter_index == Some(content::encounters::DICTATOR_BOSS) {
            self.floor = 4;
            self.final_battle = true;
        }
        let node = &mut self.map.nodes[id];
        node.kind = kind;
        node.encounter = encounter_index;
        node.relic = if kind == NodeKind::Treasure {
            content::relics::reward_pool(&self.squad_id).first().copied()
        } else { None };
        self.assign_map_relics();
        self.log(format!("디버거 강제 노드: {:?}", kind));
        match self.dispatch(Command::SelectNode { id }) {
            Ok(()) => Ok(()),
            Err(error) => { *self = previous; Err(error) }
        }
    }

    /// The override sets only the result; check_finish remains the sole cleanup/reward path.
    pub fn debug_finish_battle(&mut self, winner: PlayerId) -> Result<(), String> {
        if self.phase != RunPhase::Combat { return Err("진행 중인 전투가 없습니다.".into()); }
        let previous = self.clone();
        self.battle.result = GameResult::Win { winner, reason: ResultReason::RoyalCaptured };
        self.battle.phase = Phase::Finished;
        self.log(if winner == PLAYER { "디버거: 강제 승리" } else { "디버거: 강제 패배" });
        match self.check_finish() {
            Ok(()) => Ok(()),
            Err(error) => { *self = previous; Err(error) }
        }
    }

    pub fn debug_end_run(&mut self) -> Result<(), String> {
        if self.phase == RunPhase::Combat { return Err("전투 중에는 강제 승패로 종료하세요.".into()); }
        self.phase = RunPhase::Defeat;
        self.log("디버거: 원정 종료");
        Ok(())
    }

    pub fn debug_inspect(&self) -> serde_json::Value {
        serde_json::json!({
            "phase": self.phase, "floor": self.floor, "current_node": self.current_node,
            "path": self.path, "completed": self.completed,
            "last_cleared_node": self.last_cleared_node, "last_counted_node": self.last_counted_node,
            "reward_node": self.reward_node, "gold": self.gold,
            "seed": self.seed, "mode": self.mode, "final_battle": self.final_battle,
            "progress": self.progress,
            "cleared_stages": self.cleared_stages, "bosses_defeated": self.bosses_defeated,
            "cards_burned": self.cards_burned, "ammo": self.ammo,
            "reinforcement_used": self.reinforcement_used,
            "hunger_protection_used": self.hunger_protection_used,
            "event": {"active": self.event_state.active, "last": self.event_state.last,
                "ambushed": self.event_state.ambushed,
                "ambush_combat": self.event_state.ambush_combat,
                "next_battle_penalty": self.event_state.next_battle_penalty,
                "hidden_outcome": self.event_state.hidden_outcome,
                "selected_burns": self.event_state.selected_burns},
            "reward_choice": self.reward_choice, "upgrade_claimed": self.upgrade_claimed,
            "shop_offers": self.shop_offers, "shop_bought": self.shop_bought,
            "next_card_id": self.next_card_id, "boss_sequence_step": self.boss_sequence_step,
            "endless_scaling": self.endless_scaling,
            "rewards": self.snapshot().rewards,
            "roster": self.battle.outcome().iter().map(|p| serde_json::json!({
                "id": p.id, "kind": p.kind, "alive": p.position.is_some(),
                "origin": cell(p.origin), "position": p.position.map(cell),
                "upgrades": p.upgrades, "persistent": p.persistent
            })).collect::<Vec<_>>(),
            "cards": {"mana": self.cards.mana, "draw_locked": self.cards.draw_locked,
                "cost_overrides": self.cards.cost_overrides,
                "hand": self.cards.hand, "draw": self.cards.draw,
                "discard": self.cards.discard, "exhausted": self.cards.exhausted},
            "deck": self.deck, "relics": self.relics,
            "round": self.round, "actions_left": self.battle.actions_left,
            "battle_result": format!("{:?}", self.battle.result),
            "battle": self.battle.debug_state(),
            "log": self.log,
            "trace_log": self.debug_log,
            "trace_events": self.debug_events,
        })
    }
}
