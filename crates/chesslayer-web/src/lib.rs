use chesslayer::game::{Command, Game, Progress};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SavedRun {
    version: u32,
    character: String,
    seed: u32,
    floor: u8,
    progress: Progress,
    commands: Vec<Command>,
}

const SAVE_VERSION: u32 = 19;

/// The only browser state owner. JavaScript sends intentions and renders snapshots.
#[wasm_bindgen]
pub struct WebGame {
    game: Game,
    save: SavedRun,
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
        let progress: Progress = serde_json::from_str(progress)
            .map_err(|_| JsValue::from_str("해금 정보를 읽을 수 없습니다."))?;
        let game = Game::new_run(character, seed, floor, progress.clone())
            .map_err(|e| JsValue::from_str(&e))?;
        Ok(Self {
            game,
            save: SavedRun {
                version: SAVE_VERSION,
                character: character.into(),
                seed,
                floor,
                progress,
                commands: vec![],
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
    pub fn dispatch(&mut self, command: &str) -> Result<String, JsValue> {
        let command: Command = serde_json::from_str(command)
            .map_err(|_| JsValue::from_str("잘못된 게임 명령입니다."))?;
        self.game
            .dispatch(command.clone())
            .map_err(|e| JsValue::from_str(&e))?;
        self.save.commands.push(command);
        Ok(self.snapshot())
    }
    pub fn save(&self) -> String {
        serde_json::to_string(&self.save).expect("run history is serializable")
    }
    pub fn restore(saved: &str) -> Result<WebGame, JsValue> {
        let header: serde_json::Value = serde_json::from_str(saved)
            .map_err(|_| JsValue::from_str("저장된 원정을 읽을 수 없습니다."))?;
        let version = header.get("version").and_then(|v| v.as_u64());
        if version != Some(u64::from(SAVE_VERSION)) {
            return Err(JsValue::from_str("이전 게임 규칙으로 저장된 원정입니다. 이어갈 수 없어 새 원정을 시작해야 합니다. 기존 기록은 새 원정을 시작하기 전까지 보존됩니다."));
        }
        let mut save: SavedRun = serde_json::from_str(saved)
            .map_err(|_| JsValue::from_str("저장된 원정을 읽을 수 없습니다."))?;
        save.version = SAVE_VERSION;
        let mut game = Game::new_run(
            &save.character,
            save.seed,
            save.floor,
            save.progress.clone(),
        )
        .map_err(|e| JsValue::from_str(&e))?;
        for command in &save.commands {
            game.dispatch(command.clone())
                .map_err(|_| JsValue::from_str("저장된 원정의 진행 기록이 올바르지 않습니다."))?;
        }
        Ok(Self { game, save })
    }
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
