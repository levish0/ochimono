use serde::{Deserialize, Serialize};

/// One millisecond is 60 ticks; one 60 Hz frame is exactly 1,000 ticks.
pub const TICKS_PER_SECOND: u64 = 60_000;
pub const RULESET_VERSION: &str = "solo-v2";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Zen,
    Sprint,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rules {
    pub version: String,
    pub mode: Mode,
    /// Zero disables gravity without disabling automatic locking.
    pub gravity_interval: u64,
    pub lock_delay: u64,
    pub lock_reset_limit: u8,
    pub automatic_lock: bool,
    /// Delay between locking a piece and spawning the next one.
    pub entry_delay: u64,
}

impl Rules {
    pub fn solo(mode: Mode, gravity: bool) -> Self {
        Self {
            version: RULESET_VERSION.into(),
            mode,
            gravity_interval: if mode == Mode::Sprint || gravity {
                50_000
            } else {
                0
            },
            lock_delay: 30_000,
            lock_reset_limit: 15,
            automatic_lock: mode == Mode::Sprint || gravity,
            entry_delay: 0,
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.version != RULESET_VERSION {
            return Err("Unsupported ruleset version".into());
        }
        if self.gravity_interval > TICKS_PER_SECOND * 3600
            || self.lock_delay == 0
            || self.lock_delay > TICKS_PER_SECOND * 60
            || self.lock_reset_limit == 0
            || self.entry_delay > TICKS_PER_SECOND
        {
            return Err("Invalid timing policy".into());
        }
        if self.mode == Mode::Sprint && self != &Self::solo(Mode::Sprint, true) {
            return Err("Sprint rules are fixed".into());
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BufferMode {
    #[default]
    Off,
    Hold,
    Tap,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Handling {
    pub das: u64,
    pub arr: u64,
    pub dcd: u64,
    /// Zero means sonic drop. A nonzero value is an explicit interval, not an SDF multiplier.
    pub soft_drop_interval: u64,
    pub cancel_das_on_direction_change: bool,
    pub prefer_soft_drop: bool,
    /// Hard drop suppression after automatic locking; zero disables it.
    pub safe_lock_delay: u64,
    pub irs: BufferMode,
    pub ihs: BufferMode,
}
impl Handling {
    pub fn validate(&self) -> Result<(), String> {
        if self.das > TICKS_PER_SECOND
            || self.arr > TICKS_PER_SECOND
            || self.dcd > TICKS_PER_SECOND
            || self.soft_drop_interval > TICKS_PER_SECOND
            || self.safe_lock_delay > TICKS_PER_SECOND
        {
            return Err("Handling interval exceeds one second".into());
        }
        Ok(())
    }
}
