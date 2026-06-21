# Debug harness (REMOVABLE)

Scripted-input + screenshot tooling for diagnosing the UI without a human
at the mouse. **Gated behind the `automation` cargo feature — never in a
ship build.** To rip out: delete this `debug/` dir, the `automation`
feature in both `Cargo.toml`s, `src/debug_config.rs`, the
`#[cfg(feature = "automation")]` lines in `src/main.rs`, and the
`#[cfg(feature = "automation")]` blocks + `src/automation.rs` in
`../opal-gfx`.

## Run

```
./debug/run.ps1            # Windows; uses debug/home.json
./debug/run.sh             # Linux/macOS
./debug/run.ps1 debug/liked.json
```

The launcher kills any stale `opal.exe` (the lock that breaks
`cargo run` mid-session), builds with `--features automation`, and runs
`--config <path>`.

## Config (JSON)

```json
{
  "force_home": true,            // boot to Home (skip Splash/Login)
  "window": [1280, 780],         // logical-px window size
  "log_filter": "info,...",      // env_logger override
  "script": [ <steps> ]          // optional; empty = launch + sit
}
```

### Steps (one action field each)

| field | meaning |
|---|---|
| `{ "wait_ms": 1500 }` | idle (let art/worker/animations land) |
| `{ "screenshot": "path.png" }` | render + write PNG |
| `{ "move_mouse": [x, y] }` | move cursor |
| `{ "click": [x, y] }` | move + left press/release (fires `on_click`) |
| `{ "hover": [x, y], "dwell_ms": 400 }` | move + dwell (hover tint + tooltip) |
| `{ "scroll": [x, y], "by": [dx, dy] }` | scroll by wheel lines at a point |
| `{ "drag": [[x1,y1],[x2,y2]] }` | press, move, release |

**Coordinates are PHYSICAL pixels** (top-left origin) = logical × DPI
scale. On a 2× display a `1280×780` window is `2560×1560` physical, so a
logical `(60, 120)` target is `(120, 240)`. Tune coordinates by taking a
screenshot first and reading pixel positions off it.

Synthetic input goes through the *same* handlers as real winit events, so
hover/click/scroll behave exactly like a user. The script self-terminates
(exits the app) when it ends.
