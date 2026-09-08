use crate::{Game, Handling, Input, Mode, Rules, Snapshot};
use wasm_bindgen::prelude::*;

fn error(error: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&error.to_string())
}

#[wasm_bindgen(js_name = Game)]
pub struct WasmGame(Game);

#[wasm_bindgen(js_class = Game)]
impl WasmGame {
    #[wasm_bindgen(constructor)]
    pub fn new(seed: &str, mode: &str, gravity: bool, handling: &str) -> Result<WasmGame, JsValue> {
        let mode: Mode =
            serde_json::from_value(serde_json::Value::String(mode.into())).map_err(error)?;
        let handling: Handling = serde_json::from_str(handling).map_err(error)?;
        Game::new(
            seed.parse().map_err(error)?,
            Rules::solo(mode, gravity),
            handling,
        )
        .map(Self)
        .map_err(error)
    }

    pub fn input(&mut self, input: &str) -> Result<(), JsValue> {
        let input: Input = serde_json::from_str(input).map_err(error)?;
        self.0.input(input).map_err(error)
    }

    pub fn advance(&mut self, ticks: f64) -> Result<(), JsValue> {
        if !ticks.is_finite() || ticks < 0.0 || ticks.fract() != 0.0 || ticks > 9_000_000_000_000.0
        {
            return Err(error("Invalid simulation time"));
        }
        self.0.advance_to(ticks as u64).map_err(error)
    }

    pub fn view(&self) -> Result<String, JsValue> {
        serde_json::to_string(&self.0.view()).map_err(error)
    }

    pub fn snapshot(&self) -> Result<String, JsValue> {
        serde_json::to_string(&self.0.snapshot()).map_err(error)
    }

    pub fn restore(&mut self, snapshot: &str) -> Result<(), JsValue> {
        let snapshot: Snapshot = serde_json::from_str(snapshot).map_err(error)?;
        self.0 = Game::from_snapshot(snapshot).map_err(error)?;
        Ok(())
    }

    pub fn configure(&mut self, gravity: bool, handling: &str) -> Result<(), JsValue> {
        let mode = self.0.snapshot().rules.mode;
        self.0
            .configure(
                Rules::solo(mode, gravity),
                serde_json::from_str(handling).map_err(error)?,
            )
            .map_err(error)
    }

    pub fn release_inputs(&mut self) {
        self.0.release_inputs();
    }
    pub fn undo(&mut self) -> bool {
        self.0.undo()
    }
    pub fn redo(&mut self) -> bool {
        self.0.redo()
    }
}
