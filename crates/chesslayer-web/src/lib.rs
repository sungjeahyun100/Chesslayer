use chesslayer::game::{Command, Game};
use wasm_bindgen::prelude::*;

/// The only browser state owner. JavaScript sends intentions and renders snapshots.
#[wasm_bindgen]
pub struct WebGame {
    game: Game,
}
#[wasm_bindgen]
impl WebGame {
    #[wasm_bindgen(constructor)]
    pub fn new(character: &str, seed: u32) -> Result<WebGame, JsValue> {
        Ok(Self {
            game: Game::new(character, seed).map_err(|e| JsValue::from_str(&e))?,
        })
    }
    pub fn snapshot(&self) -> String {
        serde_json::to_string(&self.game.snapshot()).expect("snapshot is serializable")
    }
    pub fn dispatch(&mut self, command: &str) -> Result<String, JsValue> {
        let command: Command = serde_json::from_str(command)
            .map_err(|_| JsValue::from_str("잘못된 게임 명령입니다."))?;
        self.game
            .dispatch(command)
            .map_err(|e| JsValue::from_str(&e))?;
        Ok(self.snapshot())
    }
}
#[wasm_bindgen]
pub fn catalog() -> String {
    serde_json::to_string(&chesslayer::content::characters()).expect("catalog is serializable")
}
