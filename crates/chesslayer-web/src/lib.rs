use chesslayer::game::{Command, Game};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SavedRun {
    version: u32,
    character: String,
    seed: u32,
    commands: Vec<Command>,
}

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
        Ok(Self {
            game: Game::new(character, seed).map_err(|e| JsValue::from_str(&e))?,
            save: SavedRun {
                version: 1,
                character: character.into(),
                seed,
                commands: vec![],
            },
        })
    }
    pub fn snapshot(&self) -> String {
        serde_json::to_string(&self.game.snapshot()).expect("snapshot is serializable")
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
        let save: SavedRun = serde_json::from_str(saved)
            .map_err(|_| JsValue::from_str("저장된 원정을 읽을 수 없습니다."))?;
        if save.version != 1 {
            return Err(JsValue::from_str("지원하지 않는 저장 버전입니다."));
        }
        let mut game = Game::new(&save.character, save.seed).map_err(|e| JsValue::from_str(&e))?;
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
