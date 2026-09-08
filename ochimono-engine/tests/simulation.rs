use ochimono_engine::piece::Piece;
use ochimono_engine::{Action, Game, Handling, Input, Mode, Rules};

fn handling() -> Handling {
    Handling {
        das: 6_000,
        arr: 1_000,
        dcd: 0,
        soft_drop_interval: 0,
        cancel_das_on_direction_change: true,
        ..Handling::default()
    }
}

fn game() -> Game {
    Game::new(42, Rules::solo(Mode::Zen, false), handling()).unwrap()
}

#[test]
fn pieces_spawn_above_the_skyline_and_the_view_preserves_upper_rows() {
    for piece in Piece::ALL {
        let mut state = game().snapshot();
        state.queue[0] = piece;
        let mut game = Game::from_snapshot(state).unwrap();
        press(&mut game, 0, Action::HardDrop);
        let state = game.snapshot();
        assert_eq!(state.piece, piece);
        assert!(
            piece
                .cells(state.rotation)
                .iter()
                .all(|(_, y)| state.y + y < 0)
        );
        assert_eq!(game.view().board_top, -20);
        assert_eq!(game.view().board.len(), 40);
    }
}

#[test]
fn locking_above_the_visible_board_keeps_the_cells_and_does_not_end_the_run() {
    let mut state = game().snapshot();
    state.piece = Piece::O;
    state.x = 0;
    state.y = -2;
    state.board[20][0] = Some(Piece::J);
    state.board[20][1] = Some(Piece::J);
    let mut game = Game::from_snapshot(state).unwrap();
    press(&mut game, 0, Action::HardDrop);
    assert!(!game.view().over);
    assert_eq!(game.view().board[18][0], Some(Piece::O));
    assert_eq!(game.view().board[19][1], Some(Piece::O));
}

#[test]
fn complete_rows_in_the_upper_buffer_clear_normally() {
    let mut state = game().snapshot();
    state.piece = Piece::I;
    state.x = 3;
    state.y = -2;
    state.board[19] = [Some(Piece::J); 10];
    for x in 3..7 {
        state.board[19][x] = None;
        state.board[20][x] = Some(Piece::J);
    }
    let mut game = Game::from_snapshot(state).unwrap();
    press(&mut game, 0, Action::HardDrop);
    assert_eq!(game.view().lines, 1);
    assert!(!game.view().over);
    assert!(game.view().board[19].iter().all(Option::is_none));
}

fn press(game: &mut Game, at: u64, action: Action) {
    game.input(Input {
        at,
        action,
        pressed: true,
    })
    .unwrap();
}

fn json(game: &Game) -> serde_json::Value {
    serde_json::to_value(game.snapshot()).unwrap()
}

#[test]
fn bags_contain_each_piece_and_restore_the_generator() {
    let mut game = game();
    let state = game.snapshot();
    let first: Vec<_> = std::iter::once(state.piece)
        .chain(state.queue.iter().copied())
        .take(7)
        .collect();
    for piece in Piece::ALL {
        assert!(first.contains(&piece));
    }
    let mut restored = Game::from_snapshot(serde_json::from_value(json(&game)).unwrap()).unwrap();
    for _ in 0..12 {
        press(&mut game, 0, Action::HardDrop);
        press(&mut restored, 0, Action::HardDrop);
        assert_eq!(json(&game), json(&restored));
    }
}

#[test]
fn render_cadence_does_not_change_timers_or_movement() {
    let mut rules = Rules::solo(Mode::Zen, true);
    rules.gravity_interval = 1_500;
    let mut single = Game::new(17, rules.clone(), handling()).unwrap();
    let mut split = Game::new(17, rules, handling()).unwrap();
    press(&mut single, 0, Action::Right);
    press(&mut split, 0, Action::Right);
    single.advance_to(50_000).unwrap();
    for at in (0..=50_000).step_by(137) {
        split.advance_to(at).unwrap();
    }
    split.advance_to(50_000).unwrap();
    assert_eq!(json(&single), json(&split));
}

#[test]
fn arr_zero_waits_for_das_then_reaches_the_wall() {
    let mut settings = handling();
    settings.arr = 0;
    let mut game = Game::new(42, Rules::solo(Mode::Zen, false), settings).unwrap();
    press(&mut game, 0, Action::Left);
    let x = game.view().x;
    game.advance_to(5_999).unwrap();
    assert_eq!(game.view().x, x);
    game.advance_to(6_000).unwrap();
    assert_eq!(game.view().x, 0);
}

#[test]
fn sonic_drop_does_not_lock_and_lock_delay_has_an_exact_boundary() {
    let mut game = Game::new(42, Rules::solo(Mode::Sprint, true), handling()).unwrap();
    press(&mut game, 0, Action::SoftDrop);
    assert_eq!(game.view().y, game.ghost_y());
    assert_eq!(game.view().placed, 0);
    game.advance_to(29_999).unwrap();
    assert_eq!(game.view().placed, 0);
    game.advance_to(30_000).unwrap();
    assert_eq!(game.view().placed, 1);
}

#[test]
fn hold_is_limited_until_lock_and_undo_restores_the_bag() {
    let mut game = game();
    let original = json(&game);
    press(&mut game, 0, Action::Hold);
    let after_hold = json(&game);
    press(&mut game, 0, Action::Hold);
    assert_eq!(json(&game), after_hold);
    press(&mut game, 0, Action::HardDrop);
    let after_drop = json(&game);
    assert!(!game.view().held);
    assert!(game.undo());
    assert_eq!(json(&game), original);
    assert!(game.redo());
    assert_eq!(json(&game), after_drop);
    assert!(game.undo());
    press(&mut game, 0, Action::Clockwise);
    assert!(!game.redo());
}

#[test]
fn sprint_finishes_on_the_line_clear_tick_and_restores_finished_snapshots() {
    let game = Game::new(1, Rules::solo(Mode::Sprint, true), handling()).unwrap();
    let mut state = game.snapshot();
    state.lines = 36;
    state.piece = Piece::I;
    state.rotation = 1;
    state.x = 2;
    for row in &mut state.board[36..] {
        *row = [Some(Piece::J); 10];
        row[4] = None;
    }
    let mut game = Game::from_snapshot(state).unwrap();
    press(&mut game, 1_234, Action::HardDrop);
    assert_eq!(game.view().lines, 40);
    assert!(game.view().complete);
    assert_eq!(game.view().time, 1_234);
    game.advance_to(10_000).unwrap();
    assert_eq!(game.view().time, 1_234);
    assert!(Game::from_snapshot(game.snapshot()).is_ok());
    assert!(!game.undo());
}

#[test]
fn malformed_snapshots_cannot_install_nonadvancing_timers() {
    let initial = game().snapshot();
    let mut state = initial.clone();
    state.gravity_at = Some(0);
    assert!(Game::from_snapshot(state).is_err());
    let mut state = initial.clone();
    state.input.soft_drop = true;
    state.input.soft_drop_at = Some(0);
    assert!(Game::from_snapshot(state).is_err());
    let mut state = initial;
    state.input.charged = true;
    assert!(Game::from_snapshot(state).is_err());
}

#[test]
fn srs_plus_i_floor_kick_uses_upward_cartesian_offset() {
    let mut state = game().snapshot();
    state.piece = Piece::I;
    state.x = 3;
    state.y = 18;
    let mut game = Game::from_snapshot(state).unwrap();
    press(&mut game, 0, Action::Clockwise);
    let state = game.snapshot();
    assert_eq!((state.rotation, state.x, state.y), (1, 4, 16));
    assert_eq!(state.last_kick, Some(4));
}

#[test]
fn failed_rotation_preserves_position_and_orientation() {
    let mut state = game().snapshot();
    state.piece = Piece::T;
    state.x = 3;
    state.y = 10;
    state.board.fill([Some(Piece::J); 10]);
    for (dx, dy) in state.piece.cells(0) {
        state.board[(state.y + dy + 20) as usize][(state.x + dx) as usize] = None;
    }
    let mut game = Game::from_snapshot(state).unwrap();
    press(&mut game, 0, Action::Clockwise);
    assert_eq!(
        (game.snapshot().rotation, game.view().x, game.view().y),
        (0, 3, 10)
    );
}

#[test]
fn releasing_inputs_stops_pending_repeats() {
    let mut game = game();
    press(&mut game, 0, Action::Left);
    let x = game.view().x;
    game.release_inputs();
    game.advance_to(10_000).unwrap();
    assert_eq!(game.view().x, x);
}

#[test]
fn grounded_moves_lock_on_the_configured_reset_limit() {
    let game = Game::new(42, Rules::solo(Mode::Sprint, true), handling()).unwrap();
    let mut state = game.snapshot();
    state.piece = Piece::O;
    state.x = 3;
    state.y = 18;
    state.lowest_y = 18;
    let mut game = Game::from_snapshot(state).unwrap();
    for index in 0..15 {
        let action = if index % 2 == 0 {
            Action::Left
        } else {
            Action::Right
        };
        press(&mut game, index * 100, action);
        assert_eq!(game.view().placed, if index == 14 { 1 } else { 0 });
        game.input(Input {
            at: index * 100,
            action,
            pressed: false,
        })
        .unwrap();
    }
}

#[test]
fn half_turn_at_the_floor_uses_the_first_valid_kick() {
    let mut state = game().snapshot();
    state.piece = Piece::T;
    state.x = 3;
    state.y = 18;
    let mut game = Game::from_snapshot(state).unwrap();
    press(&mut game, 0, Action::HalfTurn);
    let state = game.snapshot();
    assert_eq!(
        (state.rotation, state.x, state.y, state.last_kick),
        (2, 3, 17, Some(1))
    );
}
