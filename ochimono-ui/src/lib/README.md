# Source organization

- `game/`: board rules and background playback planning. Keep these independent of rendering, browser APIs, and UI state. Rule tests live beside their implementation.
- `game-client/`: application lifecycle, screen composition, input routing, and persisted client settings. `GameClient` connects rules and presentation.
- `game-ui/components/`: reusable Pixi presentation grouped by purpose: `navigation`, `menu`, and `background`.
- `game-ui/motion/`: shared animation timing and interpolation, with colocated tests.
- `game-ui/scroll/`: scroll behavior shared by UI surfaces.
- `game-ui/rendering/`: text and icon rendering helpers.
- `game-ui/audio/`: interaction and gameplay sound feedback.
- `paraglide/`: generated localization output; edit source messages instead.

Components receive actions through callbacks; they must not import `GameClient` or own browser storage. Screen-specific composition currently lives in `GameClient`; extract new reusable controls into the matching component group. Svelte routes mount the client and provide the accessible HTML shell.
