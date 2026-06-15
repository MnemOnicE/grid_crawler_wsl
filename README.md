# Grid Crawler WSL

[![CI](https://github.com/MnemOnicE/grid_crawler_wsl/actions/workflows/ci.yml/badge.svg)](https://github.com/MnemOnicE/grid_crawler_wsl/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-stable-orange.svg)](https://www.rust-lang.org)

A terminal-based tank crawler with seeded maze generation, pickups, WebSocket mobile control, and tactical grid combat.

## Demo

Terminal view (8x8 grid with tactical display):
```
┌─ BATTLEFIELD ─────────────────────┐
│ ··  ··  ··  ··  ██  ··  ··  ··    │
│ ··  ··  ██  ··  ··  ··  ⊙⊙  ··    │
│ ··  ··  ··  ··  ··  ··  ··  ··    │
│ ██  ··  ··  ··  ··  ··  ··  ··    │
│ ··  ··  ··  ··  ··  ··  ··  ··    │
│ ··  ◆◆  ··  ··  ··  ··  ··  ··    │
│ ··  ··  ··  ··  ··  ··  ··  ··    │
│ ··  ··  ··  ··  ··  ··  ··  ✖✖    │
└────────────────────────────────────┘

Legend:
  ⊙⊙ = Player tank
  ✖✖ = Enemy
  ◆◆ = Pickup
  ██ = Wall/cover
  ░░ = Wreckage
  ·· = Empty space
```

Mobile client (browser at `http://127.0.0.1:9001/`):
- On-screen directional buttons for movement and firing
- Real-time map updates via WebSocket
- AP, HP, and armor gauges

## Overview

`grid_crawler_wsl` is a lightweight Rust game that runs in the terminal via `crossterm` and `ratatui`, while exposing a mobile-friendly WebSocket interface for remote control.

Key features:

- Seeded random map generation for reproducible runs
- Terminal UI with a tactical grid viewport
- Player movement, action points, pickups, and hazards
- Mobile WebSocket client served at `http://127.0.0.1:9001/`
- Versioned CLI support via `--version`
- Continuous integration support with `cargo fmt`, `cargo clippy`, and `cargo test`

## Quick Start

### Prerequisites

- Rust stable toolchain
- Cargo

### Build and run

```bash
cd grid_crawler_wsl
cargo run --release --
```

Open a browser to:

```text
http://127.0.0.1:9001/
```

### CLI examples

```bash
cargo run --release -- --help
cargo run --release -- --version
```

### Environment overrides

```bash
GRID_SEED=42 GRID_SIZE=12 cargo run --release --
```

## Controls

### Terminal controls

- `WASD` or arrow keys: move the tank
- `Space`: reserved for fire/action
- `S` during the start screen: change map size
- `G` during the start screen: randomize seed
- `Enter` on start screen: begin game
- `ESC`: quit

### Mobile client controls

After opening the mobile page, use the on-screen buttons to:

- Move up/down/left/right
- Fire up/down/left/right

### WebSocket command format

The mobile client sends JSON messages like:

```json
{ "action": "move", "direction": "up" }
{ "action": "fire", "direction": "right" }
```

## Project structure

- `Cargo.toml` - Rust package metadata and dependencies
- `src/main.rs` - Terminal game loop, UI, and CLI support
- `src/state.rs` - Game state, map generation, movement, pickups, and spawn logic
- `src/net.rs` - WebSocket server and mobile interface
- `src/serial_daemon.rs` - Serial hardware fallback and bridge logic
- `.github/workflows/ci.yml` - CI build/test workflow

## Versioning

The current project version is tracked in `Cargo.toml` and exposed by the CLI via `--version`.
A separate `VERSION` file is also included for tooling and release workflows.

## Contributing

1. Fork the repository.
2. Create a new branch for your feature or fix.
3. Run `cargo fmt` and `cargo clippy -- -D warnings`.
4. Add or update tests.
5. Submit a pull request.

## Roadmap

See [ROADMAP.md](ROADMAP.md) for planned features, improvements, and future milestones.

## License

This project is licensed under the [MIT License](LICENSE).
