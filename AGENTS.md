# Agent Guidelines for pocket_flow

## Build/Test Commands
- **Build**: `cargo build` (dev) or `cargo build --release` (optimized)
- **Run**: `cargo run` (dev) or `cargo run --release`
- **Test**: `cargo test` (all tests) or `cargo test <test_name>` (single test)
- **Lint**: `cargo clippy -- -D warnings`
- **Format**: `cargo fmt` (check: `cargo fmt -- --check`)

## Code Style
- **Edition**: Rust 2021
- **Imports**: Group std library first, then external crates (eframe, chrono), separated by blank line
- **Constants**: SCREAMING_SNAKE_CASE with explicit types (e.g., `const WORK_DURATION: u64 = 25 * 60`)
- **Enums**: Use `#[derive(PartialEq, Clone, Copy)]` for simple state enums
- **Structs**: Implement `Default` trait when appropriate
- **Methods**: Group impl methods logically (state transitions, UI updates, helpers)
- **Formatting**: 4-space indentation, 100 char line limit recommended
- **Error Handling**: Use `Result<()>` for main, propagate errors with `?`
- **Comments**: Use `//` for single-line, keep commented-out code minimal and explained

## Project Context
- GUI app using eframe/egui for Pomodoro timer with work / break / overflow modes and fullscreen break periods.

## Pomodoro Modes

The app has three runtime modes, modeled by the `PomodoroMode` enum in `src/main.rs`. The code uses `Overflow` as the identifier; the user-facing UX label is **"Pocket"**.

### 1. Work mode (`PomodoroMode::Work`)
- **Duration**: `WORK_DURATION` = 45 minutes (`src/main.rs`).
- **Purpose**: Focused work session. The window minimizes when the timer starts so the user is not distracted.
- **Persistence**: Successful completion saves a `WorkSession` to SQLite (`db.save_work_session`) and increments the today-dots counter. Uncompleted sessions (Restart, app exit) are not saved.
- **On completion**: Auto-transitions into Break mode (fullscreen).
- **UI controls**: Start (when stopped), Pause / Resume (when running / paused), Restart (when not stopped).
- **Keybindings**: None mode-specific. Buttons only.

### 2. Break mode (`PomodoroMode::Break`)
- **Duration**: `BREAK_DURATION` = 15 minutes.
- **Purpose**: Rest period. The window enters fullscreen to enforce the break.
- **Persistence**: None — breaks are not recorded.
- **On completion**: Returns to a stopped Work timer (`finish_break`) at full duration.
- **UI controls**:
  - While running: **Pocket (5 min)** + **Skip Break**.
  - When break time hits zero: **Start New Timer**.
- **Keybindings** (only while `remaining_seconds > 0`):
  - `Enter` — `skip_break`: end the break immediately and start a fresh running 45-minute Work timer.
  - `P` — `start_overflow`: enter Overflow ("Pocket") mode for a short wrap-up timer.
  - `Esc` — `minimize_break_window`: exit fullscreen but keep the break running in the menu bar.

### 3. Overflow mode (`PomodoroMode::Overflow`, UX: "Pocket")
- **Duration**: `OVERFLOW_DURATION` = 5 minutes.
- **Purpose**: A short, configurable wrap-up window for the user to finish a train of thought when the 45-minute work timer ends but the break is starting. Triggered from the Break view via the **Pocket (5 min)** button or the `P` key.
- **Persistence**: None — overflow is treated as an extension of the just-completed work session, not a new session. The today-dots counter does **not** advance.
- **On completion**: Auto-transitions into a running Break timer via `start_break(ctx)`. The contract is asserted by `overflow_finished_state()` for tests.
- **Chaining**: Once the break starts, `P` and `Enter` are available again on the Break view, so a user can chain Pocket → Break starts → Pocket again, or Pocket → Break starts → Enter (skip break, fresh 45-min work).
- **UI**: Reuses the Work view layout with the title swapped to **"Wrapping Up"** in `COLOR_ACCENT`. Two dedicated buttons replace the Work-mode Pause / Resume / Restart controls:
  - **Start New Session** — abandons the wrap-up and starts a fresh running 45-minute Work timer (delegates to `skip_break`).
  - **Start Break Early** — ends the wrap-up immediately and begins the fullscreen Break timer (delegates to `start_break`).
- **Tray title**: Prefixed with `P ` (e.g. `P 04:32`, `P 04:32 (Paused)`) so the menu-bar shows when overflow time is running.
- **Keybindings**: None mode-specific. Buttons only.

## Mode Transition Summary

```
Work (45m) --completes--> Break (15m) --completes--> Work (stopped, 45m)
                          |
                          |-- Enter / Skip Break btn --> Work (running, 45m)
                          |
                          |-- P / Pocket btn -----------> Overflow (running, 5m)
                                                          |
                                                          --completes--> Break (running, 15m)
```

## Timer Mechanism

The countdown uses **absolute wall-clock deadlines** (`end_time: Option<DateTime<Utc>>`) rather than tick-based deltas. This makes the timer immune to macOS screen-lock / sleep suspending the process — when the computer wakes, the timer shows the correct remaining time.

- **`remaining_until(deadline, now)`** — pure function that computes remaining seconds from an absolute deadline. Returns `(remaining_seconds, deadline_reached)`.
- **`compute_deadline(now, duration_secs)`** — pure function that computes the absolute `DateTime<Utc>` when a timer should reach zero.
- **`end_time`** field on `PomodoroApp` — `Some` when the timer is running, `None` when stopped or paused.
- **Pause** snapshots `remaining_seconds` from `end_time - now` before clearing `end_time`.
- **Resume** computes a new `end_time` from the paused `remaining_seconds`.
- **`update_timer()`** calls `remaining_until(end_time, Utc::now())` each frame; only updates the display when the whole-second value changes.

## Configurability
Per the current implementation, durations are code constants in `src/main.rs` (`WORK_DURATION`, `BREAK_DURATION`, `OVERFLOW_DURATION`). There is no settings UI or persisted preference — adjust the constants and rebuild to change durations.
