use chesslayer::game::{Command, Game, Progress, RunMode};
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
}

const SAVE_VERSION: u32 = 20;

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
    if !matches!(version, Some(19) | Some(20)) {
        return Err("이전 게임 규칙으로 저장된 원정입니다. 이어갈 수 없어 새 원정을 시작해야 합니다. 기존 기록은 새 원정을 시작하기 전까지 보존됩니다.".into());
    }
    let mut save: SavedRun =
        serde_json::from_str(saved).map_err(|_| "저장된 원정을 읽을 수 없습니다.")?;
    if save.version == 19 {
        save.legacy_boss_commands = save.commands.len();
    }
    if save.legacy_boss_commands > save.commands.len() {
        return Err("저장된 원정의 진행 기록이 올바르지 않습니다.".into());
    }
    save.version = SAVE_VERSION;
    let mut game = Game::new_run_with_mode(
        &save.character,
        save.seed,
        save.floor,
        save.progress.clone(),
        save.mode,
    )?;
    for (index, command) in save.commands.iter().enumerate() {
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
    use chesslayer::{ENEMY, PLAYER};

    // Captured from the version-19 WASM build after two real dictator turns.
    const LEGACY_BOSS_RUN: &str = r#"{
        "version":19,"character":"guardian","seed":19,"floor":0,
        "mode":"campaign",
        "progress":{"highest_floor":1,"unlocked_squads":["guardian"],
            "final_unlocked":false,"final_defeated":false},
        "commands":[{"type":"select_node","id":0},{"type":"end_turn"},
            {"type":"enemy_step_selected","from":4,"index":3},
            {"type":"end_turn"},
            {"type":"enemy_step_selected","from":11,"index":4}]
    }"#;

    #[test]
    fn legacy_boss_history_migrates_and_new_two_action_turns_restore() {
        let (mut game, mut save) = restore_saved_run(LEGACY_BOSS_RUN).unwrap();
        let snapshot = game.snapshot();
        assert_eq!(snapshot.round, 3);
        assert_eq!(snapshot.turn, PLAYER);
        assert_eq!(snapshot.boss.as_ref().unwrap().hp, 5);
        assert_eq!(snapshot.boss.as_ref().unwrap().ammo, 2);
        assert_eq!(
            snapshot.pieces.iter().find(|p| p.id == 100).unwrap().cell,
            18
        );
        assert_eq!(save.version, SAVE_VERSION);
        assert_eq!(save.legacy_boss_commands, 5);

        let command = Command::EndTurn;
        game.dispatch(command.clone()).unwrap();
        save.commands.push(command);
        for action in 0..2 {
            let selected = game.enemy_recommendation().unwrap();
            let command = Command::EnemyStepSelected {
                from: selected.map(|(from, _)| from),
                index: selected.map(|(_, index)| index),
            };
            game.dispatch(command.clone()).unwrap();
            save.commands.push(command);
            assert_eq!(game.battle.turn, if action == 0 { ENEMY } else { PLAYER });
            // In particular, a save between the movement and shot must resume
            // the remaining action instead of replaying a third boss action.
            let encoded = serde_json::to_string(&save).unwrap();
            let (restored, restored_save) = restore_saved_run(&encoded).unwrap();
            assert_eq!(restored_save.legacy_boss_commands, 5);
            assert_eq!(
                serde_json::to_value(restored.snapshot()).unwrap(),
                serde_json::to_value(game.snapshot()).unwrap()
            );
        }
    }

    #[test]
    fn legacy_boss_save_during_target_choices_resumes_planning_after_resolution() {
        // Frontline opens a sequence of target selections without using the
        // player's move. These commands have identical behavior in version 19.
        let saved = r#"{
            "version":19,"character":"artillery","seed":19,"floor":0,
            "progress":{"unlocked_squads":["artillery"]},
            "commands":[{"type":"select_node","id":0},
                {"type":"play_card_selection","id":5,"targets":[1,10040]}]
        }"#;
        let (mut game, mut save) = restore_saved_run(saved).unwrap();
        assert_eq!(game.battle.turn, PLAYER);
        assert!(game.battle.pending_choice().is_some());
        assert_eq!(save.legacy_boss_commands, 2);

        // Select the mortar, move it again, then finish Frontline. Repeated
        // saves must retain the deferred migration until the final selection.
        for target in [1, 10032, 0] {
            let command = Command::ResolveEffect { target };
            game.dispatch(command.clone()).unwrap();
            save.commands.push(command);
            assert_eq!(game.battle.pending_choice().is_some(), target != 0);
            let (restored, _) = restore_saved_run(&serde_json::to_string(&save).unwrap()).unwrap();
            assert_eq!(
                serde_json::to_value(restored.snapshot()).unwrap(),
                serde_json::to_value(game.snapshot()).unwrap()
            );
            game = restored;
        }
        assert!(game.snapshot().boss.unwrap().plan.is_some());
        game.dispatch(Command::EndTurn).unwrap();
        save.commands.push(Command::EndTurn);
        let mut actions = 0;
        while game.battle.turn == ENEMY {
            assert!(actions < 2);
            let (from, index) = game.enemy_recommendation().unwrap().unwrap();
            let command = Command::EnemyStepSelected {
                from: Some(from),
                index: Some(index),
            };
            game.dispatch(command.clone()).unwrap();
            save.commands.push(command);
            actions += 1;
        }
        assert!(actions > 0);
        let (restored, _) = restore_saved_run(&serde_json::to_string(&save).unwrap()).unwrap();
        assert_eq!(
            serde_json::to_value(restored.snapshot()).unwrap(),
            serde_json::to_value(game.snapshot()).unwrap()
        );
    }

    #[test]
    fn legacy_regular_run_keeps_its_state_and_invalid_boundaries_are_rejected() {
        let mut game = WebGame::new("guardian", 19).unwrap();
        let id = game
            .game
            .snapshot()
            .map
            .nodes
            .iter()
            .find(|node| node.available)
            .unwrap()
            .id;
        game.dispatch(&serde_json::to_string(&Command::SelectNode { id }).unwrap())
            .unwrap();
        let mut saved: serde_json::Value = serde_json::from_str(&game.save()).unwrap();
        saved["version"] = serde_json::json!(19);
        saved
            .as_object_mut()
            .unwrap()
            .remove("legacy_boss_commands");
        let (restored, _) = restore_saved_run(&saved.to_string()).unwrap();
        assert_eq!(
            serde_json::to_value(restored.snapshot()).unwrap(),
            serde_json::to_value(game.game.snapshot()).unwrap()
        );

        saved["version"] = serde_json::json!(20);
        saved["legacy_boss_commands"] = serde_json::json!(2);
        assert!(restore_saved_run(&saved.to_string()).is_err());
        saved["version"] = serde_json::json!(18);
        assert!(restore_saved_run(&saved.to_string()).is_err());
    }
}
