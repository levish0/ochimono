use ochimono_engine::piece::{Piece, kicks};
use ochimono_engine::{Action, Game, Handling, Input, Mode, Rules};
use serde::Deserialize;

#[derive(Deserialize)]
struct Reference {
    cases: Vec<Case>,
}

#[derive(Deserialize)]
struct Case {
    piece: Piece,
    from: u8,
    to: u8,
    kicks: Vec<(i32, i32)>,
}

fn reference() -> Reference {
    serde_json::from_str(include_str!("fixtures/rotation-reference.json")).unwrap()
}

#[test]
fn kick_offsets_and_order_match_the_published_diagrams() {
    for case in reference().cases {
        assert_eq!(
            kicks(case.piece, case.from, case.to),
            case.kicks,
            "{:?}: {} -> {}",
            case.piece,
            case.from,
            case.to
        );
    }
}

fn cells(piece: Piece, rotation: u8, x: i32, y: i32) -> Vec<(i32, i32)> {
    piece
        .cells(rotation)
        .iter()
        .map(|(dx, dy)| (x + dx, y + dy))
        .collect()
}

fn inside(cells: &[(i32, i32)]) -> bool {
    cells
        .iter()
        .all(|(x, y)| (0..10).contains(x) && (-20..20).contains(y))
}

#[test]
fn every_reference_kick_is_reachable_and_earlier_candidates_are_rejected() {
    let base = Game::new(42, Rules::solo(Mode::Zen, false), Handling::default())
        .unwrap()
        .snapshot();
    let mut checked = 0;
    for case in reference().cases {
        for (index, (dx, dy)) in case.kicks.iter().copied().enumerate() {
            let mut verified = false;
            'position: for y in -20..20 {
                for x in -3..10 {
                    let before = cells(case.piece, case.from, x, y);
                    let after = cells(case.piece, case.to, x + dx, y - dy);
                    if !inside(&before) || !inside(&after) {
                        continue;
                    }
                    let mut blockers = Vec::new();
                    for (earlier_x, earlier_y) in &case.kicks[..index] {
                        let earlier = cells(case.piece, case.to, x + earlier_x, y - earlier_y);
                        if !inside(&earlier) {
                            continue;
                        }
                        if let Some(blocker) = earlier
                            .iter()
                            .find(|cell| !before.contains(cell) && !after.contains(cell))
                        {
                            blockers.push(*blocker);
                        } else {
                            continue 'position;
                        }
                    }
                    let mut state = base.clone();
                    state.piece = case.piece;
                    state.rotation = case.from;
                    state.x = x;
                    state.y = y;
                    state.lowest_y = y;
                    for (bx, by) in blockers {
                        state.board[(by + 20) as usize][bx as usize] = Some(Piece::Z);
                    }
                    let mut game = Game::from_snapshot(state).unwrap();
                    let action = match (case.to + 4 - case.from) % 4 {
                        1 => Action::Clockwise,
                        2 => Action::HalfTurn,
                        3 => Action::Counterclockwise,
                        _ => unreachable!(),
                    };
                    game.input(Input {
                        at: 0,
                        action,
                        pressed: true,
                    })
                    .unwrap();
                    let actual = game.snapshot();
                    assert_eq!(
                        (actual.x, actual.y, actual.rotation, actual.last_kick),
                        (x + dx, y - dy, case.to, Some(index)),
                        "{:?}: {} -> {}, candidate {index}",
                        case.piece,
                        case.from,
                        case.to
                    );
                    verified = true;
                    checked += 1;
                    break 'position;
                }
            }
            assert!(
                verified,
                "No witness for {:?}: {} -> {}, candidate {index}",
                case.piece, case.from, case.to
            );
        }
    }
    assert_eq!(checked, 64);
}
