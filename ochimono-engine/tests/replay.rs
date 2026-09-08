use ochimono_engine::{Game, Handling, Input, Mode, Rules};
use serde::Deserialize;

#[derive(Deserialize)]
struct Fixture {
    seed: String,
    mode: Mode,
    handling: Handling,
    inputs: Vec<Input>,
    expected: Vec<serde_json::Value>,
}

#[test]
fn native_engine_matches_the_browser_replay_fixture() {
    let fixture: Fixture = serde_json::from_str(include_str!("fixtures/replay.json")).unwrap();
    let mut game = Game::new(
        fixture.seed.parse().unwrap(),
        Rules::solo(fixture.mode, true),
        fixture.handling,
    )
    .unwrap();
    assert_eq!(fixture.inputs.len(), fixture.expected.len());
    for (input, expected) in fixture.inputs.into_iter().zip(fixture.expected) {
        game.input(input).unwrap();
        assert_eq!(serde_json::to_value(game.snapshot()).unwrap(), expected);
    }
}
