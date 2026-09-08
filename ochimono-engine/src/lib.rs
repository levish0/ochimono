//! Deterministic falling-block simulation shared by native and WebAssembly clients.
pub mod game;
pub mod input;
pub mod piece;
pub mod rules;
pub mod state;

#[cfg(feature = "wasm")]
mod wasm;

pub use game::Game;
pub use input::{Action, Input};
pub use rules::{BufferMode, Handling, Mode, Rules, TICKS_PER_SECOND};
pub use state::{Snapshot, View};
