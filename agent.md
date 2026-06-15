# Grid Crawler WSL Agent Guide

This repository uses a VS Code AI-assisted workflow. The `agent.md` file documents how to contribute to and extend the project using automated support.

## Goals

- Keep the project lightweight and testable.
- Maintain a clear separation between terminal UI, game state, networking, and hardware fallback.
- Prefer small, incremental changes with unit tests.

## Useful commands

- `cargo fmt` — format the Rust source code.
- `cargo clippy -- -D warnings` — lint the code and treat warnings as errors.
- `cargo test` — run the test suite.
- `cargo run --release --` — run the terminal game.
- `cargo run --release -- --help` — print CLI usage.
- `cargo run --release -- --version` — print the project version.

## File responsibilities

- `src/main.rs` — application entrypoint, TUI rendering, input handling, and lifecycle control.
- `src/state.rs` — game engine, map generation, pickups, and action validation.
- `src/net.rs` — WebSocket server and simple browser UI.
- `src/serial_daemon.rs` — hardware abort/fallback bridge.
- `.github/workflows/ci.yml` — CI configuration for formatting, linting, and testing.

## Documentation updates

When adding features, update:

- `README.md` for user-facing launch instructions
- `ROADMAP.md` for planned work and priorities
- `Cargo.toml` for package metadata and version changes

## Version control

- Use Git tags or the `VERSION` file for release numbering.
- Keep `Cargo.toml` `version` synchronized with `VERSION`.
