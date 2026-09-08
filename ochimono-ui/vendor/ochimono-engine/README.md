# ochimono-engine

Deterministic falling-block simulation for native Rust and WebAssembly.

The engine is a single crate. `Game` owns the board, bag generator, input state,
timers, and solo run state. Rendering, browser events, authentication, persistence,
and matchmaking belong to its callers. The optional `wasm` feature exposes the
same simulation through `src/wasm.rs`; it does not implement a second rules engine.

## Build

```sh
cargo test --all-features
cargo clippy --all-targets --all-features -- -D warnings
just wasm-setup
just wasm-npm-pack
just wasm-vendor
```

The package command requires `wasm-pack` and produces a bundler package in `pkg-npm`.
`wasm-vendor` also copies its runtime files into the UI's tracked vendor directory;
run `pnpm install` in `ochimono-ui` after regenerating the package.
It installs the matching wasm-bindgen tool when necessary. No Rust toolchain file
is required; CI selects its compiler and local development uses the installed one.

Native tests and browser Vitest tests replay the same checked-in fixture and
compare the full snapshot after each input. `cargo run --example replay_fixture`
prints `tests/fixtures/replay.json` for deliberate ruleset updates. The vendor
command copies this fixture to the UI tests. Regenerate the WASM package
together with it; changing expected output alone is not a compatibility check.

## Rules and reproducibility

`Rules` holds gravity and lock policy; `Handling` holds DAS, ARR, DCD and an
explicit soft-drop interval. Time uses integer ticks: 60 ticks per millisecond,
1,000 per 60 Hz frame. Inputs carry simulation timestamps. Timers at a timestamp
run before inputs at that timestamp, independent of render cadence.

Snapshots include the ChaCha8 generator, queue, board, timers and held inputs.
Restoring a snapshot resumes the same sequence. Undo/redo history is local to a
`Game` instance and is not included in an exported snapshot. Seeds and bag order
are Ochimono's replay format, not TETR.IO replay compatibility.

`solo-v1` is an initial practice ruleset, **not a claim of current TETR.IO parity**.
Its 0.02 G gravity, 30-frame lock delay and 15-reset limit come from historical
benchmark material. Current spawn/lock-reset boundaries, finite SDF scaling, IRS,
IHS, safe-lock, Zen recovery and scoring still require reference fixtures before
being advertised as equivalent. Soft drop uses an explicit interval so an unknown
SDF formula is not silently substituted. Sprint fixes its rules for a run.
Pieces start entirely above the visible frame. Ochimono represents this with
matrix origin `y = -2`; subsequent gravity changes their position. A delayed
screenshot of a piece crossing the skyline does not establish its initial spawn.
Its 40-row board includes
20 upper buffer rows. `View.board_top = -20` maps the full board to display
coordinates; rendering must not discard negative rows of either the active piece
or the stack. The frame still marks the lower 20 rows.

Above-frame placement is distinct from block-out at the next spawn; see
[the report and osk's confirmation](https://github.com/tetrio/issues/issues/1125#issuecomment-1518452284).
The precise initial spawn check before any immediate one-row descent, and current
mode-specific top-out policies, still require further fixtures.

## Rotation references

The [official mechanics FAQ](https://tetrio.github.io/faq/mechanics.html) links to
SRS+ and the 180-degree rotation diagrams. The diagrams below are by **osk**,
reproduced on Tetris Wiki. They are external reference images, not Ochimono assets.

### SRS+ I-piece kicks

![SRS+ I-piece kick order by osk](https://tetris.wiki/images/7/7d/TETR.IO_SRS%2Bkicks.png)

[Image attribution and source](https://tetris.wiki/File:TETR.IO_SRS%2Bkicks.png).
Each transition tries the pictured positions in order. The I-piece table in
`src/piece.rs` transcribes those offsets; JLSTZ use the standard SRS table.

### 180-degree kicks

![180-degree kick order by osk](https://tetris.wiki/images/5/52/TETR.IO_180kicks.png)

[Image attribution and source](https://tetris.wiki/File:TETR.IO_180kicks.png),
[author's original diagram link](https://github.com/tetrio/issues/issues/478).
Kick offsets use positive Y upward; board rows use positive Y downward, so the
simulation subtracts the kick's Y component. O rotation leaves occupied cells
unchanged. These historical diagrams establish table provenance, not verification
of every behavior in the live game.

## Existing foundations considered

Reviewed on 2026-09-09:

| Candidate | Relevant foundation | Fit for this engine |
| --- | --- | --- |
| [Cold Clear libtetris](https://github.com/MinusKelvin/cold-clear/tree/master/libtetris) | Board, piece movement and SRS used by a playable versus bot | Archived; private rotation implementation uses fixed standard SRS offsets, with CW/CCW entry points. Adopting it would still require replacing rotation and adding the handling timeline. |
| [deep-trinity](https://github.com/s-shin/deep-trinity) | Core/grid crates and web bindings | Repository last pushed in 2023; the inspected material does not establish production use or current TETR.IO compatibility. |
| [tetris-engine-in-rust](https://github.com/nyctivoe/tetris-engine-in-rust) | SRS+, attacks and Rust/Python parity fixtures | A recent implementation with 14 commits; Python parity is not live TETR.IO parity. Not established as a mature dependency. |

No candidate has been adopted as a runtime dependency on this evidence. Rotation
and simulation remain small, separately testable modules. RNG uses the maintained
[rand_chacha](https://docs.rs/rand_chacha/) implementation with portable reference
vectors instead of a custom random generator.
