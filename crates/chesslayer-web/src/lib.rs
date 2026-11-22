use chesslayer::game::{Command, Game, Progress, RunMode, SimulatorConfig};
use chesslayer::Position;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SavedRun {
    version: u32,
    character: String,
    seed: u32,
    floor: u8,
    #[serde(default)]
    mode: RunMode,
    progress: Progress,
    commands: Vec<Command>,
    /// Commands recorded before the dictator gained separate movement and
    /// weapon actions. Keep this boundary when appending new commands so a
    /// migrated run can always reproduce its original history.
    #[serde(default)]
    legacy_boss_commands: usize,
    /// Commands saved before the card and event content update.
    #[serde(default)]
    legacy_content_commands: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    simulator: Option<SimulatorConfig>,
}

const SAVE_VERSION: u32 = 24;

/// The only browser state owner. JavaScript sends intentions and renders snapshots.
#[wasm_bindgen]
pub struct WebGame {
    game: Game,
    save: SavedRun,
    developer_mode: bool,
}
#[wasm_bindgen]
impl WebGame {
    #[wasm_bindgen(constructor)]
    pub fn new(character: &str, seed: u32) -> Result<WebGame, JsValue> {
        Self::start_run(
            character,
            seed,
            1,
            &serde_json::to_string(&Progress::default()).unwrap(),
        )
    }
    pub fn start_run(
        character: &str,
        seed: u32,
        floor: u8,
        progress: &str,
    ) -> Result<WebGame, JsValue> {
        Self::start_run_with_mode(character, seed, floor, progress, "campaign")
    }
    pub fn start_run_with_mode(
        character: &str,
        seed: u32,
        floor: u8,
        progress: &str,
        mode: &str,
    ) -> Result<WebGame, JsValue> {
        let mode = match mode {
            "campaign" => RunMode::Campaign,
            "endless" => RunMode::Endless,
            _ => return Err(JsValue::from_str("존재하지 않는 원정 모드입니다.")),
        };
        let progress: Progress = serde_json::from_str(progress)
            .map_err(|_| JsValue::from_str("해금 정보를 읽을 수 없습니다."))?;
        let game = Game::new_run_with_mode(character, seed, floor, progress.clone(), mode)
            .map_err(|e| JsValue::from_str(&e))?;
        Ok(Self {
            game,
            developer_mode: false,
            save: SavedRun {
                version: SAVE_VERSION,
                character: character.into(),
                seed,
                floor,
                mode,
                progress,
                commands: vec![],
                legacy_boss_commands: 0,
                legacy_content_commands: 0,
                simulator: None,
            },
        })
    }
    /// Creates a disposable battle using the normal Game battle flow and UI snapshot.
    pub fn start_simulator(config: &str) -> Result<WebGame, JsValue> {
        let config: SimulatorConfig = serde_json::from_str(config)
            .map_err(|e| JsValue::from_str(&format!("시뮬레이터 설정을 읽을 수 없습니다: {e}")))?;
        let game = Game::new_simulator(&config).map_err(|e| JsValue::from_str(&e))?;
        Ok(Self {
            game,
            developer_mode: false,
            save: SavedRun {
                version: SAVE_VERSION,
                character: config.squad.clone(),
                seed: config.seed,
                floor: 1,
                mode: RunMode::Campaign,
                progress: Progress::default(),
                commands: vec![],
                legacy_boss_commands: 0,
                legacy_content_commands: 0,
                simulator: Some(config),
            },
        })
    }
    pub fn snapshot(&self) -> String {
        serde_json::to_string(&self.game.snapshot()).expect("snapshot is serializable")
    }
    pub fn recommend_enemy(&self) -> Result<String, JsValue> {
        let choice = self
            .game
            .enemy_recommendation()
            .map_err(|e| JsValue::from_str(&e))?;
        Ok(serde_json::to_string(&choice).expect("recommendation is serializable"))
    }
    /// Query only: no dispatch, turn advance, or saved command is written.
    pub fn inspect_enemy(&self, cell: usize) -> Result<String, JsValue> {
        let view = self
            .game
            .inspect_enemy(cell)
            .map_err(|e| JsValue::from_str(&e))?;
        Ok(serde_json::to_string(&view).expect("inspection is serializable"))
    }
    /// The engine's exact blast cells for a piece before its action is dispatched.
    /// This is presentation data only and is never written to the saved run.
    pub fn blast_cells(&self, cell: usize) -> Result<String, JsValue> {
        if cell >= 64 {
            return Err(JsValue::from_str("보드 밖의 칸입니다."));
        }
        let at = Position::new((cell % 8) as i32, (cell / 8) as i32);
        let piece = self
            .game
            .battle
            .board
            .piece_at(at)
            .map_err(|_| JsValue::from_str("기물을 조회할 수 없습니다."))?
            .ok_or_else(|| JsValue::from_str("해당 칸에 기물이 없습니다."))?;
        let cells: Vec<usize> = self
            .game
            .battle
            .blast_preview(piece.kind, at)
            .into_iter()
            .map(|p| (p.rank * 8 + p.file) as usize)
            .collect();
        Ok(serde_json::to_string(&cells).expect("blast cells are serializable"))
    }
    /// Target of the indexed legal enemy action used by EnemyStepSelected.
    pub fn enemy_action_target(&self, cell: usize, index: usize) -> Result<usize, JsValue> {
        if cell >= 64 {
            return Err(JsValue::from_str("보드 밖의 칸입니다."));
        }
        let from = Position::new((cell % 8) as i32, (cell / 8) as i32);
        let action = self
            .game
            .battle
            .legal_actions(from)
            .map_err(|_| JsValue::from_str("적 행동을 조회할 수 없습니다."))?
            .into_iter()
            .nth(index)
            .ok_or_else(|| JsValue::from_str("해당 적 행동이 없습니다."))?;
        let step = action.steps().last().expect("legal action has a step");
        let target = if step.move_to == from {
            step.take
        } else {
            step.move_to
        };
        Ok((target.rank * 8 + target.file) as usize)
    }
    pub fn dispatch(&mut self, command: &str) -> Result<String, JsValue> {
        let command: Command = serde_json::from_str(command)
            .map_err(|_| JsValue::from_str("잘못된 게임 명령입니다."))?;
        if matches!(&command, Command::DeveloperSetUpgrade { .. }) && !self.developer_mode {
            return Err(JsValue::from_str(
                "개발자 모드에서만 강화 상태를 바꿀 수 있습니다.",
            ));
        }
        self.game
            .dispatch(command.clone())
            .map_err(|e| JsValue::from_str(&e))?;
        self.save.commands.push(command);
        Ok(self.snapshot())
    }
    pub fn set_developer_mode(&mut self, enabled: bool) {
        self.developer_mode = enabled;
    }
    pub fn save(&self) -> String {
        serde_json::to_string(&self.save).expect("run history is serializable")
    }
    pub fn restore(saved: &str) -> Result<WebGame, JsValue> {
        let (game, save) = restore_saved_run(saved).map_err(|e| JsValue::from_str(&e))?;
        Ok(Self {
            game,
            save,
            developer_mode: false,
        })
    }
}

fn restore_saved_run(saved: &str) -> Result<(Game, SavedRun), String> {
    let header: serde_json::Value =
        serde_json::from_str(saved).map_err(|_| "저장된 원정을 읽을 수 없습니다.")?;
    let version = header.get("version").and_then(|v| v.as_u64());
    if !matches!(version, Some(23) | Some(24)) {
        return Err("이전 게임 규칙으로 저장된 원정입니다. 이어갈 수 없어 새 원정을 시작해야 합니다. 기존 기록은 새 원정을 시작하기 전까지 보존됩니다.".into());
    }
    if header.get("character").and_then(|value| value.as_str()) != Some("avenger") {
        return Err("이 저장 데이터의 부대는 v1.0.0에서 플레이할 수 없습니다. 새 복수귀 원정을 시작하세요. 기존 저장 데이터는 보존됩니다.".into());
    }
    let mut save: SavedRun =
        serde_json::from_str(saved).map_err(|_| "저장된 원정을 읽을 수 없습니다.")?;
    if version == Some(23) {
        save.legacy_content_commands = save.commands.len();
    }
    if save.legacy_boss_commands > save.commands.len()
        || save.legacy_content_commands > save.commands.len()
    {
        return Err("저장된 원정의 진행 기록이 올바르지 않습니다.".into());
    }
    save.version = SAVE_VERSION;
    let mut game = if let Some(config) = &save.simulator {
        Game::new_simulator(config)?
    } else {
        Game::new_run_with_mode(
            &save.character,
            save.seed,
            save.floor,
            save.progress.clone(),
            save.mode,
        )?
    };
    for (index, command) in save.commands.iter().enumerate() {
        game.legacy_content = index < save.legacy_content_commands;
        let result = if index < save.legacy_boss_commands {
            game.replay_legacy_command(command.clone())
        } else {
            game.replay_command(command.clone())
        };
        result.map_err(|_| "저장된 원정의 진행 기록이 올바르지 않습니다.")?;
        if index + 1 == save.legacy_boss_commands {
            game.finish_legacy_replay();
        }
    }
    game.legacy_content = false;
    Ok((game, save))
}
#[wasm_bindgen]
pub fn catalog() -> String {
    serde_json::to_string(&chesslayer::content::characters()).expect("catalog is serializable")
}

#[wasm_bindgen]
pub fn compendium() -> String {
    serde_json::to_string(&chesslayer::content::compendium::catalog())
        .expect("compendium is serializable")
}

#[wasm_bindgen]
pub fn simulator_catalog() -> String {
    use chesslayer::content::{cards, characters, encounters, pieces, ENCOUNTERS};
    let squads: Vec<_> = characters::ALL
        .iter()
        .map(|character| {
            let roster: Vec<_> = characters::roster(character.id)
                .iter()
                .map(|piece| {
                    let definition =
                        pieces::definition(piece.kind).expect("registered starting piece");
                    serde_json::json!({
                        "id": piece.id, "kind": piece.kind, "key": definition.key,
                        "name": definition.name, "cell": piece.origin.rank * 8 + piece.origin.file,
                    })
                })
                .collect();
            serde_json::json!({
                "id": character.id, "deck": cards::starting_deck(character.id)
                    .iter().map(|card| card.kind).collect::<Vec<_>>(),
                "relics": [character.relic], "pieces": roster,
            })
        })
        .collect();
    let battles: Vec<_> = ENCOUNTERS.iter().enumerate().map(|(index, definition)| {
        let lineup = encounters::encounter(index);
        let pieces: Vec<_> = lineup.enemies.iter().map(|piece| {
            let name = pieces::definition(piece.kind).expect("registered enemy piece");
            serde_json::json!({
                "id": piece.id, "kind": piece.kind, "key": name.key, "name": name.name,
                "cell": piece.position.rank * 8 + piece.position.file,
            })
        }).collect();
        serde_json::json!({
            "index": index, "id": definition.id, "name": definition.name,
            "description": definition.description, "floor": definition.floor,
            "final": index == encounters::DICTATOR_BOSS,
            "tier": format!("{:?}", definition.tier).to_lowercase(),
            "pieces": pieces, "royal_id": lineup.royal_id,
            "additional_royal_ids": lineup.additional_royal_ids,
            "random_royal": definition.royal_selection == encounters::RoyalSelection::UniformDeployed,
        })
    }).collect();
    serde_json::json!({"squads": squads, "encounters": battles}).to_string()
}

/// Sandbox state is never saved with an expedition.
#[wasm_bindgen]
pub struct WebSandbox {
    sandbox: chesslayer::sandbox::Sandbox,
}
#[wasm_bindgen]
impl WebSandbox {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<WebSandbox, JsValue> {
        Ok(Self {
            sandbox: chesslayer::sandbox::Sandbox::new().map_err(|e| JsValue::from_str(&e))?,
        })
    }
    pub fn snapshot(&self, selected: Option<usize>) -> String {
        serde_json::to_string(&self.sandbox.snapshot(selected))
            .expect("sandbox snapshot is serializable")
    }
    pub fn place(&mut self, kind: u16, owner: u16, cell: usize) -> Result<(), JsValue> {
        self.sandbox
            .place(kind, owner, cell)
            .map_err(|e| JsValue::from_str(&e))
    }
    pub fn remove(&mut self, cell: usize) -> Result<(), JsValue> {
        self.sandbox.remove(cell).map_err(|e| JsValue::from_str(&e))
    }
    pub fn change_kind(&mut self, cell: usize, kind: u16) -> Result<(), JsValue> {
        self.sandbox
            .change_kind(cell, kind)
            .map_err(|e| JsValue::from_str(&e))
    }
    pub fn progress_effects(&mut self) -> Result<(), JsValue> {
        self.sandbox
            .progress_effects()
            .map_err(|e| JsValue::from_str(&e))
    }
    pub fn act(&mut self, from: usize, index: usize) -> Result<(), JsValue> {
        self.sandbox
            .act(from, index)
            .map_err(|e| JsValue::from_str(&e))
    }
    pub fn detonate(&mut self, from: usize) -> Result<(), JsValue> {
        self.sandbox
            .detonate(from)
            .map_err(|e| JsValue::from_str(&e))
    }
    pub fn resolve_choice(&mut self, target: u32) -> Result<(), JsValue> {
        self.sandbox
            .resolve_choice(target)
            .map_err(|e| JsValue::from_str(&e))
    }
    pub fn clear(&mut self) -> Result<(), JsValue> {
        self.sandbox.clear().map_err(|e| JsValue::from_str(&e))
    }
    pub fn set_relic(&mut self, relic: &str, enabled: bool) -> Result<(), JsValue> {
        let relic = serde_json::from_value(serde_json::Value::String(relic.into()))
            .map_err(|_| JsValue::from_str("등록되지 않은 유물입니다."))?;
        self.sandbox
            .set_relic(relic, enabled)
            .map_err(|e| JsValue::from_str(&e))
    }
    pub fn add_card(&mut self, kind: &str) -> Result<(), JsValue> {
        self.sandbox.add_card(parse_card_kind(kind)?);
        Ok(())
    }
    pub fn remove_card(&mut self, id: u32) -> Result<(), JsValue> {
        self.sandbox
            .remove_card(id)
            .map_err(|e| JsValue::from_str(&e))
    }
    pub fn clear_deck(&mut self) {
        self.sandbox.clear_deck();
    }
    pub fn reset_cards(&mut self) {
        self.sandbox.reset_cards();
    }
    pub fn draw_card(&mut self) -> Result<(), JsValue> {
        self.sandbox.draw_card().map_err(|e| JsValue::from_str(&e))
    }
    pub fn redraw_hand(&mut self) {
        self.sandbox.redraw_hand();
    }
    pub fn add_to_hand(&mut self, kind: &str) -> Result<(), JsValue> {
        self.sandbox.add_to_hand(parse_card_kind(kind)?);
        Ok(())
    }
    pub fn set_mana(&mut self, amount: u32) {
        self.sandbox.set_mana(amount);
    }
    pub fn play_card(&mut self, id: u32, selection: &str) -> Result<(), JsValue> {
        let selection = serde_json::from_str(selection)
            .map_err(|_| JsValue::from_str("카드 대상 선택이 올바르지 않습니다."))?;
        self.sandbox
            .play_card(id, selection)
            .map_err(|e| JsValue::from_str(&e))
    }
}

fn parse_card_kind(kind: &str) -> Result<chesslayer::content::cards::CardKind, JsValue> {
    serde_json::from_value(serde_json::Value::String(kind.into()))
        .map_err(|_| JsValue::from_str("등록되지 않은 카드입니다."))
}

#[cfg(test)]
mod save_tests {
    use super::*;

    #[test]
    fn current_avenger_save_replays_and_removed_squad_is_rejected() {
        let progress = Progress::default();
        let save = SavedRun {
            version: SAVE_VERSION,
            character: "avenger".into(),
            seed: 19,
            floor: 1,
            mode: RunMode::Campaign,
            progress,
            commands: vec![],
            legacy_boss_commands: 0,
            legacy_content_commands: 0,
            simulator: None,
        };
        let encoded = serde_json::to_string(&save).unwrap();
        let (restored, _) = restore_saved_run(&encoded).unwrap();
        assert_eq!(restored.snapshot().squad_id, "avenger");
        let removed = encoded.replace("\"avenger\"", "\"artillery\"");
        assert!(restore_saved_run(&removed)
            .err()
            .unwrap()
            .contains("플레이할 수 없습니다"));
    }
}
