use crate::{
    input::InputState,
    piece::Piece,
    rules::{Handling, Rules},
};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

pub const WIDTH: usize = 10;
pub const HEIGHT: usize = 40;
pub const HIDDEN: i32 = 20;
pub type Board = Vec<[Option<Piece>; WIDTH]>;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub format_version: u32,
    pub rules: Rules,
    pub handling: Handling,
    pub rng: ChaCha8Rng,
    pub board: Board,
    pub queue: Vec<Piece>,
    pub piece: Piece,
    pub rotation: u8,
    pub x: i32,
    pub y: i32,
    pub hold: Option<Piece>,
    pub held: bool,
    pub lines: u32,
    pub placed: u32,
    pub over: bool,
    pub complete: bool,
    pub time: u64,
    pub gravity_at: Option<u64>,
    pub lock_at: Option<u64>,
    pub spawn_at: Option<u64>,
    pub hard_drop_after: u64,
    pub lock_resets: u8,
    pub lowest_y: i32,
    pub last_kick: Option<usize>,
    pub input: InputState,
}

#[derive(Serialize)]
pub struct View {
    pub active: bool,
    pub board: Board,
    /// Y coordinate of the first board row, including the upper buffer.
    pub board_top: i32,
    pub queue: Vec<Piece>,
    pub piece: Piece,
    pub matrix: Vec<Vec<u8>>,
    pub x: i32,
    pub y: i32,
    pub hold: Option<Piece>,
    pub held: bool,
    pub lines: u32,
    pub placed: u32,
    pub over: bool,
    pub complete: bool,
    pub time: u64,
    pub ghost_y: i32,
}
