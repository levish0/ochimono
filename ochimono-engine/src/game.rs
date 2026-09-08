use crate::{
    input::{Action, Input, InputState},
    piece::{Piece, kicks},
    rules::{BufferMode, Handling, Mode, Rules, TICKS_PER_SECOND},
    state::{HEIGHT, HIDDEN, Snapshot, View, WIDTH},
};
use rand_chacha::ChaCha8Rng;
use rand_core::{Rng, SeedableRng};

pub struct Game {
    state: Snapshot,
    checkpoint: Snapshot,
    history: Vec<Snapshot>,
    future: Vec<Snapshot>,
}

impl Game {
    pub fn new(seed: u64, rules: Rules, handling: Handling) -> Result<Self, String> {
        rules.validate()?;
        handling.validate()?;
        let state = Snapshot {
            format_version: 2,
            rules,
            handling,
            rng: ChaCha8Rng::seed_from_u64(seed),
            board: vec![[None; WIDTH]; HEIGHT],
            queue: Vec::new(),
            piece: Piece::T,
            rotation: 0,
            x: 3,
            y: -2,
            hold: None,
            held: false,
            lines: 0,
            placed: 0,
            over: false,
            complete: false,
            time: 0,
            gravity_at: None,
            lock_at: None,
            spawn_at: None,
            hard_drop_after: 0,
            lock_resets: 0,
            lowest_y: -2,
            last_kick: None,
            input: InputState::default(),
        };
        let mut game = Self {
            checkpoint: state.clone(),
            state,
            history: Vec::new(),
            future: Vec::new(),
        };
        game.spawn(None);
        game.checkpoint = game.state.clone();
        Ok(game)
    }

    pub fn snapshot(&self) -> Snapshot {
        self.state.clone()
    }

    pub fn from_snapshot(state: Snapshot) -> Result<Self, String> {
        state.rules.validate()?;
        state.handling.validate()?;
        if state.format_version != 2
            || state.board.len() != HEIGHT
            || !(7..=14).contains(&state.queue.len())
            || state.rotation > 3
            || !(-4..=10).contains(&state.x)
            || !(-HIDDEN..20).contains(&state.y)
            || state.time > 9_000_000_000_000
            || state.lock_resets > state.rules.lock_reset_limit
            || state.placed > 1_000_000
            || state.lines > 4_000_000
            || !(-HIDDEN..20).contains(&state.lowest_y)
            || state.last_kick.is_some_and(|index| index >= 6)
            || !(-1..=1).contains(&state.input.direction)
            || state.input.buffered_rotation > 3
            || state.hard_drop_after > state.time + TICKS_PER_SECOND
            || (state.spawn_at.is_some() && (state.gravity_at.is_some() || state.lock_at.is_some()))
            || (state.gravity_at.is_some() && state.rules.gravity_interval == 0)
            || (state.input.soft_drop_at.is_some()
                && (!state.input.soft_drop || state.handling.soft_drop_interval == 0))
            || (state.input.direction == -1 && !state.input.left)
            || (state.input.direction == 1 && !state.input.right)
            || (state.input.direction == 0
                && (state.input.repeat_at.is_some() || state.input.charged))
            || [
                state.gravity_at,
                state.lock_at,
                state.spawn_at,
                state.input.repeat_at,
                state.input.soft_drop_at,
            ]
            .into_iter()
            .flatten()
            .any(|at| at < state.time || at > state.time + TICKS_PER_SECOND * 3600)
        {
            return Err("Invalid snapshot".into());
        }
        let game = Self {
            checkpoint: state.clone(),
            state,
            history: Vec::new(),
            future: Vec::new(),
        };
        if !game.state.over
            && !game.state.complete
            && game.state.spawn_at.is_none()
            && !game.fits(game.state.x, game.state.y, game.state.rotation)
        {
            return Err("Active piece overlaps board".into());
        }
        Ok(game)
    }

    pub fn view(&self) -> View {
        let s = &self.state;
        let mut matrix = vec![vec![0; s.piece.size()]; s.piece.size()];
        for (x, y) in s.piece.cells(s.rotation) {
            matrix[y as usize][x as usize] = 1;
        }
        View {
            active: s.spawn_at.is_none(),
            board: s.board.clone(),
            board_top: -HIDDEN,
            queue: s.queue.clone(),
            piece: s.piece,
            matrix,
            x: s.x,
            y: s.y,
            hold: s.hold,
            held: s.held,
            lines: s.lines,
            placed: s.placed,
            over: s.over,
            complete: s.complete,
            time: s.time,
            ghost_y: self.ghost_y(),
        }
    }

    fn fits(&self, x: i32, y: i32, rotation: u8) -> bool {
        self.state.piece.cells(rotation).iter().all(|(dx, dy)| {
            let (x, y) = (x + dx, y + dy + HIDDEN);
            (0..WIDTH as i32).contains(&x)
                && (0..HEIGHT as i32).contains(&y)
                && self.state.board[y as usize][x as usize].is_none()
        })
    }

    fn replenish(&mut self) {
        while self.state.queue.len() < 7 {
            let mut bag = Piece::ALL;
            for i in (1..7).rev() {
                // Rejection sampling avoids modulo bias and is identical on native and wasm32.
                let bound = (i + 1) as u32;
                let threshold = bound.wrapping_neg() % bound;
                let value = loop {
                    let value = self.state.rng.next_u32();
                    if value >= threshold {
                        break value;
                    }
                };
                bag.swap(i, (value % bound) as usize);
            }
            self.state.queue.extend(bag);
        }
    }

    fn spawn(&mut self, held: Option<Piece>) {
        self.prepare_spawn(held);
        self.finish_spawn();
    }

    fn prepare_spawn(&mut self, held: Option<Piece>) {
        self.state.spawn_at = None;
        self.replenish();
        self.state.piece = held.unwrap_or_else(|| self.state.queue.remove(0));
        self.replenish();
        self.state.rotation = 0;
        self.state.x = (WIDTH - self.state.piece.size()) as i32 / 2;
        self.state.y = -2;
        self.state.lowest_y = -2;
        self.state.lock_resets = 0;
        self.state.lock_at = None;
        self.state.last_kick = None;
        self.state.gravity_at = (self.state.rules.gravity_interval > 0)
            .then_some(self.state.time + self.state.rules.gravity_interval);
    }

    fn finish_spawn(&mut self) {
        self.state.over = !self.fits(self.state.x, self.state.y, self.state.rotation);
        self.cut_das();
        self.grounded(false);
    }

    fn spawn_buffered(&mut self) {
        self.prepare_spawn(None);
        let hold = match self.state.handling.ihs {
            BufferMode::Off => false,
            BufferMode::Hold => self.state.input.hold,
            BufferMode::Tap => self.state.input.buffered_hold,
        };
        if hold {
            let previous = self.state.hold.replace(self.state.piece);
            self.prepare_spawn(previous);
            self.state.held = true;
        }
        let rotation = match self.state.handling.irs {
            BufferMode::Off => 0,
            BufferMode::Hold => {
                self.state
                    .input
                    .rotations
                    .iter()
                    .zip([1, 3, 2])
                    .filter_map(|(held, amount)| held.then_some(amount))
                    .sum::<u8>()
                    % 4
            }
            BufferMode::Tap => self.state.input.buffered_rotation,
        };
        self.state.input.buffered_rotation = 0;
        self.state.input.buffered_hold = false;
        if rotation != 0 {
            self.try_rotate(rotation);
        }
        self.finish_spawn();
        self.checkpoint = self.state.clone();
    }

    pub fn ghost_y(&self) -> i32 {
        let mut y = self.state.y;
        if !self.state.over {
            while self.fits(self.state.x, y + 1, self.state.rotation) {
                y += 1;
            }
        }
        y
    }

    fn grounded(&mut self, reset: bool) {
        if self.state.y > self.state.lowest_y {
            self.state.lowest_y = self.state.y;
            self.state.lock_resets = 0;
            self.state.lock_at = None;
        }
        if !self.state.rules.automatic_lock
            || self.state.over
            || self.state.complete
            || self.state.spawn_at.is_some()
        {
            self.state.lock_at = None;
            return;
        }
        if self.fits(self.state.x, self.state.y + 1, self.state.rotation) {
            self.state.lock_at = None;
            return;
        }
        if reset && self.state.lock_resets < self.state.rules.lock_reset_limit {
            self.state.lock_resets += 1;
            self.state.lock_at = Some(self.state.time + self.state.rules.lock_delay);
        }
        if self.state.lock_at.is_none() {
            self.state.lock_at = Some(self.state.time + self.state.rules.lock_delay);
        }
        if self.state.lock_resets >= self.state.rules.lock_reset_limit {
            self.lock(true);
        }
    }

    fn move_piece(&mut self, dx: i32, dy: i32) -> bool {
        if self.state.over
            || self.state.complete
            || self.state.spawn_at.is_some()
            || !self.fits(self.state.x + dx, self.state.y + dy, self.state.rotation)
        {
            return false;
        }
        let grounded = !self.fits(self.state.x, self.state.y + 1, self.state.rotation);
        self.state.x += dx;
        self.state.y += dy;
        self.state.last_kick = None;
        self.grounded(grounded && dx != 0);
        true
    }

    fn rotate(&mut self, amount: u8) {
        let grounded = !self.fits(self.state.x, self.state.y + 1, self.state.rotation);
        if self.try_rotate(amount) {
            self.cut_das();
            self.grounded(grounded);
        }
    }

    fn try_rotate(&mut self, amount: u8) -> bool {
        let to = (self.state.rotation + amount) % 4;
        for (index, (dx, dy)) in kicks(self.state.piece, self.state.rotation, to)
            .iter()
            .enumerate()
        {
            if self.fits(self.state.x + dx, self.state.y - dy, to) {
                self.state.x += dx;
                self.state.y -= dy;
                self.state.rotation = to;
                self.state.last_kick = Some(index);
                return true;
            }
        }
        false
    }

    fn cut_das(&mut self) {
        if self.state.input.direction != 0 && self.state.handling.dcd > 0 {
            self.state.input.repeat_at = Some(
                self.state
                    .input
                    .repeat_at
                    .unwrap_or(self.state.time)
                    .max(self.state.time)
                    + self.state.handling.dcd,
            );
        }
    }

    fn lock(&mut self, automatic: bool) {
        if self.state.over || self.state.complete {
            return;
        }
        if self.state.rules.mode == Mode::Zen {
            self.history.push(self.checkpoint.clone());
            if self.history.len() > 100 {
                self.history.remove(0);
            }
        }
        for (dx, dy) in self.state.piece.cells(self.state.rotation) {
            self.state.board[(self.state.y + dy + HIDDEN) as usize][(self.state.x + dx) as usize] =
                Some(self.state.piece);
        }
        let before = self.state.board.len();
        self.state
            .board
            .retain(|row| row.iter().any(Option::is_none));
        let cleared = before - self.state.board.len();
        for _ in 0..cleared {
            self.state.board.insert(0, [None; WIDTH]);
        }
        self.state.lines += cleared as u32;
        self.state.placed += 1;
        self.state.held = false;
        if automatic {
            self.state.hard_drop_after = self.state.time + self.state.handling.safe_lock_delay;
        }
        if self.state.rules.mode == Mode::Sprint && self.state.lines >= 40 {
            self.state.complete = true;
            self.state.lock_at = None;
        } else if self.state.rules.entry_delay > 0 {
            self.state.spawn_at = Some(self.state.time + self.state.rules.entry_delay);
            self.state.gravity_at = None;
            self.state.lock_at = None;
        } else {
            self.spawn_buffered();
        }
        self.checkpoint = self.state.clone();
    }

    fn instant_movement(&mut self) {
        if self.state.spawn_at.is_some() || self.state.over || self.state.complete {
            return;
        }
        let placed = self.state.placed;
        if self.state.handling.prefer_soft_drop {
            self.sonic_drop();
        }
        if self.state.input.direction != 0
            && self.state.input.charged
            && self.state.handling.arr == 0
            && self.state.input.repeat_at.is_none()
        {
            while placed == self.state.placed && self.move_piece(self.state.input.direction, 0) {}
        }
        if placed == self.state.placed {
            self.sonic_drop();
        }
    }

    fn sonic_drop(&mut self) {
        if self.state.spawn_at.is_none()
            && !self.state.over
            && !self.state.complete
            && self.state.input.soft_drop
            && self.state.handling.soft_drop_interval == 0
        {
            self.state.y = self.ghost_y();
            self.grounded(false);
        }
    }

    pub fn input(&mut self, input: Input) -> Result<(), String> {
        self.advance_to(input.at)?;
        if self.state.over || self.state.complete {
            return Ok(());
        }
        if input.pressed {
            self.future.clear();
        }
        let rotation = match input.action {
            Action::Clockwise => Some((0, 1)),
            Action::Counterclockwise => Some((1, 3)),
            Action::HalfTurn => Some((2, 2)),
            _ => None,
        };
        if let Some((index, amount)) = rotation {
            self.state.input.rotations[index] = input.pressed;
            if self.state.spawn_at.is_some()
                && input.pressed
                && self.state.handling.irs == BufferMode::Tap
            {
                self.state.input.buffered_rotation =
                    (self.state.input.buffered_rotation + amount) % 4;
            }
        }
        if input.action == Action::Hold {
            self.state.input.hold = input.pressed;
            if self.state.spawn_at.is_some()
                && input.pressed
                && self.state.handling.ihs == BufferMode::Tap
            {
                self.state.input.buffered_hold = true;
            }
        }
        match input.action {
            Action::Left | Action::Right => {
                let dir = if input.action == Action::Left { -1 } else { 1 };
                let previous = if dir == -1 {
                    &mut self.state.input.left
                } else {
                    &mut self.state.input.right
                };
                if *previous == input.pressed {
                    return Ok(());
                }
                *previous = input.pressed;
                if input.pressed || self.state.input.direction == dir {
                    self.state.input.direction = if input.pressed {
                        dir
                    } else if (dir == -1 && self.state.input.right)
                        || (dir == 1 && self.state.input.left)
                    {
                        -dir
                    } else {
                        0
                    };
                    if self.state.input.direction == 0 {
                        self.state.input.charged = false;
                        self.state.input.repeat_at = None;
                    } else if self.state.handling.cancel_das_on_direction_change
                        || (!self.state.input.charged && self.state.input.repeat_at.is_none())
                    {
                        self.state.input.charged = false;
                        self.state.input.repeat_at =
                            Some(self.state.time + self.state.handling.das);
                    }
                    if input.pressed {
                        self.move_piece(dir, 0);
                    }
                }
            }
            Action::SoftDrop => {
                if self.state.input.soft_drop == input.pressed {
                    return Ok(());
                }
                self.state.input.soft_drop = input.pressed;
                self.state.input.soft_drop_at = (input.pressed
                    && self.state.handling.soft_drop_interval > 0)
                    .then_some(self.state.time + self.state.handling.soft_drop_interval);
                if input.pressed && self.state.handling.soft_drop_interval > 0 {
                    self.move_piece(0, 1);
                }
            }
            _ if !input.pressed => {}
            _ if self.state.spawn_at.is_some() => {}
            Action::Clockwise => self.rotate(1),
            Action::Counterclockwise => self.rotate(3),
            Action::HalfTurn => self.rotate(2),
            Action::Hold if !self.state.held => {
                let previous = self.state.hold.replace(self.state.piece);
                self.spawn(previous);
                self.state.held = true;
            }
            Action::Hold => {}
            Action::HardDrop => {
                if self.state.time >= self.state.hard_drop_after {
                    self.state.y = self.ghost_y();
                    self.lock(false);
                }
            }
        }
        self.instant_movement();
        self.advance_to(self.state.time)
    }

    /// Timers run before inputs at the same tick. Render calls never change this ordering.
    pub fn advance_to(&mut self, target: u64) -> Result<(), String> {
        if target < self.state.time
            || target - self.state.time > TICKS_PER_SECOND * 3600
            || target > 9_000_000_000_000
        {
            return Err("Invalid simulation time".into());
        }
        while !self.state.over && !self.state.complete {
            let next = [
                self.state.input.repeat_at,
                self.state.input.soft_drop_at,
                self.state.gravity_at,
                self.state.lock_at,
                self.state.spawn_at,
            ]
            .into_iter()
            .flatten()
            .min();
            let Some(next) = next.filter(|next| *next <= target) else {
                break;
            };
            self.state.time = next;
            if self.state.spawn_at == Some(next) {
                self.spawn_buffered();
            }
            if self.state.handling.prefer_soft_drop && self.state.input.soft_drop_at == Some(next) {
                self.state.input.soft_drop_at = Some(next + self.state.handling.soft_drop_interval);
                self.move_piece(0, 1);
            }
            if self.state.handling.prefer_soft_drop {
                self.instant_movement();
            }
            if self.state.input.repeat_at == Some(next) {
                self.state.input.charged = true;
                self.state.input.repeat_at =
                    (self.state.handling.arr > 0).then_some(next + self.state.handling.arr);
                if self.state.handling.arr > 0 {
                    self.move_piece(self.state.input.direction, 0);
                }
            }
            if self.state.input.soft_drop_at == Some(next) {
                self.state.input.soft_drop_at = Some(next + self.state.handling.soft_drop_interval);
                self.move_piece(0, 1);
            }
            if self.state.gravity_at == Some(next) {
                self.state.gravity_at = Some(next + self.state.rules.gravity_interval);
                self.move_piece(0, 1);
            }
            self.instant_movement();
            if self.state.lock_at == Some(next) {
                self.lock(true);
            }
        }
        if !self.state.over && !self.state.complete {
            self.state.time = target;
        }
        Ok(())
    }

    pub fn release_inputs(&mut self) {
        self.state.input = InputState::default();
    }

    pub fn configure(&mut self, rules: Rules, handling: Handling) -> Result<(), String> {
        rules.validate()?;
        handling.validate()?;
        if rules.mode != self.state.rules.mode {
            return Err("Cannot change mode during a run".into());
        }
        self.state.handling = handling;
        if rules != self.state.rules {
            self.state.rules = rules;
            self.state.gravity_at = (self.state.rules.gravity_interval > 0
                && self.state.spawn_at.is_none())
            .then_some(self.state.time + self.state.rules.gravity_interval);
            self.state.lock_at = None;
        }
        self.release_inputs();
        self.grounded(false);
        Ok(())
    }

    pub fn undo(&mut self) -> bool {
        if self.state.rules.mode != Mode::Zen {
            return false;
        }
        let Some(previous) = self.history.pop() else {
            return false;
        };
        self.future.push(self.state.clone());
        self.state = previous;
        self.checkpoint = self.state.clone();
        true
    }
    pub fn redo(&mut self) -> bool {
        let Some(next) = self.future.pop() else {
            return false;
        };
        self.history.push(self.state.clone());
        self.state = next;
        self.checkpoint = self.state.clone();
        true
    }
}
