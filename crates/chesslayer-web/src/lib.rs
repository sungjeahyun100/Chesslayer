use chesslayer::game::{Command, Game};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SavedRun {
    version: u32,
    squad: String,
    seed: u32,
    floor: u8,
    commands: Vec<Command>,
}

const SAVE_VERSION: u32 = 15;

/// The only browser state owner. JavaScript sends intentions and renders snapshots.
#[wasm_bindgen]
pub struct WebGame {
    game: Game,
    save: SavedRun,
}
#[wasm_bindgen]
impl WebGame {
    #[wasm_bindgen(constructor)]
    pub fn new(squad: &str, seed: u32) -> Result<WebGame, JsValue> {
        Ok(Self {
            game: Game::new(squad, seed).map_err(|e| JsValue::from_str(&e))?,
            save: SavedRun {
                version: SAVE_VERSION,
                squad: squad.into(),
                seed,
                floor: 1,
                commands: vec![],
            },
        })
    }
    pub fn start(squad: &str, seed: u32, floor: u8, cleared: u8) -> Result<WebGame, JsValue> {
        let choice = chesslayer::content::squads()
            .into_iter()
            .find(|s| s.id == squad)
            .ok_or_else(|| JsValue::from_str("부대를 선택하세요."))?;
        let unlocked = |f: u8| f == 0 || (f <= 4 && cleared & (1 << (f - 1)) != 0);
        if !unlocked(choice.unlock_floor)
            || !unlocked(floor.saturating_sub(1))
            || !(1..=4).contains(&floor)
        {
            return Err(JsValue::from_str("이전 층 보스를 격파해 해금하세요."));
        }
        Ok(Self {
            game: Game::new_on_floor(squad, seed, floor).map_err(|e| JsValue::from_str(&e))?,
            save: SavedRun {
                version: SAVE_VERSION,
                squad: squad.into(),
                seed,
                floor,
                commands: vec![],
            },
        })
    }
    pub fn snapshot(&self) -> String {
        serde_json::to_string(&self.game.snapshot()).expect("snapshot is serializable")
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
        if header.get("version").and_then(|v| v.as_u64()) != Some(u64::from(SAVE_VERSION)) {
            return Err(JsValue::from_str("이전 게임 규칙으로 저장된 원정입니다. 이어갈 수 없어 새 원정을 시작해야 합니다. 기존 기록은 새 원정을 시작하기 전까지 보존됩니다."));
        }
        let save: SavedRun = serde_json::from_str(saved)
            .map_err(|_| JsValue::from_str("저장된 원정을 읽을 수 없습니다."))?;
        let mut game = Game::new_on_floor(&save.squad, save.seed, save.floor)
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
    serde_json::to_string(&chesslayer::content::squads()).expect("catalog is serializable")
}
