# Reference verification

Checked on 2026-09-09. Reference-document checks and live-client measurements
are recorded separately. Passing native/WASM tests is not evidence of live
TETR.IO parity.

## Completed checks

| Subject | Evidence | Result |
| --- | --- | --- |
| SRS+ I quarter turns | osk's [I-kick diagram](https://tetris.wiki/File:TETR.IO_SRS%2Bkicks.png), published copy dated 2020-12-24 | All eight transitions and 40 ordered offsets match. |
| T half turns | osk's [180 diagram](https://tetris.wiki/File:TETR.IO_180kicks.png), published copy dated 2020-12-23; linked from [issue 478](https://github.com/tetrio/issues/issues/478) | All four transitions and 24 ordered offsets match. This check does not independently establish I/O half-turn behavior. |
| Applying kicks | `tests/rotation_reference.rs` | For each of the 64 candidates, a board configuration rejects all preceding candidates while keeping the source and expected destination clear. The simulation reaches the expected position, orientation and kick index. |
| IRS/IHS before block-out | Official [Beta 1.5.0](https://tetr.io/about/patchnotes/#chlog_BETA_1_5_0) and [Beta 1.5.1](https://tetr.io/about/patchnotes/#chlog_BETA_1_5_1) notes | Found and fixed an ordering bug: the old engine declared block-out before trying IRS. It now prepares the piece, applies IHS and IRS/kicks, then checks block-out. |

The spawn-collision regression failed before the fix and passes after it. It
covers IRS alone and IHS followed by IRS, with a T piece whose default orientation
overlaps a block but whose rotated orientation is clear. Initial rotation does
not consume an active-piece lock reset or apply DCD twice.

The reference tables are transcribed in
`tests/fixtures/rotation-reference.json`. Their source diagrams were visually
inspected; the expected offsets are not generated from the engine implementation.

## Confirmed gaps

The official Beta 1.5.0 notes describe moving the next piece upward when a line
clear would otherwise cause immediate top-out. Beta 1.5.1 explicitly enables
this in Zen. Ochimono currently allows upper-buffer placement and line clears,
but does **not** implement this additional Clutch Clear spawn recovery. These
are different behaviors; upper-buffer tests do not verify Clutch Clears.

For finite SDF, the [author's 2020 explanation](https://github.com/tetrio/issues/issues/182)
establishes gravity dependence. The official Alpha 2.2.3 notes subsequently
describe changing the minimum speed to depend on SDF. The
[mechanics FAQ](https://tetrio.github.io/faq/mechanics.html#soft-drop-factor-sdf)
also states that soft drop works at zero gravity. These sources do not specify
the current numeric conversion. No multiplier or zero-gravity constant has
been inferred from them.

## Live measurements still required

- Finite SDF at several factors and gravity values, including zero gravity.
- Exact spawn position and first gravity step, per piece and mode.
- Clutch Clear recovery distance, limits and ordering relative to IRS/IHS.
- Safe-lock duration and exact expiry boundary.
- DCD during uncharged DAS, failed rotation and repeated rotations.
- Multiple simultaneous IRS keys and exact-deadline input ordering.
- Mode-specific lock-out and lock-reset boundaries.

The available signed-in browser reported an active Tetra League match, so game
interaction was stopped and the temporary tab was closed. A separate headless
session was rejected by the site before gameplay. No user-agent workaround was
attempted. No timed live measurements were obtained in this verification pass.
Account handling settings were not changed.
