use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Left,
    Right,
    SoftDrop,
    Clockwise,
    Counterclockwise,
    HalfTurn,
    Hold,
    HardDrop,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    pub at: u64,
    pub action: Action,
    pub pressed: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputState {
    pub left: bool,
    pub right: bool,
    pub soft_drop: bool,
    pub direction: i32,
    pub repeat_at: Option<u64>,
    pub soft_drop_at: Option<u64>,
    pub charged: bool,
    pub rotations: [bool; 3],
    pub hold: bool,
    pub buffered_rotation: u8,
    pub buffered_hold: bool,
}
