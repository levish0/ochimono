use ochimono_engine::{Action, Game, Handling, Input, Mode, Rules};

fn main() {
    let handling = Handling {
        das: 8_400,
        arr: 1_800,
        dcd: 0,
        soft_drop_interval: 1_500,
    };
    let mut game = Game::new(42, Rules::solo(Mode::Sprint, true), handling.clone()).unwrap();
    let inputs = [
        Input {
            at: 0,
            action: Action::Left,
            pressed: true,
        },
        Input {
            at: 10_000,
            action: Action::Clockwise,
            pressed: true,
        },
        Input {
            at: 12_000,
            action: Action::Left,
            pressed: false,
        },
        Input {
            at: 15_000,
            action: Action::HardDrop,
            pressed: true,
        },
        Input {
            at: 20_000,
            action: Action::Hold,
            pressed: true,
        },
        Input {
            at: 25_000,
            action: Action::SoftDrop,
            pressed: true,
        },
        Input {
            at: 55_000,
            action: Action::SoftDrop,
            pressed: false,
        },
        Input {
            at: 60_000,
            action: Action::Counterclockwise,
            pressed: true,
        },
        Input {
            at: 65_000,
            action: Action::Right,
            pressed: true,
        },
        Input {
            at: 80_000,
            action: Action::Right,
            pressed: false,
        },
        Input {
            at: 90_000,
            action: Action::HardDrop,
            pressed: true,
        },
    ];
    let mut expected = Vec::new();
    for input in &inputs {
        game.input(*input).unwrap();
        expected.push(game.snapshot());
    }
    println!("{}", serde_json::to_string_pretty(&serde_json::json!({
        "seed": "42", "mode": "sprint", "handling": handling, "inputs": inputs, "expected": expected
    })).unwrap());
}
