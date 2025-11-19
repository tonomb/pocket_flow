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
- GUI app using eframe/egui for Pomodoro timer with work/break modes and fullscreen break periods
