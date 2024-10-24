mod bop;
pub mod engine;
mod features;
mod svg;
mod utils;

use crate::engine::Engine;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub fn create_rpg_engine() -> Engine {
    bop::mount()
}
