use ochimono_engine::piece::Piece;
use ochimono_engine::{Action, BufferMode, Game, Handling, Input, Mode, Rules};

fn game(handling: Handling, entry_delay: u64) -> Game {
    let mut rules = Rules::solo(Mode::Zen, false);
    rules.entry_delay = entry_delay;
    Game::new(42, rules, handling).unwrap()
}

fn input(game: &mut Game, at: u64, action: Action, pressed: bool) {
    game.input(Input {
        at,
        action,
        pressed,
    })
    .unwrap();
}

#[test]
fn direction_change_can_preserve_or_cancel_das_charge() {
    for cancel in [false, true] {
        let mut game = game(
            Handling {
                das: 6000,
                cancel_das_on_direction_change: cancel,
                ..Handling::default()
            },
            0,
        );
        input(&mut game, 0, Action::Left, true);
        game.advance_to(6000).unwrap();
        input(&mut game, 7000, Action::Right, true);
        assert_eq!(game.snapshot().input.charged, !cancel);
        assert_eq!(
            game.snapshot().input.repeat_at,
            if cancel { Some(13000) } else { None }
        );
        input(&mut game, 7100, Action::Right, false);
        assert_eq!(game.snapshot().input.direction, -1);
        input(&mut game, 7200, Action::Left, false);
        assert_eq!(game.snapshot().input.direction, 0);
        assert_eq!(game.snapshot().input.repeat_at, None);
    }
}

#[test]
fn dcd_holds_charged_movement_after_rotation_and_spawn() {
    let mut game = game(
        Handling {
            das: 1000,
            dcd: 2000,
            ..Handling::default()
        },
        0,
    );
    input(&mut game, 0, Action::Left, true);
    game.advance_to(1000).unwrap();
    input(&mut game, 1000, Action::Clockwise, true);
    assert_eq!(game.snapshot().input.repeat_at, Some(3000));
    game.advance_to(3000).unwrap();
    input(&mut game, 3000, Action::HardDrop, true);
    assert_eq!(game.snapshot().input.repeat_at, Some(5000));
    let x = game.view().x;
    game.advance_to(4999).unwrap();
    assert_eq!(game.view().x, x);
    game.advance_to(5000).unwrap();
    assert!(game.view().x < x);
}

#[test]
fn tap_rotation_accumulates_only_between_pieces_and_survives_release() {
    let mut game = game(
        Handling {
            irs: BufferMode::Tap,
            ..Handling::default()
        },
        6000,
    );
    input(&mut game, 0, Action::HardDrop, true);
    assert!(!game.view().active);
    input(&mut game, 1000, Action::Clockwise, true);
    input(&mut game, 1100, Action::Clockwise, false);
    input(&mut game, 2000, Action::Clockwise, true);
    input(&mut game, 2100, Action::Clockwise, false);
    let snapshot = game.snapshot();
    let mut restored = Game::from_snapshot(snapshot).unwrap();
    game.advance_to(6000).unwrap();
    restored.advance_to(6000).unwrap();
    assert_eq!(game.snapshot().rotation, 2);
    assert!(game.view().active);
    assert_eq!(
        serde_json::to_value(game.snapshot()).unwrap(),
        serde_json::to_value(restored.snapshot()).unwrap()
    );
    input(&mut game, 7000, Action::HardDrop, true);
    game.advance_to(13000).unwrap();
    assert_eq!(game.snapshot().rotation, 0);
}

#[test]
fn hold_buffer_uses_keys_at_spawn_and_off_ignores_them() {
    for mode in [BufferMode::Off, BufferMode::Hold, BufferMode::Tap] {
        for released in [false, true] {
            let mut game = game(
                Handling {
                    irs: mode,
                    ihs: mode,
                    ..Handling::default()
                },
                6000,
            );
            input(&mut game, 0, Action::HardDrop, true);
            let next = game.snapshot().queue[0];
            input(&mut game, 1000, Action::Hold, true);
            input(&mut game, 1100, Action::Clockwise, true);
            if released {
                input(&mut game, 2000, Action::Hold, false);
                input(&mut game, 2100, Action::Clockwise, false);
            }
            game.advance_to(6000).unwrap();
            let applied = mode == BufferMode::Tap || (mode == BufferMode::Hold && !released);
            assert_eq!(game.snapshot().hold, applied.then_some(next));
            assert_eq!(game.snapshot().held, applied);
            assert_eq!(game.snapshot().rotation, u8::from(applied));
        }
    }
}

#[test]
fn release_inputs_clears_pending_buffers() {
    let mut game = game(
        Handling {
            irs: BufferMode::Tap,
            ihs: BufferMode::Tap,
            ..Handling::default()
        },
        6000,
    );
    input(&mut game, 0, Action::HardDrop, true);
    input(&mut game, 1000, Action::Hold, true);
    input(&mut game, 1100, Action::Clockwise, true);
    game.release_inputs();
    game.advance_to(6000).unwrap();
    assert_eq!(game.snapshot().rotation, 0);
    assert_eq!(game.snapshot().hold, None);
}

#[test]
fn automatic_lock_guard_expires_at_exact_tick_and_does_not_guard_manual_locks() {
    let mut rules = Rules::solo(Mode::Zen, false);
    rules.automatic_lock = true;
    let mut game = Game::new(
        42,
        rules,
        Handling {
            safe_lock_delay: 3000,
            ..Handling::default()
        },
    )
    .unwrap();
    input(&mut game, 0, Action::SoftDrop, true);
    input(&mut game, 1, Action::SoftDrop, false);
    game.advance_to(30000).unwrap();
    assert_eq!(game.view().placed, 1);
    input(&mut game, 32999, Action::HardDrop, true);
    assert_eq!(game.view().placed, 1);
    input(&mut game, 33000, Action::HardDrop, true);
    assert_eq!(game.view().placed, 2);
    input(&mut game, 33001, Action::HardDrop, true);
    assert_eq!(game.view().placed, 3);
}

#[test]
fn sonic_drop_priority_changes_which_side_of_a_ledge_is_reached() {
    for priority in [false, true] {
        let mut game = game(
            Handling {
                das: 6000,
                prefer_soft_drop: priority,
                ..Handling::default()
            },
            0,
        );
        let mut snapshot = game.snapshot();
        snapshot.piece = Piece::O;
        snapshot.x = 3;
        snapshot.y = 0;
        snapshot.lowest_y = 0;
        snapshot.board[25][0] = Some(Piece::J);
        snapshot.board[25][1] = Some(Piece::J);
        snapshot.input.left = true;
        snapshot.input.direction = -1;
        snapshot.input.charged = true;
        game = Game::from_snapshot(snapshot).unwrap();
        input(&mut game, 0, Action::SoftDrop, true);
        assert_eq!(game.view().y, if priority { 18 } else { 3 });
    }
}

#[test]
fn finite_drop_works_without_gravity_and_settings_reject_invalid_values() {
    let mut game = game(
        Handling {
            soft_drop_interval: 1500,
            ..Handling::default()
        },
        0,
    );
    let y = game.view().y;
    input(&mut game, 0, Action::SoftDrop, true);
    game.advance_to(1499).unwrap();
    assert_eq!(game.view().y, y + 1);
    game.advance_to(1500).unwrap();
    assert_eq!(game.view().y, y + 2);
    assert!(
        Handling {
            safe_lock_delay: 60001,
            ..Handling::default()
        }
        .validate()
        .is_err()
    );
}
